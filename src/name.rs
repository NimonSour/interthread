
use quote::format_ident;
use crate::attribute::AAExpand;
use proc_macro_error::abort;

pub fn get_name_and_type(mac: AAExpand, item: &syn::Item) -> (syn::Ident,syn::Type) {

    let name ;
    let mut tp = false;

    match mac {
        AAExpand::Group => {
            match item {
                syn::Item::Fn(item_fn) => {
                    name = item_fn.sig.ident.clone();
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
                        },
                        _ => {
                            let msg ="Internal Error.'actor_gen::impl_get_name'. Could not get item Impl's name!";
                            abort!(item_impl,msg);
                        }
                    }
                },
                syn::Item::Trait(item_trait) => { 
                    name = item_trait.ident.clone();
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
    // let res = if tp { (name.clone(), syn::parse_quote!{ dyn #name }) } else { (name.clone(), syn::parse_quote!{ #name })};
    // proc_macro_error::abort!(item, "After format");
    // res
    (name,ty)
}


pub fn script(name: &syn::Ident) -> syn::Ident{
    let new_name = name.to_string() + "Script";
    format_ident!("{}",new_name)
}
pub fn script_field(name: &syn::Ident) -> syn::Ident{
    let new_name = fn_to_struct(&name.to_string());
    format_ident!("{}",new_name)
}
pub fn direct(name: &syn::Ident) -> syn::Ident{
    let new_name = struct_to_fn(&name.to_string()) + "_direct";
    format_ident!("{}",new_name)
}
pub fn play(name: &syn::Ident) -> syn::Ident{
    let new_name = struct_to_fn(&name.to_string()) + "_play";
    format_ident!("{}",new_name)
}
pub fn live(name: &syn::Ident) -> syn::Ident{
    let new_name = name.to_string() + "Live";
    format_ident!("{}",new_name)
}


fn struct_to_fn(input: &str) -> String {
    let mut snake_case = String::new();
    for (i, c) in input.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                snake_case.push('_');
            }
            snake_case.push(c.to_ascii_lowercase());
        } else {
            snake_case.push(c);
        }
    }
    snake_case
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