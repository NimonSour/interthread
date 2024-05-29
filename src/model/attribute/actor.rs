use crate::error;
use crate::model::{FilterSet,Channel,Lib,EditActor,Debut,get_ident,get_lit,get_lit_str,get_list,to_usize};


use std::path::PathBuf;
use proc_macro2::Span;
use proc_macro_error::abort;
use quote::format_ident;
use syn::{Ident,punctuated::Punctuated};

//-----------------------  ACTOR  

#[derive(Debug,Clone)]
pub struct ActorAttributeArguments {

    pub name    :  Option<syn::Ident>,
    pub lib     :  Lib,
    pub show   :  bool,
    pub channel :  Channel,
    pub edit    :  EditActor,
    pub debut   :  Debut,
    pub file    :  Option<PathBuf>,
    pub path    :  Option<PathBuf>,
    pub interact:  bool,
    pub filter  :  Option<FilterSet>,

    /* ADD NEW OPTION */
}


impl Default for ActorAttributeArguments {

    fn default() -> ActorAttributeArguments {

        Self { 
            name    : None,
            lib     : Lib::default(),
            show    : false,
            channel : Channel::default(),
            edit    : EditActor::default(),
            debut   : Debut::default(),
            file    : None,
            path    : None,
            interact: false,
            filter  : None,
            /* ADD NEW ATTRIBUTE */
        }  
    }
}

impl ActorAttributeArguments {
       
    pub fn parse_nested(&mut self, nested: Punctuated::<syn::Meta,syn::Token![,]>) {

        // check if unique options
        super::check_path_set(&nested); 

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

            // SHOW
            else if meta.path().is_ident("show"){

                match meta {
                    syn::Meta::Path(_) => { self.show = true; },
                    _ => { abort!(meta.path(),error::EXPECTED_IDENTIFIER_SHOW )},
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
            else if meta.path().is_ident("debut"){

                if let Some(meta_list) = get_list( meta,Some(error::AVAIL_DEBUT) ) {

                    for m in meta_list {
                        if m.path().is_ident("legend"){
                            match m {
                                syn::Meta::Path(_) => { self.debut.legend = Some(true); },
                                _ => { abort!(meta.path(),"Expected an identifier.";help=error::AVAIL_DEBUT); },
                            } 
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
            // INTERACT
            else if meta.path().is_ident("interact"){
                match meta {
                    syn::Meta::Path(_) => { self.interact = true; },
                    _ => { abort!(meta, error::EXPECT_IDENT ;help=error::AVAIL_ACTOR) },
                }
            }

            // INCLUDE
            else if meta.path().is_ident("include"){

                if let Some(filter) = &self.filter{
                    match filter {
                        FilterSet::Include(_) => abort!(ident,error::double_decl("include")),
                        FilterSet::Exclude(_) => abort!(ident,error::FILTER_CONURENT_USE_OF_OPTIONS; help=error::FILTER_OPTION_USE_HELP),
                    }
                }
                self.filter = Some(FilterSet::parse(&meta,true));
            }

            // EXCLUDE
            else if meta.path().is_ident("exclude"){

                if let Some(filter) = &self.filter{
                    match filter {
                        FilterSet::Include(_) => abort!(ident,error::FILTER_CONURENT_USE_OF_OPTIONS; help=error::FILTER_OPTION_USE_HELP), 
                        FilterSet::Exclude(_) => abort!(ident,error::double_decl("exclude")),
                    }
                }
                self.filter = Some(FilterSet::parse(&meta,false));

            }

            else if meta.path().is_ident("debug") {
                abort!(meta,"Did you mean `debut`?"; help=error::AVAIL_ACTOR);
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
    }

    pub fn get_inter_field_names(&self) -> Vec<Ident> {
        let mut loc = vec![format_ident!("sender")];
        if self.debut.active() {
            loc.push(format_ident!("debut"));
            loc.push(format_ident!("name"));
        }
        loc
    }   

} 


