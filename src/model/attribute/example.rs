use crate::error;
use crate::model::Mac;
use syn::{punctuated::Punctuated,Meta,Token,Ident};
use std::path::PathBuf;
// use proc_macro2::Span;
use proc_macro_error::{abort,abort_call_site};

//-----------------------  EXAMPLE 
#[derive(Debug, Eq, PartialEq)]
pub struct ExampleAttributeArguments {

    pub path     : Option<PathBuf>,
    pub main     :            bool,
    pub expand   :        Vec<Mac>,  
    /* ADD NEW OPTION */ 
}

impl Default for ExampleAttributeArguments {

    fn default() -> Self {

        let path  = None ;
        let main             = false;
        let expand       = vec![Mac::Actor, Mac::Family] ;
        /* ADD NEW OPTION */ 

        Self { path, main, expand }
    }
}

impl ExampleAttributeArguments {


    pub fn from( nested: Punctuated::<Meta,Token![,]> ) -> Self 
    {
        let mut eaa = ExampleAttributeArguments::default();
        eaa.parse_nested(&nested);
        eaa.arguments_cross_check();
        eaa

    }

    pub fn parse_nested(&mut self, nested: &Punctuated::<Meta,Token![,]>){
        super::check_path_set(nested,None); 

        for meta in nested.into_iter(){

            //MAIN
            if meta.path().is_ident("main"){
                self.main = true;
            }
            // PATH
            else if meta.path().is_ident("path") { 
                self.path = Some(super::meta_get_path(meta));
            }

            // EXPAND
            else if meta.path().is_ident("expand") {
                if let Some(meta_list) = super::get_list( meta,None ){
                    self.expand = vec![];
                    for ident in super::get_idents(&meta_list){
                        if let Some(mac) = Self::mac_from_ident(&ident){
                            self.expand.push(mac);
                        } else {
                            abort!(ident,"expected arguments: actor | group | group_actor | family ");
                        }
                    }
                } else {
                    abort!(meta,error::EXPECT_LIST)
                }
            }
        }
    }

    pub fn arguments_cross_check(&self){

        if  self.path.is_none() {
            abort_call_site!( "Expected a 'path' argument with a path to a file.  file=\"path/to/file.rs\"" )
        }
    }

    pub fn get_path(&mut self) -> std::path::PathBuf {
        self.path.clone().unwrap()
    }

    pub fn mac_from_ident( ident: &Ident ) -> Option<Mac> {
        if *ident == crate::ACTOR           { Some(Mac::Actor) }
        else if ident == crate::FAMILY      { Some(Mac::Family) }
        else { None }
    }
}