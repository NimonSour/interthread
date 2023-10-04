pub mod icb;
pub mod atp;
pub mod nested;

pub use icb::ItemCodeBlock;
pub use atp::ActiveTextParser;

use crate::model::name::get_ident_type_generics;
use crate::file::get_idents;
use crate::show::get_text;
use crate::model::argument::{Model,EditAttribute};

use crate::LINE_ENDING;

use proc_macro_error::abort;
use proc_macro2::Span;

use syn::{Attribute,Item,ItemImpl};



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

// takes the &str from ItemCodeBlock

fn find_file( index: usize, s: &str, attr: &Attribute) -> (usize,usize) {

    let open  = '(';
    let close = ')';
    let comm  = ',';

    let mut depth = 0;
    let mut loc = Vec::new();
    // find commas
    for (i,c) in s.char_indices(){
        if !loc.is_empty() {
            if c == open { depth += 1; } 
            else if c == close {
                depth -= 1;
                if depth == 0 { loc.push(i); break;}
            } 
            else if c == comm {
                if depth == 1 { loc.push(i); }
            } 
        } else {
            if c == open { depth += 1; loc.push(i); }
        } 
    }
    

    // find file arg
    let nested = crate::file::to_nested(attr);
    let idents = get_idents(&nested);
    let file_ident = quote::format_ident!("file");

    if let Some(pos) = idents.iter().position(|x| file_ident.eq(x)){
        let (start, end) = 
        if pos == 0 {
            (loc[pos] + 1, loc[pos+1]+1)
        } else {

            (loc[pos],loc[pos+1])
        };

        let sub = &s[start..end];
        if sub.contains("file"){
            return (start+index,end+index);
        }
    }

    abort!(Span::call_site(), "InternalError. `parse::find_file` .Could not find `file` argument!");

}



pub fn split_file( edit_attr: &EditAttribute, item_impl: &ItemImpl, edit: bool ) -> (String,String,) {

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

                        if edit {
                            let end = index + s.len();
                            prefix.replace_range(index..=end, "");
                        } else {
                            let new_attr = &edit_attr.new_attr;
                            let new_attr_str = quote::quote!{ #new_attr}.to_string();
                            let end = index + s.len();
                            prefix.replace_range(index..=end, &new_attr_str);
                            // let (start,end) = find_file(index,s,&aaf.attr);
                            // prefix.replace_range(start..end, "");
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
                   edit_attr: EditAttribute, 
                   item_impl: &ItemImpl, 
                        repl: bool,
                        _mac: &Model,  
                        edit: proc_macro2::TokenStream ) {

    let (name, _, _)     =  get_ident_type_generics(&item_impl);
    let edifile    =  syn::parse2::<syn::File>(edit).unwrap();
    let edifix   =  prettyplease::unparse(&edifile);

    let (mut prefix, suffix) = split_file( &edit_attr, item_impl, repl );
    
    let attr = &edit_attr.attr;
    let mut attr_str = quote::quote!{ #attr }.to_string();
    attr_str = (&attr_str).replace(LINE_ENDING,"");
    attr_str = (&attr_str).replace(" ","");
    
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


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_functions(){
        let attr_str = r#"
#[interthread::actor(
    file="path/to/abc.rs",
    edit(
         
        file ,
        script( def, imp, trt ),
        file(live(   def, imp, trt)),

        Self::a( script( def, imp(file(bla),file), trt(file)),
           live(   file(def), imp, trt ), 
        ),

        Self::b( file(script( def, imp)),
           live(   def, imp(file), trt), 
        ),
    )
)]"#;


    let new_attr_str = nested::edit_remove_active_file_args(attr_str);

    let expect_attr_str = r#"
#[interthread::actor(
    file="path/to/abc.rs",
    edit(
        script( def, imp, trt ),
        live(   def, imp, trt),

        Self::a( script( def, imp(bla,file), trt(file)),
           live(   def, imp, trt ), 
        ),

        Self::b( script( def, imp),
           live(   def, imp(file), trt), 
        ),
    )
)]"#;

    assert_eq!(expect_attr_str,new_attr_str);
    // println!("{new_attr_str}");

    }
    

    
    // #[test]
    // fn test_find_attr(){
    //     // NOT USED
    //     fn get_attrs( item: &Item ) -> Result<(Vec<Attribute>, Item),String>{
    //         let item = item.clone();
    //         let attrs;
        
    //         let res = match item {
            
    //             Item::Fn(mut body)       => {
    //                 attrs = body.attrs;
    //                 body.attrs = Vec::new();
    //                 (attrs, Item::Fn(body))
    //             },
    //             Item::Trait(mut body) => {
    //                 attrs = body.attrs;
    //                 body.attrs = Vec::new();
    //                 (attrs, Item::Trait(body))
    //             },
    //             Item::Impl(mut body)   => {
    //                 attrs = body.attrs;
    //                 body.attrs = Vec::new();
    //                 (attrs, Item::Impl(body))
    //             },
    //             _ => { return Err("Internal Error. `parses::get_attrs`. Expected Fn, Imbl block or Trait !".to_string())},
    //         };
        
    //         Ok( res )
    //     }


    //     let s = r#"
    //     struct Bla(i8);

    //     #[example(file="src/bla.rs")]
    //     #[ actor (channel = 2,
    //          edit(play))
    //     ] 
    //     impl Bla {

    //         fn new(v: i8) -> Self {
    //             Self(v)
    //         }
    //     }

    //     #[example(
    //         file="src/bla.rs"
        
        
    //     )] #[ actor (channel = 2, edit(play))] 
    //     pub fn actor_exam_play( value: i8 ) -> i8 {
    //     9
    //     }"#;


    //     let data_fn = r#"

    //     #[example(file="src/bla.rs")] 
    //     #[ actor (channel = 2, edit(play))]
    //     // bla 
    //     pub fn actor_exam_play( value: i8 ) -> i8 {
    //     9 }
    //     "#;

    //     let item_fn: syn::Item = syn::parse_str(data_fn).expect("could not parse item");

    //     let mut icb = ItemCodeBlock::new(s.to_string());
    //     let (attrs,item) = get_attrs(&item_fn).unwrap();
    //     match icb.get_item_code(attrs,item){
    //         Ok(v) => {println!("Ok({:?})",v);},
    //         Err(e) => {println!("Err({})",e);},
    //     }
    // }
    
    
//     #[test]
//     fn catch_nn() {
//         // THIS FUNCTION IS NOT USED
//         fn find_nn( s: &str) -> usize {
//             if let Some(pos) = s.find('\n'){
//                 // first item is a ','
//                 if pos > 1 {
//                     if (&s[1..pos]).chars().all(|x| x==' '){
//                         return pos;
//                     } else { return 0;}
//                 } else { return pos; }
//             }
//             0
//         }
    
//     // zero space after comma
//     let s = r#",
// bla"#;

//     // one space after comma
//     let ss = r#", 
// bla"#;
//     // two spaces after comma 
//     let sss = r#",  
// bla"#;
//     // no '\n' at all 
//     let ssss = r#",  bla"#;
    

//     println!(r#"
// s    - {},
// ss   - {},
// sss  - {},    
// ssss - {},    
// "#,find_nn(s), find_nn(ss), find_nn(sss), find_nn(ssss));

// }

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