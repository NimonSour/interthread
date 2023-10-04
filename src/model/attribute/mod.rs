pub mod actor;
pub mod group;
pub mod example;

pub use actor::*;
pub use group::*;
pub use example::*; 

use crate::error;
use syn::punctuated::Punctuated;
use proc_macro_error::abort;
use std::path::PathBuf;


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


/*
    filter_file  returns  Some(punctuated) if file
                            None 

    this is fo single  Ident(file) options
    applicable for def(file) as well

    in Meta bool ->  out (syn::Ident,bool) 
*/


pub fn edit_ident( meta: &syn::Meta, scope: bool ) -> (syn::Ident,bool) {
    if let Some(ident) = meta.path().get_ident(){
        if let Some(list) = get_list(&meta,Some(&format!("Did you mean '{ident}(file)'."))){
            if let Some(new_list) = filter_file(&list){
                if scope {
                    abort!(new_list,"1The option 'file' is overlapped.";help=error::HELP_EDIT_FILE_ACTOR);
                } else {
                    if new_list.is_empty() {
                        (ident.clone(),true)
                    } else { abort!(new_list, "Unexpected option.";help=error::HELP_EDIT_FILE_ACTOR)}
                }
            } else { abort!(list, "Unexpected option.";help=error::HELP_EDIT_FILE_ACTOR) }
        } else { (ident.clone(),false) }
    } else { abort!( meta, "Expected an identation."); }
}

pub fn filter_file(meta_list: &Punctuated::<syn::Meta,syn::Token![,]>) 
    -> Option<Punctuated::<syn::Meta,syn::Token![,]>>{

    let filtered_list: Punctuated::<syn::Meta,syn::Token![,]> = 
        meta_list.clone()
              .into_iter()
              .filter(|m| !m.path().is_ident("file"))
              .collect();

    if meta_list.len() == filtered_list.len() {
        None
    } else {
        Some(filtered_list)
    }
}


//-----------------------  ACTOR FILE
#[derive(Debug, Eq, PartialEq, Clone)]

pub struct EditAttribute {
    pub path:              PathBuf,
    pub attr:       syn::Attribute,
    pub attrs: Vec<syn::Attribute>,
    pub new_attr:   syn::Attribute,
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


