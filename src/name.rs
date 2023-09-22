
use quote::format_ident;
// use crate::attribute::AAExpand;
use proc_macro_error::abort;

pub fn get_ident_type_generics(item_impl: &syn::ItemImpl) -> (syn::Ident,syn::Type,syn::Generics) {

    // match item {
        // syn::Item::Impl(item_impl)  => {
            match &*item_impl.self_ty {
                syn::Type::Path(tp) => {
                    let ident = tp.path.segments.last().unwrap().ident.clone();
                    let typ :syn::Type = syn::parse_quote!{ #ident };
                    let generics = item_impl.generics.clone();
                    (ident,typ,generics)
                },
                _ => {
                    let msg ="Internal Error.'actor_gen::impl_get_name'. Could not get item Impl's name!";
                    abort!(item_impl,msg);
                }
            }
        // },
        // v => {
        //     let msg = "Macro `actror` expected an `impl` item.";
        //     abort!( v, msg ); 
        // },
    // }
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