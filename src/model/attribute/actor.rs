use crate::error;
// use crate::model::EditActor;
// use crate::file::get_ident;
use crate::model::{Model, Channel,Lib,EditActor,Debut,get_ident,get_lit,get_lit_str,get_list,to_usize};


use std::path::PathBuf;
use proc_macro2::Span;
use proc_macro_error::abort;
use quote::format_ident;
use syn::punctuated::Punctuated;
//-----------------------  ACTOR  

#[derive(Debug,Clone, Eq, PartialEq)]
pub struct ActorAttributeArguments {

    pub name    :  Option<syn::Ident>,
    pub lib     :  Lib,
    pub assoc   :  bool,
    pub channel :  Channel,
    pub edit    :  EditActor,
    pub debut   :  Debut,
    // pub file    :  Option<AAFile>,
    pub file    :  Option<PathBuf>,
    /* ADD NEW OPTION */
}


impl Default for ActorAttributeArguments {

    fn default() -> ActorAttributeArguments {

        Self { 
            name   : None,
            lib    : Lib::default(),
            assoc  : false,
            channel: Channel::default(),
            edit   : EditActor::default(),
            debut  : Debut::default(),
            file   : None,
            /* ADD NEW ATTRIBUTE */
        }  
    }
}

impl ActorAttributeArguments {
       
    pub fn parse_nested(&mut self, nested: Punctuated::<syn::Meta,syn::Token![,]>) {

        // check if unique options
        super::check_path_set(&nested); 
        // super::check_ident_sets(&nested); 

        for meta in nested.iter(){

            let ident = get_ident(meta);
            // NAME
            if meta.path().is_ident("name"){

                let str_name = get_lit_str(&meta,"name"); 
                
                if &str_name == "" {
                    abort!(&ident,"Attribute argument 'name' value is empty. Enter a name.") 
                }

                else { self.name = Some(format_ident!("{str_name}")); }
            }


            // LIB
            else if meta.path().is_ident("lib"){

                let lib_str = get_lit_str(&meta,"lib");
                self.lib = Lib::from(&lib_str);

            }

            // ASSOC
            else if meta.path().is_ident("assoc"){

                match meta {
                    syn::Meta::Path(_) => { self.assoc = true; },
                    _ => { abort!(meta.path(), error::OLD_ARG_ASSOC); },
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
                    v => abort!(v, error::error_name_type( &meta.path(), "Int (usize)"),; help=error::AVAIL_ACTOR ),
                }
            }


            // EDIT
            else if meta.path().is_ident(crate::EDIT){
                self.edit.parse(&meta);
            }

            // DEBUT
            // pub fn check_legend_path( model: &Model, name: &syn::Ident, path: &PathBuf ) -> (PathBuf, PathBuf) {
            else if meta.path().is_ident("debut"){

                if let Some(meta_list) = get_list( meta,Some(error::AVAIL_DEBUT) ) {

                    for m in meta_list {
                        if m.path().is_ident("legend"){
                            if let Some(meta_list) = get_list( meta,Some(error::AVAIL_DEBUT) ) {
                                super::check_path_set(&nested);
                                for m in meta_list{

                                    if m.path().is_ident("path"){

                                        let path_str = get_lit_str(&meta,"path");
                                        let path = std::path::PathBuf::from(&path_str);

                                        if path.exists() { 
                                            self.debut.path = Some(path);
                                        }

                                    } else {
                                        let msg = "Unknown option for argument 'debut'.";
                                        abort!(m,msg;help=error::AVAIL_DEBUT);
                                    }
                                }
                            } else { self.debut.legend = Some(true); }  
                        } else {
                            let msg = "Unknown option for argument 'debut'.";
                            abort!(m,msg;help=error::AVAIL_DEBUT);
                        }
                    }
                } else {  self.debut.legend = Some(false);  }
            }

            // FILE
            else if meta.path().is_ident(crate::FILE) {

                let file_str = get_lit_str(&meta,crate::FILE);
                let path = std::path::PathBuf::from(&file_str);

                if path.exists() { self.file = Some(path); }
                else { abort!(meta, format!("Path - {file_str:?} does not exist.")); } 

            }

            else if meta.path().is_ident("id"){ 
                abort!(ident, error::OLD_ARG_ID);
            }

            // UNKNOWN ARGUMENT
            else { error::unknown_attr_arg("actor",meta.path() ) }
        }
    }


    pub fn cross_check(&mut self){
        // file count 
        if self.edit.is_any_active(){
            if let Some(file_path) = &self.file {
                match crate::file::active_file_count(file_path) {
                    Ok(edit_attr) => {
                        self.edit.attr = Some(edit_attr);
                    },
                    Err(e) => { abort!(Span::call_site(),e); },
                }
            } else { abort!(Span::call_site(),error::REQ_FILE;help=error::AVAIL_ACTOR); }
        }

        // debut paths check 
        // get debut script live in generating function
    }
} 


