use crate::error;
use crate::file::get_ident;
use crate::model::{
    argument::{Channel,Lib,Edit,Debut},
    attribute::{get_lit,get_list,to_usize},
};

use std::path::PathBuf;
use proc_macro2::Span;
use proc_macro_error::abort;
use quote::format_ident;
use syn::punctuated::Punctuated;


// GROUP ARGUMENTS 
pub struct  AGEdit {
    pub script:( bool, Option<Vec<syn::Ident>>, Option<Vec<syn::Ident>> ),
    pub live:  ( bool, Option<Vec<syn::Ident>>, Option<Vec<syn::Ident>> ),
    pub groupart: Option<Vec< (syn::Ident, Edit)>>,
}

impl AGEdit {

    pub fn set_live_all(&mut self){
        self.live = (true,Some(Vec::new()),Some(Vec::new()));
    }

    pub fn set_script_all (&mut self){
        self.script = (true,Some(Vec::new()),Some(Vec::new()));
    }

    pub fn set_groupart_all (&mut self){
        self.groupart = Some(Vec::new());
    }
    
    pub fn is_all(&self) -> bool {
        let empty = Some(Vec::new());
        let empty_g = Some(Vec::<(syn::Ident, Edit)>::new());
        self.live.0 == true  && self.script.0 == true  &&
        self.live.1 == empty && self.script.1 == empty &&
        self.live.2 == empty && self.script.1 == empty &&
        self.groupart == empty_g
    } 

    pub fn is_none(&self) -> bool {

        self.live.0 == false && self.script.0 == false &&
        self.live.1 == None  && self.script.1 == None  &&
        self.live.2 == None  && self.script.2 == None  &&
        self.groupart == None
    }  

}

impl Default for AGEdit {

    fn default() -> Self {
        let script  = (false,None,None);
        let live    = (false,None,None);
        let groupart = None;
        Self { script, live, groupart }
    } 
}

pub struct GroupAttributeArguments {

    pub name    :  Option<syn::Ident>,
    pub lib     :  Lib,
    pub assoc   :  bool,
    pub channel :  Channel,
    pub file    :  Option<std::path::PathBuf>,
 
}

impl GroupAttributeArguments {

    pub fn parse_nested(&mut self, nested: Punctuated::<syn::Meta,syn::Token![,]>) {
        for meta in nested.iter(){

            if let Some(ident) = get_ident(meta) {

                // NAME
                if meta.path().is_ident("name"){

                    match get_lit(meta) {
                        syn::Lit::Str(val) => {  
                            let str_name = val.value();

                            if str_name == "".to_string() {
                                abort!(&ident,"Attribute field 'name' is empty. Enter a name.") 
                            }
                            else {
                                self.name = Some(format_ident!("{}",val.value()));
                            } 
                        },
                        v => abort!(v, error::error_name_type( &ident, "str"); help=error::AVAIL_ACTOR ),
                    }
                }


                // LIB
                else if meta.path().is_ident("lib"){

                    match get_lit(meta) {
                        syn::Lit::Str(val) => {

                            self.lib = Lib::from(&val);
                        },
                        v => abort!(v, error::error_name_type( &ident, "str"); help=error::AVAIL_ACTOR ),
                    }
                }

                // ASSOC
                else if meta.path().is_ident("assoc"){

                    match meta {
                        syn::Meta::Path(_) => { self.assoc = true; },
                        _ => {
                            match get_lit(meta) {
                                syn::Lit::Bool(val) => { self.assoc = val.value(); },
                                v => abort!(v, error::error_name_type( &ident, "bool"); help=error::AVAIL_ACTOR ),
                            }
                        },
                    }
                }


                // CHANNEL
                else if meta.path().is_ident("channel"){

                    match get_lit(meta) {
                        syn::Lit::Int(val) => { 
                            let value = to_usize(&val);
                            if value > 0 {
                                self.channel = Channel::Buffer(val.clone());
                            }
                        },
                        v => abort!(v, error::error_name_type( &ident, "Int (usize)"),; help=error::AVAIL_ACTOR ),
                    }
                }

                // FILE
                else if meta.path().is_ident("file") {
                    let value = get_lit(meta);

                    match value.clone() {
                        syn::Lit::Str(val) => {

                            // the path needs to be checked first 
                            let path = std::path::PathBuf::from(val.value());

                            if path.exists() {
                                // one only check 
                                self.file = Some(path);
                            }
                            else {
                                abort!(val, format!("Path - {:?} does not exists.",val.value())); 
                            } 
                        },
                        _ => { abort!(value, error::error_name_type( &ident, "str"); help=error::AVAIL_ACTOR ) },
                    }
                }
            } else { 
                abort!(meta,"Unknown configuration option!"; help=error::AVAIL_ACTOR); 
            }
        }
    }
}

impl Default for GroupAttributeArguments {

    fn default() -> GroupAttributeArguments {

        Self { 
            name   : None,
            lib    : Lib::default(),
            assoc  : false,
            channel: Channel::default(),
            file   : None,
            // edit   : Edit::default(),
            // debut  : AADebut::default(),
            // file   : None,
            /* ADD NEW ATTRIBUTE */
        }  
    }
}

