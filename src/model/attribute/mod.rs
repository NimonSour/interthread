pub mod actor;
pub mod example;


pub use actor::*;
pub use example::*; 

use crate::error;

use syn::{parse_quote, punctuated::Punctuated,Meta,Token,Attribute,Path};
use proc_macro_error::abort;
use quote::{format_ident,quote};


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
                let msg = format!("Internal Error|`attribute::mod::attr_to_meta`|: Failed to parse Attribute to Punctuated. Error {}",e.to_string());
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

pub fn check_path_set<'a,I>( nested: I , except: Option<Vec<&'static str>> )
where I: IntoIterator<Item = &'a Meta>,
{

    let except =  
        except.as_ref()
            .map(|xx|
            xx.iter()
                .map(|x| format_ident!("{x}"))
                .map(|i| parse_quote!(#i))
                .collect::<Vec<Path>>()
            );

    let mut meta_list = nested.into_iter().cloned().collect::<Vec<_>>();
    if meta_list.len() > 1 { 
        for _ in 0..(meta_list.len() -1) {

            if let Some(meta) = meta_list.pop(){
                let path = meta.path();
                if meta_list.iter().any(|x| path.eq(x.path())){
                    if let Some(except) = &except {
                        if except.contains(&path) { continue; }
                    }
                    let s = quote!(#path).to_string();
                    abort!(meta, error::double_decl( &s.replace(" ","")));
                }
            }
        }
    }
}

pub fn meta_get_path( meta: &Meta ) -> std::path::PathBuf {
    let file_str = get_lit_str(&meta,crate::FILE);
    let path = std::path::PathBuf::from(&file_str);
    if path.exists() { path }
    else { abort!(meta, format!("Path - {file_str:?} does not exist.")); }
}

