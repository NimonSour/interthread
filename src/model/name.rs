
use quote::format_ident;
use syn::{Type,Ident};
use proc_macro::Span;
use proc_macro_error::abort;

use crate::error;

pub fn get_ident_type_generics(item_impl: &syn::ItemImpl) -> (syn::Ident,syn::Type,syn::Generics) {

    match &*item_impl.self_ty {
        syn::Type::Path(tp) => {
            let ident = tp.path.segments.last().unwrap().ident.clone();
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

pub fn check_name_conflict( names: Vec<&Ident> ){

    let mut names = 
    names.iter().map(|&x| (x.clone(), script_field(x))).collect::<Vec<_>>();

    while let Some((o_name,m_name)) = names.pop(){
        if let Some(pos) =  names.iter().position(|(_,x)| m_name.eq(x)){
            let msg = crate::error::type_naming_conflict(&o_name,&names[pos].0);
            abort!(Span::call_site(), msg;help=error::HELP_TYPE_NAMING_CONFLICT )
        }
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

pub fn gen_temp_inter( ident: &Ident) -> Ident {
    quote::format_ident!("inter{ident}")
}

pub fn gen_add_field( field: &Ident, ident: &Ident,) -> Ident {
    let field_str = &fn_to_struct( &field.to_string());
    quote::format_ident!("{field_str}{ident}")
}