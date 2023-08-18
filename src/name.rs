
use quote::format_ident;
use crate::attribute::AAExpand;
use proc_macro_error::abort;

pub fn get_name_and_type(mac: &AAExpand, item: &syn::Item) -> (syn::Ident,syn::Type,Option<syn::Generics>) {

    let name ;
    let generics;
    let mut tp = false;

    match mac {
        AAExpand::Group => {
            match item {
                syn::Item::Fn(item_fn) => {
                    name = item_fn.sig.ident.clone();
                    generics = None
                },
                v => {
                    let msg = "Macro `group` expected a `fn` item."; 
                    abort!( v, msg ); 
                },
            }
        },

        AAExpand::Actor => {

            match item {
                syn::Item::Impl(item_impl)  => {
                    match &*item_impl.self_ty {
                        syn::Type::Path(tp) => {
                            name = tp.path.segments.last().unwrap().ident.clone();
                            generics = Some(item_impl.generics.clone());
                        },
                        _ => {
                            let msg ="Internal Error.'actor_gen::impl_get_name'. Could not get item Impl's name!";
                            abort!(item_impl,msg);
                        }
                    }
                },
                syn::Item::Trait(item_trait) => { 
                    name = item_trait.ident.clone();
                    generics = None;
                    tp = true; 
                },
                v => {
                    let msg = "Macro `actror` expected an `impl` or `trait` item.";
                    abort!( v, msg ); 
                },
            }
        },
    }
    
    let ty: syn::Type = if tp {syn::parse_quote!{ impl #name }} else { syn::parse_quote!{ #name }} ;
    (name,ty,generics)
}


pub fn script(name: &syn::Ident) -> syn::Ident{
    let new_name = name.to_string() + "Script";
    format_ident!("{}",new_name)
}
pub fn script_field(name: &syn::Ident) -> syn::Ident{
    let new_name = fn_to_struct(&name.to_string());
    format_ident!("{}",new_name)
}

pub fn live(name: &syn::Ident) -> syn::Ident{
    let new_name = name.to_string() + "Live";
    format_ident!("{}",new_name)
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