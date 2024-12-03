
use quote::format_ident;
use syn::Ident;
use proc_macro_error::abort_call_site;

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
    let new_name = to_upper_camel_case(&name.to_string());
    format_ident!("{}",new_name)
}

// Family 
pub fn family( name: &Ident )-> Ident {
    format_ident!("{name}Family")
}
pub fn family_field_name(name: &Ident) -> Ident{
    let new_name = to_lower_snake_case(&name.to_string());
    format_ident!("{}",new_name)
}

fn to_upper_camel_case(input: &str) -> String {

    let words: Vec<&str> = input.split('_').collect();
    let mut struct_name = String::new();

    for (i,word) in words.into_iter().enumerate(){
        if i== 0 && word == "" {
            struct_name.push_str("_");
            continue; 
        }
        let (first, rest) = word.split_at(1);
        struct_name.push_str(&first.to_uppercase());
        struct_name.push_str(rest);
    }
    struct_name
}

fn to_lower_snake_case(input: &str) -> String {
    let mut result = String::new();

    for (i, c) in input.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 { result.push('_');}
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

pub fn combined_ident( idents: Vec<Ident>) -> Ident {

    match idents.len() {
        0 => { abort_call_site!("Internal Error.'name::combined_ident'. Empty container not expected."); },
        1 => return idents[0].clone(),
        _ => {
            let mut combined_ident = idents[0].clone();
            for ident in idents[1..].iter() {
                combined_ident = format_ident!("{combined_ident}_{ident}");
            }
            return combined_ident;
        }
    }
}