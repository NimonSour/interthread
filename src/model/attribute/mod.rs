pub mod actor;
pub mod group;
pub mod example;

pub use actor::*;
pub use group::*;
pub use example::*; 

use crate::error;
use proc_macro2::TokenStream;
use crate::model::argument::Model;
use syn::{ punctuated::Punctuated,ItemImpl,Meta,Token,Attribute };
use proc_macro_error::abort;
// use std::path::PathBuf;


//-----------------------  ACTOR CHANNEL 

// #[derive(Debug, Eq, PartialEq, Clone)]
// pub enum AAChannel {

//     Unbounded,
//     Buffer(syn::LitInt),
// }

// impl Default for AAChannel {
//     fn default() -> Self {
//         AAChannel::Unbounded
//     }
// }

// //-----------------------  ACTOR EDIT 




/*
needs a check for methods 
if it finds any methods with a name 
`file` return an error saying that  
active 'file' trigger argument
should be renamed to 'inter_file'.
*/



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

    pub fn get_lib(&self) -> crate::model::argument::Lib {

        match self {
            Self::Actor(aaa) => aaa.lib.clone(),
            Self::Group(gaa) => gaa.lib.clone(),
        }
    }

    pub fn generate_code(&self,item_impl: &ItemImpl)  -> (TokenStream,TokenStream){

        match self {
            Self::Actor(aaa) => {
                super::actor::macro_actor_generate_code(aaa.clone(),item_impl.clone())
            },
            Self::Group(gaa) =>{
                super::group::macro_group_generate_code(gaa.clone(),item_impl.clone())
            } ,
        }
    }

}


//// aux functions for attributes 

fn to_usize(value: &syn::LitInt) -> usize {
        
    let msg  = format!("Expected a positive integer 1..{:?}.", usize::MAX );
    value.base10_parse::<usize>()
         .unwrap_or_else(|_| abort!(value,msg))   
} 

pub fn get_list(meta: &syn::Meta, help: Option<&str>) -> Option<Punctuated::<syn::Meta,syn::Token![,]>> {
    match meta {
        syn::Meta::Path(_) => { None },
        syn::Meta::List(meta_list) => { 
            let list = 
            meta_list.parse_args_with(Punctuated::<syn::Meta,syn::Token![,]>::parse_terminated).unwrap();
            Some(list) 
        },
        syn::Meta::NameValue(_) => { 
            if let Some(help) = help {
                abort!(meta,"Expected a list!"; help=help) 
            } else { None }
        },
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
// Attribute to meta list

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
    // default empty
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

pub fn is_set( nested: &Punctuated::<Meta,Token![,]> ){

    if nested.len() > 1 { 

        let mut meta_list = nested.iter().cloned().collect::<Vec<_>>();
        
        for _ in 0..(meta_list.len() -1) {
            
            if let Some(meta) = meta_list.pop(){
                let ident = get_ident(&meta);

                if meta_list.iter().any(|x| ident.eq(&get_ident(x))){
                    abort!(meta, error::double_decl( &ident.to_string()));
                }
            }
        }
    }
}




