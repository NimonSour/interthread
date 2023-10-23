
use quote::format_ident;
use syn::{Type,Ident};
use proc_macro_error::abort;

// use crate::attribute::AAExpand;
use crate::model::argument::Model;

pub fn get_ident_type_generics(item_impl: &syn::ItemImpl) -> (syn::Ident,syn::Type,syn::Generics) {

    match &*item_impl.self_ty {
        syn::Type::Path(tp) => {
            let ident = tp.path.segments.last().unwrap().ident.clone();
            // let typ :syn::Type = syn::parse_quote!{ #ident };
            let generics = item_impl.generics.clone();
            (ident,Type::Path(tp.clone()),generics)
        },
        _ => {
            let msg ="Internal Error.'actor_gen::impl_get_name'. Could not get item Impl's name!";
            abort!(item_impl,msg);
        }
    }
}

// Actor
pub fn script(name: &Ident) -> Ident{
    let new_name = name.to_string() + "Script";
    format_ident!("{}",new_name)
}

pub fn live(name: &Ident) -> Ident{
    let new_name = name.to_string() + "Live";
    format_ident!("{}",new_name)
}

pub fn script_field(name: &Ident) -> Ident{
    let new_name = fn_to_struct(&name.to_string());
    format_ident!("{}",new_name)
}

// ActorGroup
pub fn group_script(name: &Ident) -> Ident{
    let new_name = name.to_string() + "GroupScript";
    format_ident!("{}",new_name)
}

pub fn group_live(name: &Ident) -> Ident{
    let new_name = name.to_string() + "GroupLive";
    format_ident!("{}",new_name)
}

// GroupActor
pub fn script_group(name: &Ident) -> Ident{
    let new_name = name.to_string() + "ScriptGroup";
    format_ident!("{}",new_name)
}

pub fn live_group(name: &Ident) -> Ident{
    let new_name = name.to_string() + "LiveGroup";
    format_ident!("{}",new_name)
}

pub fn get_group_names(name: &Ident) ->  ( Ident, Ident ){
    ( group_script(name), group_live(name) )
}

pub fn get_actor_names(name: &Ident, mac: &Model) -> ( Ident, Ident ){

    match mac {
        Model::Actor => ( script(name), live(name) ),
        Model::Group => ( script_group(name), live_group(name) ),
    }
}

pub fn get_names(name: &Ident, mac: Model, model: &Model) -> ( Ident, Ident ){

    match mac {
        Model::Actor => ( script(name), live(name) ),
        Model::Group => ( script_group(name), live_group(name) ),
    }
}





fn fn_to_struct(input: &str) -> String {

    let words: Vec<&str> = input.split('_').collect();
    let mut struct_name = String::new();

    for word in words {
        let (first, rest)    = word.split_at(1);
        let capitalized = first.to_uppercase();
        let word_without_first = rest.to_string();
        struct_name.push_str(&capitalized);
        struct_name.push_str(&word_without_first);
    }
    struct_name
}