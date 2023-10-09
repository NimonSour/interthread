use crate::error;
// use crate::file::get_ident;
use crate::model::{
    argument::{Channel,Lib,Edit,Debut},
    attribute::{get_ident,get_lit,get_list,to_usize},
};

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
    pub edit    :  Edit,
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
            edit   : Edit::default(),
            debut  : Debut::default(),
            file   : None,
            /* ADD NEW ATTRIBUTE */
        }  
    }
}

impl ActorAttributeArguments {
       
    pub fn parse_nested(&mut self, nested: Punctuated::<syn::Meta,syn::Token![,]>) {

        // check if unique options
        super::is_set(&nested); 

        for meta in nested.iter(){

            // if let Some(ident) = get_ident(meta) {
            let ident = get_ident(meta);
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


                // EDIT
                else if meta.path().is_ident(crate::EDIT){
                    self.edit.parse(&meta);
                }

                // DEBUT

                /*
                
                #[actor( channel=0, debut(legend(path="src")) )]
                AVAIL_DEBUT
                 */
                else if meta.path().is_ident("debut"){

                    if let Some(meta_list) = get_list( meta,Some(error::AVAIL_DEBUT) ) {

                        for m in meta_list {
                            if m.path().is_ident("legend"){
                                if let Some(meta_list) = get_list( meta,Some(error::AVAIL_DEBUT) ) {
                                    for m in meta_list{

                                        if m.path().is_ident("path"){

                                            match get_lit(&m) {
                                                syn::Lit::Str(val) => {
                                                    let path_str = val.value();
                                                    todo!()
                                                },
                                                _ => { abort!(m, error::error_name_type( &ident, "bool"); help=error::AVAIL_ACTOR ) },
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


                    // match meta {
                    //     syn::Meta::Path(_) => { self.debut.legend = Some(false); },
                    //     syn::Meta::List(_)=> {
                    //         if meta_list.
                    //     },
                    //     _ => {
                    //         match get_lit(meta) {
                    //             syn::Lit::Bool(val) => { self.id = val.value(); },
                    //             v => abort!(v, error::error_name_type( &ident, "bool"); help=error::AVAIL_ACTOR ),
                    //         }
                    //     }
                    // }
                }

                // FILE
                else if meta.path().is_ident(crate::FILE) {

                    let value = get_lit(meta);

                    match value.clone() {
                        syn::Lit::Str(val) => {

                            // the path needs to be checked first 
                            let path = std::path::PathBuf::from(val.value());
                            // one only check 
                            if path.exists() { self.file = Some(path); }
                            else { abort!(val, format!("Path - {:?} does not exists.",val.value())); } 
                        },
                        _ => { abort!(value, error::error_name_type( &ident, "str"); help=error::AVAIL_ACTOR ) },
                    }
                }

                else if meta.path().is_ident("id"){ 
                    // error "id" is "debut" since v1.2.0
                    abort!(ident, error::OLD_ARG_ID);
                }


                // UNKNOWN ARGUMENT
                else { error::unknown_attr_arg("actor",&ident ) }

        }


    }


    pub fn cross_check(&mut self){


        // here  needs to check 
        // if file = path exists 
        //        true) and  is active 
        //              count active files
        //                  ok 
        //        false) and is active  -> error
    
        if self.edit.is_any_active(){
            // let msg = format!("script - {:?}, live - {:?}", &self.edit.script, &self.edit.live);
            // abort!(Span::call_site(),msg);
            if let Some(file_path) = &self.file {
                match crate::file::macro_file_count(file_path) {
                    Ok(edit_attr) => {
                        self.edit.attr = Some(edit_attr);
                    },
                    Err(e) => { abort!(Span::call_site(),e); },
                }
            } else {
                // error for using option file active but the path is not specified 
                let msg = r#"Expected a 'file' argument ` file = "path/to/current/file.rs" ` ."#; 
                abort!(Span::call_site(),msg;help=error::AVAIL_ACTOR)
            }
    
        }
        // let msg = format!("script - {:?} \n live - {:?}",self.edit.script,self.edit.live);
        // abort!(proc_macro::Span::call_site(),msg);
    }
} 


