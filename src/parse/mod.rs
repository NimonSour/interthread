pub mod icb;
pub mod atp;
pub mod nested;

pub use icb::ItemCodeBlock;
pub use atp::ActiveTextParser;

use crate::model::name::get_ident_type_generics;
use crate::model::argument::EditAttribute;
use crate::show::get_text;
use crate::LINE_ENDING;

use proc_macro_error::abort;
use proc_macro2::{TokenStream,Span};
use syn::{Attribute,Ident,Item,ItemImpl};
use std::collections::BTreeMap;


fn set_attrs( attrs: &Vec<Attribute>, item_impl: &ItemImpl ) -> ItemImpl {
    let mut item_impl = item_impl.clone();
    item_impl.attrs = attrs.clone();
    item_impl
}


pub fn preceded_by(s: &str, pos: usize, target: &str ) -> Option<usize> {
    
    if target.len() > pos {
        return None
    } else {
        let targ_ch: Vec<char> = target.chars().rev().collect();
        let perc_ch: Vec<char> = s[..pos].chars().rev().take(targ_ch.len()).collect(); 
        if targ_ch == perc_ch {
            return Some(pos-target.len())
        } 
        None
    }
}

// pad(3,"")
fn pad(n: usize, s: &str) -> String {
    let space = " ".repeat(n);
    format!("{}{}",space,s)
} 

// parse attr 
fn parse_attr( s: &str ) -> Result<Attribute,String> {

    let text = format!(r#"{}fn foo (){{}}"#,s); 
    let msg_error = |e:&str|-> String {
        format!("Internal Error.`parse::parse_attr`. Could not parse the Attribute. Error: {}",e.to_string())
    };
    match syn::parse_str::<Item>(&text) {

        Ok(fn_) => {
            match fn_ {
                Item::Fn(func) => {
                    if let Some(attr) = func.attrs.into_iter().next(){
                        Ok(attr)
                    } else {
                        Err(msg_error("Function `attrs` is empty."))
                    }
                },
                _ => Err(msg_error("Item is not a function.")),
            }
        },
        Err(e) => Err(msg_error(&e.to_string())),
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


    if let Err(e) = crate::show::write(prefix, &edit_attr.path){
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
    // println!("{new_attr_str}");

    }
    

    #[test]
    fn test_func_actor_edit(){
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
    // println!("{new_attr_str}");

    }
    
   // TESTS FOR PARSER
    #[test]
    fn explicit_chars_in_str(){
        let mut atp = ActiveTextParser::new(0);

        let s =r#"
        let a = '#'  ;
        let b = '\n' ;
        let c = '"'  ;
        let d = "foo";
        let g ="'\"'";
        let e =     1;
        "#;
        let r =r#"
        let a =      ;
        let b =      ;
        let c =      ;
        let d =      ;
        let g =      ;
        let e =     1;
        "#;
        
        let mut loc = Vec::new();
        for (index,line) in s.lines().enumerate(){

            let code_line = atp.parse((index,line.to_string()));
            loc.push(code_line);
        }
        let loc_new = 
        loc.into_iter().map(|x| x.1).collect::<Vec<_>>();
        let result = loc_new.join(LINE_ENDING);

        assert_eq!(&result,r)
    }


    #[test]
    fn  open_comment_test() { 
        let mut atp = ActiveTextParser::new(0);
        // let s = r###"br#"r##"r#"b""/*end"###;
                          
        if let Some((code,work)) = 
            atp.open_multy_line(r###"0//br#"r##"r#"b""/*end"###){
                    assert_eq!(code,"0");
                    assert_eq!(work,r###"  br#"r##"r#"b""/*end"###.to_string());
        }
        if let Some((code,work)) = 
            atp.open_multy_line(r###"1/*br#"r##"r#"b""end"###){
                    assert_eq!(code,"1");
                    assert_eq!(work,r###"  br#"r##"r#"b""end"###.to_string());
        }
        if let Some((code,work)) = 
            atp.open_multy_line(r###"2"br#"r##"r#"b"end"###){
                    assert_eq!(code,"2");
                    assert_eq!(work,r###" br#"r##"r#"b"end"###.to_string());
        }
        if let Some((code,work)) = 
            atp.open_multy_line(r###"3b"br#"r##"r#"end"###){
                    assert_eq!(code,"3");
                    assert_eq!(work,r###"  br#"r##"r#"end"###.to_string());
        }
        if let Some((code,work)) = 
            atp.open_multy_line(r###"4r#"br#"r##"end"###){
                    assert_eq!(code,"4");
                    assert_eq!(work,r###"   br#"r##"end"###.to_string());
        }
        if let Some((code,work)) = 
            atp.open_multy_line(r###"5r##"br#"end"###){
                    assert_eq!(code,"5");
                    assert_eq!(work,r###"    br#"end"###.to_string());
        }
        if let Some((code,work)) = 
            atp.open_multy_line(r###"6br#"end"###){
                    assert_eq!(code,"6");
                    assert_eq!(work,r###"    end"###.to_string());
        }
    }

    #[test]
    fn  close_cap_test() { 
        let mut atp = ActiveTextParser::new(0);
        let s = r###"123\"#45"*\"##end"###;
                          
        if let Some((code,work)) = 
            atp.close_multy_line(s, "\"#"){
            //               123\"#45"*\"##end
            assert_eq!(code,"      ".to_string());
            assert_eq!(work,r###"45"*\"##end"###);
        }
        if let Some((code,work)) = 
            atp.close_multy_line(s, "\""){
            //               123\"#45"*\"##end
            assert_eq!(code,"         ".to_string());
            assert_eq!(work,r###"*\"##end"###);
        }
        if let Some((code,work)) = 
            atp.close_multy_line(s, "*\\"){
            //               123\"#45"*\"##end
            assert_eq!(code,"           ".to_string());
            assert_eq!(work,r###""##end"###);
        }
        if let Some((code,work)) = 
            atp.close_multy_line(s, "\"##"){
            //               123\"#45"*\"##end
            assert_eq!(code,"              ".to_string());
            assert_eq!(work,r###"end"###);
        }
    }

    #[test]
    fn  parser_close_open() { 

        let mut atp = ActiveTextParser::new(0);
        let s = r#"eprintln!("12345");"#;

        let close = atp.close_multy_line(s, "\"");
        let open = atp.open_multy_line(s);

        let r_close = Some(("           ".to_string(),  "12345\");")) ;
        let r_open  = Some(("eprintln!("             , " 12345\");".to_string())) ;
        
        assert_eq!(close, r_close);
        assert_eq!(open ,  r_open);

        let ( cc,cw) = close.unwrap();
        let ( oc,ow) = open.unwrap();

        assert!( s.len() == (cc.len() + cw.len()));
        assert!( s.len() == (oc.len() + ow.len()));
    }
    

    // Old test for text parser 


   
    #[test]
    fn  parser_close_open_inline() { 
        let mut atp = ActiveTextParser::new(0);

        let s =r###"
        println!(   "12\"34🌍"  );
        println!(   "12🌍3\"4"  );
        println!(   "12🌍34\""  );
        println!(   "\"12🌍34"  );
        println!(  b"12🌍3\"4"  );
        println!(r##"1234\"🌍"##);
        println!(r#"🌍1234\""#  );
        println!(br#"\"1234🌍"# );
        println!("\"");
        println!("");
        "###;

        let r =r#"
        println!(                 );
        println!(                 );
        println!(                 );
        println!(                 );
        println!(                 );
        println!(                 );
        println!(                 );
        println!(                 );
        println!(    );
        println!(  );
        "#;

        let mut loc = Vec::new();
        for (index,line) in s.lines().enumerate(){
            loc.push(atp.parse((index,line.to_string())));
        }
        let loc_new = 
        loc.into_iter().map(|x| x.1).collect::<Vec<_>>();
        let result = loc_new.join(LINE_ENDING);

        assert_eq!(&result,r)
    }
    
}