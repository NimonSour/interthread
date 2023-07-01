
use quote::format_ident;


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