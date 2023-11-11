pub mod actor;
pub mod group;
pub mod example;

pub use actor::*;
pub use group::*;
pub use example::*; 

use crate::error;
use crate::model::{generate_model, Lib, Model};

use syn::{ punctuated::Punctuated,ItemImpl,Meta,Token,Attribute };
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;


pub enum AttributeArguments {
    Actor(ActorAttributeArguments),
    Group(GroupAttributeArguments),
}

impl AttributeArguments {

    pub fn from(    nested: Punctuated::<syn::Meta,syn::Token![,]>,
                       mac: &Model ) -> Self {

        match mac {
            Model::Actor => { 
                let mut aaa = ActorAttributeArguments::default();
                aaa.parse_nested(nested);
                Self::Actor(aaa)
            },
            Model::Group => { 
                let mut gaa = GroupAttributeArguments::default();
                gaa.parse_nested(nested);
                Self::Group(gaa)
            },
        }
    }

    pub fn get_lib(&self) -> Lib {

        match self {
            Self::Actor(aaa) => aaa.lib.clone(),
            Self::Group(gaa) => gaa.lib.clone(),
        }
    }

    pub fn get_mac(&self) -> Model {
        match &self {
            Self::Actor(_) => Model::Actor,
            Self::Group(_) => Model::Group,
        }
    }

    pub fn cross_check(&mut self, item_impl: &ItemImpl){

        match self {
            Self::Actor(aaa) => { aaa.cross_check(); },
            Self::Group(gaa) => { gaa.cross_check(item_impl); },
        }
    }

    pub fn generate_code( self, item_impl: &ItemImpl )  -> (TokenStream,TokenStream){
        let model_sdpl = generate_model(self, item_impl, None );
        let (mut code,edit) = model_sdpl.get_code_edit();

        code = quote!{
    
            #item_impl
            #code
        };
        (code,edit)
    }
}

pub fn to_usize(value: &syn::LitInt) -> usize {
        
    let msg  = format!("Expected a positive integer 1..{:?}.", usize::MAX );
    value.base10_parse::<usize>()
         .unwrap_or_else(|_| abort!(value,msg))   
} 

pub fn get_list(meta: &syn::Meta, help: Option<&str>) -> Option<Punctuated::<syn::Meta,syn::Token![,]>> {
    match meta {
        syn::Meta::Path(_) => { None },
        syn::Meta::List(meta_list) => { 
            if let Ok(list) = 
            meta_list.parse_args_with(Punctuated::<syn::Meta,syn::Token![,]>::parse_terminated){
                Some(list) 
            } else { 
                let msg = "Internal Error.'attribute::mod::get_list'. Could not parse punctuated!";
                abort!(meta,msg);
            }
        },
        syn::Meta::NameValue(_) => { 
            if let Some(help) = help {
                abort!(meta,error::EXPECT_LIST; help=help) 
            } else { None }
        },
    }
}

pub fn get_lit_str( meta: &syn::Meta ,arg: &str ) -> String {
    match get_lit(meta) {
        syn::Lit::Str(val) => {  
            let string = val.value();
            if &string == "" {
                let msg = format!("Attribute argument '{arg}' value is empty.");
                abort!(val,msg); 
            } 
            string
        },
        v => abort!(v, error::error_name_type( &meta.path(), "str"); help=error::AVAIL_ACTOR ),
    }
}


pub fn get_lit( meta: &syn::Meta ) -> syn::Lit {

    let msg = "Expected a 'name = value' argument !";
    match meta {
        syn::Meta::NameValue(nv) => {
            match &nv.value {
                syn::Expr::Lit(expr_lit) => {
                    expr_lit.lit.clone()
                },
                v => abort!(v, msg),
            }
        },
        m => abort!(m, msg),
    }
}

pub fn attr_to_meta_list( attr: &Attribute) -> Punctuated::<Meta,Token![,]> {

    if let syn::Meta::List(_) = attr.meta { 
        match attr.parse_args_with(Punctuated::<Meta,Token![,]>::parse_terminated){
            Ok(p) => p,
            Err(e) => {
                let msg = format!("Internal Error.'attribute::mod::attr_to_meta'. Failed to parse Attribute to Punctuated. Error {}",e.to_string());
                abort!(attr,msg);
            }
        }
    } 
    else { Punctuated::new() }
}

pub fn get_ident( meta: &syn::Meta ) -> syn::Ident {
    if let Some(ident) = meta.path().get_ident(){
        ident.clone()
    } else { abort!( meta,error::EXPECT_IDENT); }
}

pub fn get_idents( nested: &Punctuated::<Meta,Token![,]> ) -> Vec<syn::Ident> {
    nested.into_iter().map(|m|{
        get_ident(m)
    }).collect::<Vec<_>>()
}

pub fn check_path_set( nested: &Punctuated::<Meta,Token![,]> ){

    if nested.len() > 1 { 

        let mut meta_list = nested.iter().cloned().collect::<Vec<_>>();
        
        for _ in 0..(meta_list.len() -1) {

            if let Some(meta) = meta_list.pop(){
                let path = meta.path();
                if meta_list.iter().any(|x| path.eq(x.path())){
                    let s = quote::quote!(#path).to_string();
                    abort!(meta, error::double_decl( &s.replace(" ","")));
                }
            }
        }
    }
}

pub fn get_ident_group( meta: &Meta,arg: &str) -> syn::Ident {
    let edit_ident = quote::format_ident!("{arg}");
    let self_ident = quote::format_ident!("Self");
    let path = meta.path();
    if path.segments.len() == 2 { 
        if let Some(first_path_segment) =  path.segments.last(){
            if first_path_segment.ident.eq(&edit_ident){
                if let Some(last_path_segment) =  path.segments.first(){
                    let ident =  last_path_segment.ident.clone();
                    if ident.eq(&self_ident){
                        let msg = format!("Expected `self::{arg}`.");
                        abort!(meta.path(),msg);
                    } else { return ident;}
                }
            }
        }
    } 
    abort!(path, error::UNEXPECTED_EDIT_GROUP_PATH; note=error::AVAIL_EDIT_GROUP );
}

