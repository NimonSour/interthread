use crate::model::{Lib,Mac,EditAttribute,ActorAttributeArguments,attr_to_meta_list};

use crate::use_macro::UseMacro;

use proc_macro_error::{emit_error,abort_call_site};
use syn::{parse2,File,Item};
use proc_macro2::TokenStream;
use quote::quote;
use std::path::PathBuf;


pub fn get_file( path: &PathBuf ) -> File {

    if path.exists() {
        match std::fs::read_to_string(path) {
            Ok(contents) => {
                match syn::parse_file(&contents){
                    Ok(file)  => { return file; },
                    Err(_) => {
                        let msg = format!("Internal Error.'file::get_file'. Could not parse file {:?}!", path.file_name().unwrap().to_string_lossy());
                        abort_call_site!( msg );
                    },
                }
            },
            Err(_) => {
                let msg = format!("Internal Error.'file::get_file'. Could not read file!");
                abort_call_site!( msg );
            }
        }
    }
    let msg = format!("Internal Error.'file::get_file'. File {:?} does not exist!", path.file_name().unwrap().to_string_lossy());
    abort_call_site!( msg );
}


pub fn active_file_count( path: &std::path::PathBuf ) -> Result<EditAttribute,String> {

    let file = get_file(path);

    let mut use_macro_actor   = UseMacro::new(crate::ACTOR);
    let mut use_macro_family   = UseMacro::new(&crate::FAMILY);

    let mut edit_attrs = Vec::new();

    for item  in file.items {

        match item {

            syn::Item::Impl( item_impl) => {
                for attr in &item_impl.attrs.clone() {
                        // -------------
                            // let is_actor = format!(" is_actor - {}", use_macro_actor.is(attr));
                            // let is_family = format!(" is_family - {}", use_macro_family.is(attr));
                            // // let msg = prettyplease::unparse(&attr);
                            // abort_call_site!( format!("res = {is_actor}; {is_family}"));

                        // -------------- 
                    if use_macro_actor.is(attr) || use_macro_family.is(attr) {  
                        if crate::parse::nested::is_active( attr ){

                            let edit_attr = EditAttribute { 
                                path: path.clone(),
                                attr: attr.clone(),
                                attrs: item_impl.attrs.clone(),
                                remove: false,
                            };
                            edit_attrs.push(edit_attr);
                        }
                    }
                }
            },
            syn::Item::Use( item_use ) => {
                if let Some(itm_use) = use_macro_actor.update(item_use){
                    let _ = use_macro_family.update(itm_use);
                }
            },
            _ => (),
        } 
    }

    // check if is one only
    if edit_attrs.len() == 0 {
        // error no file in attrs 
        let msg = format!("Internal Error.'file::macro_file_count'. Failed to find a file active macro `group_actor` or `actor` in module {} .\nhelp: Possible the 'actor' is not defined at module scope but within a block scope (ex. function,mod,block..)",
         path.to_string_lossy() );
        return Err(msg);
    }
    else if edit_attrs.len() > 1 {
        for edit_attr in edit_attrs{
            emit_error!(edit_attr.attr, "multiple macros with an active `file` argument!");
        }

        return Err(format!("multiple macros with an active `file` argument!"));
    }
    else {
        Ok(edit_attrs.remove(0))
    }
}

pub fn expand_macros( path: &PathBuf, macs: &Vec<Mac>) -> (File, Lib){
    let mut file    = get_file(path);
    let mut libr = Lib::default();
    for mac in macs {
        let(fil,lib) = expand_macro(file,*mac);
        file = fil;
        if Lib::default()!= lib{
            libr = lib;
        }
    }
    (file,libr)
}

pub fn expand_macro( mut file: File, mac: Mac  ) -> (File, Lib) { 

    let mut lib            = Lib::default();
    let mut use_macro   = UseMacro::new(&mac.to_string());
    let mut use_example = UseMacro::new(crate::EXAMPLE);

    let mut new_items_file: Vec<Vec<Item>> = Vec::new();

    for item  in &mut file.items {
        use_example.exclude_self_macro(item);
        match item {

            Item::Impl( item_impl) => {

                if item_impl.attrs.iter().any(|x| use_macro.is(x)){

                    for attr in &item_impl.attrs.clone() {
                        if use_macro.is(attr) {

                            item_impl.attrs = use_macro.exclude(&item_impl.attrs.clone());
                            let meta_list = attr_to_meta_list(attr);

                            let mut aaa = ActorAttributeArguments::from(meta_list,mac);
                            aaa.cross_check();
                            lib =  aaa.lib.clone(); 

                            
                            // generate code
                            let (mut code_file,_) = aaa.generate_example_code(item_impl);
                            code_file.items.insert(0,Item::Impl(item_impl.clone()) );

                            new_items_file.push(code_file.items);
                        }
                    }
                } else { 
                    new_items_file.push( vec![item.clone()] ); 
                }
            },

            Item::Use(item_use) => {
                
                if let Some(item) = use_macro.update(item_use.clone()){
                    if let Some(it) = use_example.update(item){

                        new_items_file.push( vec![Item::Use(it)] );
                    }
                }
            },
            _ => { 
                new_items_file.push( vec![item.clone()] ); 
            },
        }
    } 

    let nif = new_items_file.into_iter().flatten().collect::<Vec<Item>>();
    file.items = nif;
    (file,lib)

}

pub fn main_file( mod_name: String, lib: Lib ) -> File {

    let mod_name = quote::format_ident!("{}",mod_name);

    let code = match lib {
        Lib::Std => { 
            quote!{
                mod #mod_name;
                use #mod_name::*;
                
                fn main() {

                }
            }
        },
        Lib::Tokio => {
            quote!{
                mod #mod_name;
                use #mod_name::*;

                #[tokio::main]
                async fn main() {

                }
            }
        },
        Lib::AsyncStd => { 
            quote!{
                mod #mod_name;
                use #mod_name::*;
                
                #[async_std::main]
                async fn main() {

                }
            }
        },
        Lib::Smol => {
            quote!{
                mod #mod_name;
                use #mod_name::*;
    
                fn main() {
                    smol::block_on(async {

                    });
                }
            }
        },
    };  

    code_to_file( code )

}


pub fn code_to_file(code: TokenStream ) -> File {
    match  parse2::<File>(code ){
        Ok( file ) => {return file},
        Err(e) => { abort_call_site!( e ); },
    }
}




