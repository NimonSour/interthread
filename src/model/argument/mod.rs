pub mod channel;
pub mod debut;
pub mod edit;


pub use channel::Channel;
pub use debut::*;
pub use edit::*;



use crate::error;
// use crate::file::get_ident;

use proc_macro2::{Span,TokenStream};
use proc_macro_error::abort;
use quote::{quote,format_ident};
use syn::{Ident};
// use syn::punctuated::Punctuated;

use std::path::PathBuf;




//-----------------------  EXAMPLE EXPAND
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Model {
    Actor,
    Group,
}

impl Model {

    pub fn to_str(&self) -> &'static str {

        match self {
            Self::Actor => crate::ACTOR,
            Self::Group => crate::GROUP,
        }
    }
}

//-----------------------  ACTOR LIB

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Lib {
    Std,
    Smol,
    Tokio,
    AsyncStd,
}

impl Lib {

    pub fn from( s: &syn::LitStr  ) -> Self {

        match s.value() {

            val if val == "std".to_string()       =>  Lib::Std,
            val if val == "smol".to_string()      =>  Lib::Smol,
            val if val == "tokio".to_string()     =>  Lib::Tokio,
            val if val == "async_std".to_string() =>  Lib::AsyncStd,
            val => {
                let msg = format!("Unknown option  -  {:?} for 'channel' ", val);
                abort!( s, msg; help=error::AVAIL_LIB );   
            } 
        }
    }
    
    pub fn method_new_spawn(&self, play_args: &TokenStream, script_name: &Ident) -> TokenStream {

        match &self {
            Lib::Std      => {
                quote!{ std::thread::spawn(|| { #script_name :: play(#play_args) } );}
            },
            Lib::Smol     => {
                quote!{ smol::spawn( #script_name :: play(#play_args) ).detach();} 
            },
            Lib::Tokio    => {
                quote!{ tokio::spawn( #script_name :: play(#play_args) );}
            },
            Lib::AsyncStd => {
                quote!{ async_std::task::spawn( #script_name :: play(#play_args) );}
            },
        }
    }


}

impl Default for Lib {
    fn default() -> Self {
        Lib::Std
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


