use crate::attribute::{AALib,AAExpand,ActorAttributeArguments};
use crate::use_macro::UseMacro;
use crate::actor_gen;

use proc_macro_error::abort;
use proc_macro2::Span;

pub fn code_to_file(code: proc_macro2::TokenStream ) -> syn::File {
    match  syn::parse::<syn::File>(code.into() ){
        Ok( file ) => {return file},
        Err(e) => {
            abort!( Span::call_site(), e );
        },
    }
}

pub fn get_file( path: &std::path::PathBuf ) -> syn::File {

    if path.exists() {
        match std::fs::read_to_string(path) {
            Ok(contents) => {
                match syn::parse_file(&contents){
                    Ok(file)  => { return file; },
                    Err(_) => {
                        let msg = format!("Internal Error. 'file::get_file'. Could not parse file {:?}!", path.file_name().unwrap().to_string_lossy());
                        abort!( Span::call_site(),msg );
                    },
                }
            },
            Err(_) => {
                let msg = format!("Internal Error.'file::get_file'. Could not read file!");
                abort!( Span::call_site(),msg );
            }
        }
    }
    let msg = format!("Internal Error.'file::get_file'. File {:?} does not exist!", path.file_name().unwrap().to_string_lossy());
    abort!( Span::call_site(),msg );
}

pub fn expand_macros( path: &std::path::PathBuf, macs: &Vec<AAExpand>) -> (syn::File, AALib){
    let mut file    = get_file(path);
    let mut libr = AALib::default();
    for mac in macs {
        let(fil,lib) = expand_macro(file,mac);
        file = fil;
        if AALib::default()!= lib{
            libr = lib;
        }
    }
    (file,libr)
}

pub fn expand_macro( mut file: syn::File, mac: &AAExpand  ) -> (syn::File, AALib){ 
    let mut lib            = AALib::default();
    let mut use_macro   = UseMacro::new(mac.to_str());
    let mut use_example = UseMacro::new(crate::EXAMPLE);

    let mut new_items_file: Vec<Vec<syn::Item>> = Vec::new();

    'f1: for item  in &mut file.items{
        use_example.exclude_self_macro(item);
        match item {

            syn::Item::Impl( impl_block) => {

                if impl_block.attrs.iter().any(|x| use_macro.is(x)){

                    for attr in &impl_block.attrs.clone() {
                        if use_macro.is(attr){
                           
                            let mut aaa = ActorAttributeArguments::default();
                            let _ = attr.clone().parse_nested_meta(|meta| aaa.parse(meta));

                            //get lib 
                            lib = aaa.lib.clone();
                            //exclude self macro 
                            use_macro.exclude_self_macro(item);
                            //generate code
                            let (code,_) = 
                            actor_gen::actor_macro_generate_code( aaa, item.clone(), mac.clone() );

                            let f = code_to_file(code);
                            new_items_file.push(f.items);
                            continue 'f1; 
                        }
                    }
                } else { 
                    new_items_file.push( vec![item.clone()] ); 
                }
            },

            syn::Item::Trait(item_trait)  => {
                if item_trait.attrs.iter().any(|x| use_macro.is(x)){

                    for attr in &item_trait.attrs.clone() {
                        if use_macro.is(attr){
                           
                            let mut aaa = ActorAttributeArguments::default();
                            let _ = attr.clone().parse_nested_meta(|meta| aaa.parse(meta));

                            //get lib 
                            lib = aaa.lib.clone();
                            //exclude self macro 
                            use_macro.exclude_self_macro(item);
                            //generate code
                            let (code,_) = 
                            actor_gen::actor_macro_generate_code( aaa, item.clone(), mac.clone() );

                            let f = code_to_file(code);
                            new_items_file.push(f.items);
                            continue; 

                        }
                    }
                } else { 
                    new_items_file.push( vec![item.clone()] ); 
                }
            },

            syn::Item::Fn( item_fn) => {
                if item_fn.attrs.iter().any(|x| use_macro.is(x)){

                    for attr in &item_fn.attrs.clone() {
                        if use_macro.is(attr){

                            todo!("Group not implemented yet");
                        }
                    }
                } else { 
                    new_items_file.push( vec![item.clone()] ); 
                }
            },

            syn::Item::Use(item_use) => {
                
                if let Some(item) = use_macro.update(item_use.clone()){
                    if let Some(it) = use_example.update(item){

                        new_items_file.push( vec![syn::Item::Use(it)] );
                    }
                }
            },

            _ => { 
                new_items_file.push( vec![item.clone()] ); 
            },
        }
    } 

    let nif = new_items_file.into_iter().flatten().collect::<Vec<syn::Item>>();
    file.items = nif;
    (file,lib)

}

pub fn main_file( mod_name: String, lib: AALib ) -> syn::File {

    let mod_name = quote::format_ident!("{}",mod_name);

    let code = match lib {
        AALib::Std => { 
            quote::quote!{
                mod #mod_name;
                
                fn main() {

                }
            }
        },
        AALib::Tokio => {
            quote::quote!{
                mod #mod_name;

                #[tokio::main]
                async fn main() {

                }
            }
        },
        AALib::AsyncStd => { 
            quote::quote!{
                mod #mod_name;
                
                #[async_std::main]
                async fn main() {

                }
            }
        },
        AALib::Smol => {
            quote::quote!{
                mod #mod_name;
    
                fn main() {
                    smol::block_on(async {

                    });
                }
            }
        },
    };  

    code_to_file( code )

}





