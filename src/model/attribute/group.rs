use crate::error;
// use crate::model::{Model, Channel,Lib,EditGroup,Debut,get_ident,get_lit,get_list,to_usize};
use crate::model::{Model, Channel,Lib,EditGroup,Debut,get_ident,get_ident_group,get_lit,get_lit_str,get_list,to_usize};


use std::path::PathBuf;
use proc_macro2::Span;
use proc_macro_error::abort;
use quote::format_ident;
use syn::{Ident,punctuated::Punctuated};


// GROUP ARGUMENTS 
// pub struct  AGEdit {
//     pub script:( bool, Option<Vec<syn::Ident>>, Option<Vec<syn::Ident>> ),
//     pub live:  ( bool, Option<Vec<syn::Ident>>, Option<Vec<syn::Ident>> ),
//     pub groupart: Option<Vec< (syn::Ident, Edit)>>,
// }

// impl AGEdit {

//     pub fn set_live_all(&mut self){
//         self.live = (true,Some(Vec::new()),Some(Vec::new()));
//     }

//     pub fn set_script_all (&mut self){
//         self.script = (true,Some(Vec::new()),Some(Vec::new()));
//     }

//     pub fn set_groupart_all (&mut self){
//         self.groupart = Some(Vec::new());
//     }
    
//     pub fn is_all(&self) -> bool {
//         let empty = Some(Vec::new());
//         let empty_g = Some(Vec::<(syn::Ident, Edit)>::new());
//         self.live.0 == true  && self.script.0 == true  &&
//         self.live.1 == empty && self.script.1 == empty &&
//         self.live.2 == empty && self.script.1 == empty &&
//         self.groupart == empty_g
//     } 

//     pub fn is_none(&self) -> bool {

//         self.live.0 == false && self.script.0 == false &&
//         self.live.1 == None  && self.script.1 == None  &&
//         self.live.2 == None  && self.script.2 == None  &&
//         self.groupart == None
//     }  

// }

// impl Default for AGEdit {

//     fn default() -> Self {
//         let script  = (false,None,None);
//         let live    = (false,None,None);
//         let groupart = None;
//         Self { script, live, groupart }
//     } 
// }

/*
    There are two types of arguments 
    
    channel
    lib
    file
    debut

    name(..)
    assoc(..)
    edit(..)
    path(..)

*/


#[derive(Clone)]
pub struct GroupAttributeArguments {

    pub channel :  Channel,
    pub lib     :  Lib,
    pub file    :  Option<PathBuf>,

    pub name    :  Vec<(Ident,String)>,
    pub assoc   :  Option<Vec<(Ident,bool)>>,
    pub edit    :  EditGroup,
    pub path    :  Vec<(Ident,PathBuf)>,
 
}

impl GroupAttributeArguments {


    pub fn parse_nested(&mut self, nested: Punctuated::<syn::Meta,syn::Token![,]>) {
        super::check_path_set(&nested);
        for meta in nested.iter(){

            // CHANNEL
            if meta.path().is_ident("channel"){

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

            // LIB
            else if meta.path().is_ident("lib"){
                let lib_str = get_lit_str(&meta,"lib");
                self.lib = Lib::from(&lib_str);
            }
            
            // FILE
            else if meta.path().is_ident(crate::FILE) {
                let file_str = get_lit_str(&meta,crate::FILE);

                let path = std::path::PathBuf::from(&file_str);

                if path.exists() { self.file = Some(path); }
                else { abort!(meta, format!("Path - {file_str:?} does not exist.")); } 
            }
            
            // NAME
            else if meta.path().is_ident("name"){

                if let Some(meta_list) = get_list( meta,Some(error::AVAIL_GROUP) ) { 
                    super::check_path_set(&meta_list);
                    for met in meta_list {

                        let ident = get_ident_group(&met,"name");
                        let name_str = get_lit_str(&met,"name");
                        self.name.push((ident,name_str));

                    }

                } else { abort!(meta,error::EXPECT_LIST;help=error::AVAIL_GROUP); }
            }

            // ASSOC
            else if meta.path().is_ident("assoc"){

                if let Some(meta_list) = get_list( meta,Some(error::AVAIL_GROUP) ) { 
                    for met in meta_list.iter() {
                        let ident = get_ident_group(met,"assoc");
                        match met {
                            
                            syn::Meta::Path(_) => { 
                                if self.assoc.is_some() {
                                    self.assoc.as_mut().map(|x| x.push((ident,true)));
            
                                } else { self.assoc = Some(vec![(ident,true)]); }
                            },

                            _ => { abort!(meta.path(), error::OLD_ARG_ASSOC); },
                        }
                    }

                } else { self.assoc = Some( Vec::new());  }
                
            }

            // EDIT
            else if meta.path().is_ident(crate::EDIT) { 
                self.edit.parse(&meta);
            }

            // PATH 
            else if meta.path().is_ident("path") { 
                if let Some(meta_list) = get_list( meta,Some(error::AVAIL_GROUP) ) { 
                    super::check_path_set(&meta_list); 
                    for met in meta_list {

                        let ident = get_ident_group(&met,"path");
                        let path_str = get_lit_str(&met,"path");
                        let path = std::path::PathBuf::from(&path_str);

                        if path.exists() { self.path.push((ident,path)); }

                        else { abort!(met, format!("Path - {path_str:?} does not exist.")); } 
                    }

                } else { abort!(meta,error::EXPECT_LIST;help=error::AVAIL_GROUP); }
            }

            else if meta.path().is_ident("field") { 
                if let Some(meta_list) = get_list( meta,Some(error::AVAIL_GROUP) ) { 
                    super::check_path_set(&meta_list);
                    for met in meta_list {
                        let ident = get_ident_group(&met,"field");

                        // let ident = get_ident_group(&met,"path");
                        // let path_str = get_lit_str(&met,"path");
                        // let path = std::path::PathBuf::from(&path_str);

                        // if path.exists() { self.path.push((ident,path)); }

                        // else { abort!(met, format!("Path - {path_str:?} does not exist.")); } 
                    }

                } else { abort!(meta,error::EXPECT_LIST;help=error::AVAIL_GROUP); }
            }

            // UNKNOWN ARGUMENT
            else { error::unknown_attr_arg("group",meta.path() ) }
        }
    }


}

impl Default for GroupAttributeArguments {

    fn default() -> GroupAttributeArguments {

        Self {

            channel :  Channel::default(),
            lib     :  Lib::default(),
            file    :  None,
        
            name    :  Vec::new(),
            assoc   :  None,
            edit    :  EditGroup::default(),
            path    :  Vec::new(),
        }
    }
}

