use crate::attribute::{self,AALib,AAExpand,ActorAttributeArguments,GroupAttributeArguments};
use crate::use_macro::UseMacro;
use crate::gen_actor;
use crate::gen_group;

use proc_macro_error::abort;
use proc_macro2::Span;
use syn::{Attribute,Meta,Token,punctuated::Punctuated};
// use syn::punctuated::Punctuated;


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
                        let msg = format!("Internal Error.'file::get_file'. Could not parse file {:?}!", path.file_name().unwrap().to_string_lossy());
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


pub fn to_nested(attr: &Attribute) -> Punctuated::<Meta,Token![,]>{
    match attr.parse_args_with(Punctuated::<Meta,Token![,]>::parse_terminated){
        Ok(punct) => punct.clone(),
        Err(e) =>{
            let msg = format!("Internal Error.'file::to_nested'. Could not parse the attr. Error: {}",e.to_string());
            abort!(attr,msg );
        } 
    }
}
pub fn get_ident( meta: &syn::Meta ) -> Option<syn::Ident>{
    meta.path().get_ident().map(|x| x.clone())
}

pub fn get_idents( nested: &Punctuated::<Meta,Token![,]> ) -> Vec<syn::Ident> {
    nested.into_iter().filter_map(|m|{
        get_ident(m)
    }).collect::<Vec<_>>()
}


pub fn macro_file_count( path: &std::path::PathBuf ) -> Result<(Attribute, Vec<Attribute>),String> {

    let file = get_file(path);

    let mut use_macro_actor   = UseMacro::new(crate::ACTOR);
    let mut use_macro_group   = UseMacro::new(crate::GROUP);

    let mut loc = Vec::new();
    let file_ident = quote::format_ident!("file");

    let check_file_arg=|nested: &Punctuated::<Meta,Token![,]>| -> bool{
        
        let idents = get_idents(nested);
        if idents.iter().any(|x| file_ident.eq(x)) {
            return true;
        }
        false
    };


    for item  in file.items {

        match item {

            syn::Item::Impl( item_impl) => {
                for attr in &item_impl.attrs.clone() {

                    if use_macro_actor.is(attr){
                        let nested = to_nested(attr);
                        if check_file_arg(&nested){
                            loc.push((attr.clone(),item_impl.attrs.clone()));
                        }
                    }

                    else if use_macro_actor.is(attr){
                        let first_nested = to_nested(attr);
                        let second_nested = 
                        first_nested.iter().filter_map(|m| crate::attribute::get_list(m,None));
                        for nested in second_nested {
                            if check_file_arg(&nested){
                                loc.push((attr.clone(),item_impl.attrs.clone()));
                                break;
                            }
                        }
                    } 
                }
            },

            // syn::Item::Trait( item_trait) => {
            //     for attr in &item_trait.attrs.clone() {
            //         if use_macro_actor.is(attr){
            //             if check_file_arg(attr){
            //                 loc.push((attr.clone(),item_trait.attrs.clone()));
            //             }
            //         }
            //     }
            // },
            // syn::Item::Fn(item_fn) => {
            //     for attr in &item_fn.attrs.clone() {
            //         if use_macro_group.is(attr){
            //             if check_file_arg(attr){
            //                 loc.push((attr.clone(),item_fn.attrs.clone()));
            //             }
            //         }
            //     } 
            // },

            syn::Item::Use( item_use ) => {
                if let Some(itm_use) = use_macro_actor.update(item_use){
                    let _ = use_macro_group.update(itm_use);
                }
            },
            _ => (),
        } 
    }

    // check if is one only
    if loc.len() == 0 {
        // error no file in attrs 
        let msg = format!("Expected a macro `group` or `actor` in module {} .", path.to_string_lossy() );
        return Err(msg);
    }
    else if loc.len() > 1 {
        // more than one file-active 
        let msg = format!( "The module {} contains {} macros with an active `file` argument. 
        However, the allowed limit is ONE `file-active` macro per module.",path.to_string_lossy(), loc.len());
        return Err(msg);
    }
    else {
        Ok(loc.remove(0))
    }
}


pub fn get_aaa( attr: Attribute, mac: &AAExpand ) -> ActorAttributeArguments {
    let mut aaa = ActorAttributeArguments::default();

    if let syn::Meta::List(_) = attr.meta{
        let nested = 
        attr.parse_args_with(Punctuated::<Meta,Token![,]>::parse_terminated).unwrap();
        aaa.parse_nested(nested);
    }
    aaa
}

pub fn get_gaa( attr: Attribute, mac: &AAExpand ) -> GroupAttributeArguments {
    let mut gaa = GroupAttributeArguments::default();

    if let syn::Meta::List(_) = attr.meta{
        let nested = 
        attr.parse_args_with(Punctuated::<Meta,Token![,]>::parse_terminated).unwrap();
        gaa.parse_nested(nested);
    }
    gaa
}

pub fn expand_macro( mut file: syn::File, mac: &AAExpand  ) -> (syn::File, AALib){ 
    let mut lib            = AALib::default();
    let mut use_macro   = UseMacro::new(mac.to_str());
    let mut use_example = UseMacro::new(crate::EXAMPLE);

    let mut new_items_file: Vec<Vec<syn::Item>> = Vec::new();

    'f1: for item  in &mut file.items{
        use_example.exclude_self_macro(item);
        match item {

            syn::Item::Impl( item_impl) => {

                if item_impl.attrs.iter().any(|x| use_macro.is(x)){

                    for attr in &item_impl.attrs.clone() {
                        if use_macro.is(attr){
                            // let aaa = get_aaa( attr.clone(),mac);
                            // get lib 
                            // lib = aaa.lib.clone();
                            // exclude self macro 
                            // use_macro.exclude_self_macro(item);
                            // let attrs = item_impl.attrs.clone();
                            // let new_attrs = use_macro.exclude(&attrs); 
                            // generate code
                            item_impl.attrs = use_macro.exclude(&item_impl.attrs.clone());
                            // let _ = std::mem::replace(&mut item_impl.attrs, use_macro.exclude(&(item_impl.attrs.clone()))); 


                            // generate code
                            let (code,_) = 
                                match &mac {
                                    AAExpand::Actor => { 
                                        let aaa = get_aaa( attr.clone(),mac);
                                        lib = aaa.lib.clone();
                                        gen_actor::macro_actor_generate_code( aaa, item_impl.clone()) 
                                    },
                                    AAExpand::Group => { 
                                        let gaa = get_gaa( attr.clone(),mac);
                                        lib = gaa.lib.clone();
                                        gen_group::macro_group_generate_code( gaa, item_impl.clone()) },
                                };


                            let f = code_to_file(code);
                            new_items_file.push(f.items);
                            continue 'f1; 
                        }
                    }
                } else { 
                    new_items_file.push( vec![item.clone()] ); 
                }
            },

            // syn::Item::Trait(item_trait)  => {
            //     if item_trait.attrs.iter().any(|x| use_macro.is(x)){

            //         for attr in &item_trait.attrs.clone() {
            //             if use_macro.is(attr){
                           
            //                 let aaa = get_aaa( attr.clone());
            //                 //get lib 
            //                 lib = aaa.lib.clone();
            //                 //exclude self macro 
            //                 use_macro.exclude_self_macro(item);
            //                 //generate code
            //                 let (code,_) = 
            //                 actor_gen::actor_macro_generate_code( aaa, item.clone(), &mac );

            //                 let f = code_to_file(code);
            //                 new_items_file.push(f.items);
            //                 continue 'f1; 

            //             }
            //         }
            //     } else { 
            //         new_items_file.push( vec![item.clone()] ); 
            //     }
            // },

            // syn::Item::Fn( item_fn) => {
            //     if item_fn.attrs.iter().any(|x| use_macro.is(x)){

            //         for attr in &item_fn.attrs.clone() {
            //             if use_macro.is(attr){

            //                 todo!("Group not implemented yet");
            //             }
            //         }
            //     } else { 
            //         new_items_file.push( vec![item.clone()] ); 
            //     }
            // },

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





