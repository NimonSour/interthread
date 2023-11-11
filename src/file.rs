use crate::model::{Lib,Model,Edit,EditAttribute,AttributeArguments,attr_to_meta_list};

use crate::use_macro::UseMacro;

use proc_macro_error::abort;
use proc_macro2::Span;
use syn::{Ident,ItemStruct,ItemImpl,Attribute,Meta,Token,punctuated::Punctuated};



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

pub fn get_some_edit_attribute( attr: &Attribute,
                           item_impl: &syn::ItemImpl,
                                path: &std::path::PathBuf, 
                                model: Model ) -> Option<EditAttribute> {

    let file_ident = quote::format_ident!("{}",crate::FILE);
    let edit_ident = quote::format_ident!("{}",crate::EDIT);

    let check_file_arg=|nested: &Punctuated::<Meta,Token![,]>| -> bool{
        
        let idents = crate::model::attribute::get_idents(nested);
        let file_bol = idents.iter().any(|x| file_ident.eq(x));
        let edit_bol = idents.iter().any(|x| edit_ident.eq(x));
        file_bol && edit_bol
    };

    let nested = attr_to_meta_list(attr);
    if check_file_arg(&nested){
        for meta in nested {
            if meta.path().is_ident(crate::EDIT){

                let mut e = Edit::new(&model);
                e.parse(&meta);
                if e.is_any_active() {
                    return Some(EditAttribute { 
                        path: path.clone(),
                        attr: attr.clone(),
                        attrs: item_impl.attrs.clone(),
                        remove: e.get_remove(),
                        idents: e.get_some_ident_list(),
                    });
                }
            }
        }
    }
    None
}


pub fn active_file_count( path: &std::path::PathBuf ) -> Result<EditAttribute,String> {

    let file = get_file(path);

    let mut use_macro_actor   = UseMacro::new(crate::ACTOR);
    let mut use_macro_group   = UseMacro::new(crate::GROUP);

    let mut loc = Vec::new();

    for item  in file.items {

        match item {

            syn::Item::Impl( item_impl) => {
                for attr in &item_impl.attrs.clone() {

                    if use_macro_actor.is(attr){
                        if let Some( edit_attr) = 
                        get_some_edit_attribute(attr,&item_impl,path,Model::Actor){
                            loc.push(edit_attr);
                        }
                    }
                    else if use_macro_group.is(attr){
                        if let Some( edit_attr) = 
                        get_some_edit_attribute(attr,&item_impl,path,Model::Group){
                            loc.push(edit_attr);
                        }
                    }
                }
            },
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
        let msg = format!("Internal Error.'file::macro_file_count'. Failed to find a file active macro `group` or `actor` in module {} .",
         path.to_string_lossy() );
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


pub fn find_group_items( path: &std::path::PathBuf, ident: &Ident ) -> (ItemStruct,ItemImpl){//Result<(ItemStruct,ItemImpl)> {

    let file = get_file(path);

    let mut item_struct = None;
    let mut item_impl    = None;


    for item  in file.items {

        match item {
            syn::Item::Struct(i_struct) => {
                if  i_struct.ident.eq(ident) {
                    if item_struct.is_none() {
                        item_struct = Some(i_struct.clone());
                    }
                }
            },
            syn::Item::Impl(i_impl) => {
                let (name,_,_) = crate::model::get_ident_type_generics(&i_impl);
                if name.eq(ident){
                    item_impl = Some(i_impl);
                    if item_struct.is_some(){
                        break;
                    } 
                }

            },
            _ => (),
        } 
    }
    
    let error_msg = |s:&str|{
        format!("Type `{ident}` {s} not foud in `{}`.",path.to_string_lossy()) 
    };
    match (item_struct,item_impl){

        (Some(strct),Some(imp)) => {return (strct,imp);},
        (Some(_),None) => {
            abort!(Span::call_site(),error_msg("implement block"));
        },
        (None,Some(_)) => {
            abort!(Span::call_site(),error_msg("definition block"));
        },
        (None,None) => {
            abort!(Span::call_site(),error_msg(""));
        },
    }

}

pub fn expand_macros( path: &std::path::PathBuf, macs: &Vec<Model>) -> (syn::File, Lib){
    let mut file    = get_file(path);
    let mut libr = Lib::default();
    for mac in macs {
        let(fil,lib) = expand_macro(file,mac);
        file = fil;
        if Lib::default()!= lib{
            libr = lib;
        }
    }
    (file,libr)
}

pub fn expand_macro( mut file: syn::File, mac: &Model  ) -> (syn::File, Lib) { 

    let mut lib            = Lib::default();
    let mut use_macro   = UseMacro::new(&mac.to_string());
    let mut use_example = UseMacro::new(crate::EXAMPLE);

    let mut new_items_file: Vec<Vec<syn::Item>> = Vec::new();

    for item  in &mut file.items {
        use_example.exclude_self_macro(item);
        match item {

            syn::Item::Impl( item_impl) => {

                if item_impl.attrs.iter().any(|x| use_macro.is(x)){

                    for attr in &item_impl.attrs.clone() {
                        if use_macro.is(attr) {

                            item_impl.attrs = use_macro.exclude(&item_impl.attrs.clone());
                            let meta_list = attr_to_meta_list(attr);
                            let mut aa = AttributeArguments::from(meta_list,mac);
                            aa.cross_check(item_impl);
                            lib = aa.get_lib();
                            // generate code
                            let (code,_) = aa.generate_code(&item_impl);

                            let f = code_to_file(code);
                            new_items_file.push(f.items);
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

pub fn main_file( mod_name: String, lib: Lib ) -> syn::File {

    let mod_name = quote::format_ident!("{}",mod_name);

    let code = match lib {
        Lib::Std => { 
            quote::quote!{
                mod #mod_name;
                
                fn main() {

                }
            }
        },
        Lib::Tokio => {
            quote::quote!{
                mod #mod_name;

                #[tokio::main]
                async fn main() {

                }
            }
        },
        Lib::AsyncStd => { 
            quote::quote!{
                mod #mod_name;
                
                #[async_std::main]
                async fn main() {

                }
            }
        },
        Lib::Smol => {
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





