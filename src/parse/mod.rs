pub mod icb;
pub mod atp;
pub mod nested;

pub use icb::ItemCodeBlock;
pub use atp::ActiveTextParser;

use crate::model::name::get_ident_type_generics;
use crate::model::argument::EditAttribute;
use crate::write::get_text;
use crate::LINE_ENDING;

use proc_macro_error::{abort, abort_call_site};
use proc_macro2::{TokenStream,Span};
use syn::{Attribute,Ident,ItemImpl};
use std::collections::BTreeMap;


fn set_attrs( attrs: &Vec<Attribute>, item_impl: &ItemImpl ) -> ItemImpl {
    let mut item_impl = item_impl.clone();
    item_impl.attrs = attrs.clone();
    item_impl
}


pub fn preceded_by(s: &str, pos: usize, target: &str ) -> Option<usize> {
    
    if !target.len() > pos {
        let targ_ch: Vec<char> = target.chars().rev().collect();
        let perc_ch: Vec<char> = s[..pos].chars().rev().take(targ_ch.len()).collect(); 
        if targ_ch == perc_ch {
            return Some(pos-target.len())
        } 
    }

    None
}

// pad(3,"")
fn pad(n: usize, s: &str) -> String {
    let space = " ".repeat(n);
    format!("{}{}",space,s)
} 

// parse attr 
fn parse_attr( s: &str ) -> Attribute {

    let text = format!(r#"{}fn foo (){{}}"#,s); 

    match syn::parse_str::<syn::ItemFn>(&text) {

        Ok(item_fn) => {
            if let Some(attr) = item_fn.attrs.into_iter().next(){
                return attr;
            } else {
                abort_call_site!("Internal Error.`parse::parse_attr`. Function `attrs` is empty.");
            }
        },
        Err(_) => abort_call_site!("Internal Error.`parse::parse_attr`. Could not parse the Attribute."),
    }
}


pub fn split_file( 
    edit_attr: &EditAttribute, 
    item_impl: &ItemImpl ) -> (String,String,) {

    match  get_text(&edit_attr.path){

        Ok(text) => {
            
            let mut icb = ItemCodeBlock::new(text);

            match icb.get_item_code(edit_attr.attrs.clone(),&item_impl){
                Ok(attrs) => {
                    let (prefix,suffix) = icb.src.split_at(icb.index);

                    let mut prefix  = prefix.to_string();
                    if let Some(pos) = attrs.iter().position(|x| x.1.eq(&edit_attr.attr)){

                        let index =  attrs[pos].0;
                        let s   = &attrs[pos].2;

                        if edit_attr.remove {

                            let end = index + s.len();
                            prefix.replace_range(index..=end, "");

                        } else {

                            let end = index + s.len();
                            let new_attr_str = nested::edit_remove_active_file_args(s,&prefix[index..=end], &edit_attr.idents);
                            prefix.replace_range(index..=end, &new_attr_str);

                        }
                        return (prefix,suffix.into());
                    }
                    // no position internal error
                    abort!(Span::call_site(),"Internal Error. 'parse::split_file'. No matching Attribute found in the list of Attributes.");

                },
                Err(e) => {
                    // didn't find the attribute
                    abort!(Span::call_site(),e.to_string()); 
                },
            }
        },
        Err(e) => { 
            // could not get text from file 
            abort!(Span::call_site(),e.to_string());
        },
    }
} 



pub fn edit_write(  
                   edit_attr: &EditAttribute, 
                   item_impl: &ItemImpl, 
                   edit_sdpl: BTreeMap<Ident,TokenStream> ) {

    let (name, _, _)     =  get_ident_type_generics(&item_impl);
    let edifix = create_edifix( edit_sdpl);

    let (mut prefix, suffix) = split_file( &edit_attr, item_impl );
    let attr_str = edit_attr.get_attr_str();
    
    let obj_name = format!("// Object Name   : {}  {LINE_ENDING}", name.to_string() );
    let init_by  = format!("// Initiated By  : {}  {LINE_ENDING}", attr_str );


    prefix += LINE_ENDING;
    prefix += LINE_ENDING;
    prefix += "//++++++++++++++++++[ Interthread  Write to File ]+++++++++++++++++//";
    prefix += LINE_ENDING;
    prefix += &obj_name;
    prefix += &init_by;
    prefix += LINE_ENDING;
    prefix += "/*";
    prefix += LINE_ENDING;
    prefix += &edifix;
    prefix += LINE_ENDING;
    prefix += "// *///.............[ Interthread  End of Write  ].................//";
    prefix += LINE_ENDING;
    prefix += &suffix;


    if let Err(e) = crate::write::write(prefix, &edit_attr.path){
        proc_macro_error::abort!(proc_macro2::Span::call_site(),e.to_string());
    }

}  


fn create_edifix(edit_sdpl: BTreeMap<Ident,TokenStream>) -> String {

    let mut edifix = String::new();
    let len = edit_sdpl.len();
    let pin = | ident: &Ident | {
        if len == 1 { "".to_string() } 
        else { format!("{LINE_ENDING}//---({ident})") }
    };

    for (field, edit_code ) in edit_sdpl{
        if !edit_code.is_empty(){
            if let Ok(edifile) =  syn::parse2::<syn::File>(edit_code){
                edifix += &pin(&field);
                edifix += LINE_ENDING;
                edifix += &prettyplease::unparse(&edifile);
            } else {
                let msg = "Internal Error 'parse::mod::create_edifix'. Failed to parse TokenStream to syn::File.";
                abort!(Span::call_site(),msg)
            }
        }
    }
    edifix 
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_for_attr_inside_impl() {

        let attr_text = r##"#[actor(file="abs.rs", edit(script) )]"##;
        let impl_text = r#"
        impl ActorB{
            #[doc="Comment on ActorB::new"]
            pub fn new() -> Self{ Self{} }
        }
        "#;

        let text = format!("{attr_text}{impl_text}");
        let org_attr = parse_attr(attr_text);
        let org_impl = syn::parse_str::<ItemImpl>(impl_text).unwrap();

        let mut icb = ItemCodeBlock::new(text);
        let res_attr = 
        icb.get_item_code(vec![org_attr.clone()],&org_impl).unwrap()[0].1.clone();

        assert_eq!(res_attr,org_attr);
    }

    #[test]
    fn test_func_group_edit(){
        let attr_str = r#"
#[interthread::group(
    file="path/to/abc.rs",
    edit(
        a::edit( script( def, file(imp(bla,file)), trt(file)),
        live(file(def), imp, trt ), 
     ),
        c::edit(file) , 
        b::edit(file(live), script( def, imp),
        live(file(def, imp(file), trt)), 
     ),
      
    )
)]"#;

    let a = quote::format_ident!("a");
    let b = quote::format_ident!("b");
    let c = quote::format_ident!("c");
    let new_attr_str = 
    nested::edit_remove_active_file_args(attr_str,attr_str,&Some(vec![a,b,c]));

    let expect_attr_str = r#"
#[interthread::group(
    file="path/to/abc.rs",
    edit(
        a::edit( script( def, imp(bla,file), trt(file)),
        live(def, imp, trt ), 
     ),
        c::edit , 
        b::edit(live, script( def, imp),
        live(def, imp(file), trt), 
     ),
      
    )
)]"#;

    assert_eq!(expect_attr_str,new_attr_str);
    }


    #[test]
    fn test_func_actor_edit(){
        // in 'trt' file is an ident
        let attr_str = r#"
#[interthread::actor(
    file="path/to/abc.rs",
    edit(
        script( def, file(imp), trt(file) ),
        file(live(   def, imp, trt)),
    )
)]"#;

    let new_attr_str = nested::edit_remove_active_file_args(attr_str,attr_str,&None);

    let expect_attr_str = r#"
#[interthread::actor(
    file="path/to/abc.rs",
    edit(
        script( def, imp, trt(file) ),
        live(   def, imp, trt),
    )
)]"#;

    assert_eq!(expect_attr_str,new_attr_str);
    }

    

    #[test]
    fn test_func_actor_edit_plus_comment(){
        let attr_str = r#"
#[interthread::actor(
    file="path/to/abc.rs",
    edit(
        script( def, file(imp), trt(file) ),
        file(live(   def, imp, trt)),
        //file(live(def, imp, trt)),
    )
)]"#;

    let new_attr_str = nested::edit_remove_active_file_args(attr_str,attr_str,&None);

    let expect_attr_str = r#"
#[interthread::actor(
    file="path/to/abc.rs",
    edit(
        script( def, imp, trt(file) ),
        live(   def, imp, trt),
        //file(live(def, imp, trt)),
    )
)]"#;

    assert_eq!(expect_attr_str,new_attr_str);
    }
    
}