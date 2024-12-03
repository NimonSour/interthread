mod channel;
mod debut;
mod edit;
mod interact;
mod include_exclude;
mod show;
mod receiver;

pub use channel::*;
pub use debut::*;
pub use edit::*;
pub use interact::*;
pub use include_exclude::*;
pub use show::ShowComment;
pub use receiver::*;

use crate::error;

use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::{TypeGenerics, TypePath};

use std::path::PathBuf;




//-----------------------  EXAMPLE EXPAND
#[derive(Debug,Copy, Eq, PartialEq, Clone)]
pub enum Mac {
    Actor,
    Family,
}

impl std::fmt::Display for Mac {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Actor            => write!(f,"{}",crate::ACTOR),
            Self::Family           => write!(f,"{}",crate::FAMILY),
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

    pub fn from( s: &str  ) -> Self {

        match s {
            val if val == "std"       =>  Lib::Std,
            val if val == "smol"      =>  Lib::Smol,
            val if val == "tokio"     =>  Lib::Tokio,
            val if val == "async_std" =>  Lib::AsyncStd,
            val => {
                let msg = format!("Unknown option  -  {:?} for 'channel' ", val);
                abort!( s, msg; help=error::AVAIL_LIB );   
            } 
        }
    }
    
    pub fn method_new_spawn(&self, play_args: &TokenStream, script_turbo: &TypePath, pub_gen_ty: &TypeGenerics) -> TokenStream {
        let pub_turbo = pub_gen_ty.as_turbofish();
        match &self {
            Lib::Std      => {
                quote!{ std::thread::spawn(move|| { #script_turbo :: play #pub_turbo (#play_args) } );}
            },
            Lib::Smol     => {
                quote!{ smol::spawn( #script_turbo :: play #pub_turbo (#play_args) ).detach();} 
            },
            Lib::Tokio    => {
                quote!{ tokio::spawn( #script_turbo :: play #pub_turbo (#play_args) );}
            },
            Lib::AsyncStd => {
                quote!{ async_std::task::spawn( #script_turbo :: play #pub_turbo (#play_args) );}
            },
        }
    }

    pub fn is_std(&self) -> bool {
        if let Self::Std = self {
            return true;
        }
        false
    }


}

impl std::fmt::Display for Lib {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Std => write!(f,"std"),
            Self::Smol => write!(f,"smol"),
            Self::Tokio => write!(f,"tokio"),
            Self::AsyncStd => write!(f,"async_std"),
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
    pub remove:               bool,
}

impl EditAttribute {

    pub fn get_attr_str(&self) -> String {
        let attr = &self.attr;
        let mut attr_str = quote::quote!{ #attr }.to_string();
        attr_str = (&attr_str).replace(crate::LINE_ENDING,"");
        attr_str = (&attr_str).replace(" ","");
        return attr_str;
    }
}





