use crate::name::get_name_and_type;
use crate::file::get_idents;
use crate::show::get_text;
use crate::attribute::AAFile;
use crate::attribute::AAExpand;
use crate::LINE_ENDING;

use proc_macro_error::abort;
use proc_macro2::Span;

use syn::{Attribute,Item,Visibility};



fn set_attrs( attrs: &Vec<Attribute>, item: &Item ) -> Item {
    let item = item.clone();

    match item {
        Item::Fn(mut body)      => {
            body.attrs = attrs.clone();
            Item::Fn(body)
        },
        Item::Trait(mut body) => {
            body.attrs = attrs.clone();
            Item::Trait(body)
        },
        Item::Impl(mut body)   => {
            body.attrs = attrs.clone();
            Item::Impl(body)
        },
        _ => {abort!(Span::call_site(),"Internal Error. `parse::set_attrs`. Expected Fn, Imbl block or Trait !".to_string())},
    }
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
    let idents = get_idents(attr);
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


fn get_attrs( item: &Item ) -> Result<(Vec<Attribute>, Item),String>{
    let item = item.clone();
    let attrs;

    let res = match item {

        Item::Fn(mut body)       => {
            attrs = body.attrs;
            body.attrs = Vec::new();
            (attrs, Item::Fn(body))
        },
        Item::Trait(mut body) => {
            attrs = body.attrs;
            body.attrs = Vec::new();
            (attrs, Item::Trait(body))
        },
        Item::Impl(mut body)   => {
            attrs = body.attrs;
            body.attrs = Vec::new();
            (attrs, Item::Impl(body))
        },
        _ => { return Err("Internal Error. `parses::get_attrs`. Expected Fn, Imbl block or Trait !".to_string())},
    };

    Ok( res )
}



pub struct ActiveTextParser {
    pub open:      Option<String>,
    data:   Option<(usize,usize)>,
    index:                  usize,
}

impl  ActiveTextParser {

    pub fn new(index: usize) -> Self {
        Self {
            open: None,
            data: None,
            index,
        }
    }

    // Original 
    fn open_multy_line<'b>(&mut self,  s:&'b str ) -> Option<(&'b str,String)> {

        let mut ss = s;
        let len;
        if s.is_empty(){ return None; } else { len = s.len();}
        let mut work = String::new();

        if s.contains("/"){
            // Single Line
            if let Some(pos) = ss.find("//") {
                work = pad(2,&s[pos+2..]);
                ss = &s[..pos];
            }
            // Multy Line
            if let Some(pos) = ss.find("/*") {
                work = pad(2,&s[pos+2..]);
                ss = &s[..pos];
                self.open = Some("*/".into());
            }
        } 
        // Char (')
        if let Some(pos) = ss.find("'"){
            work = pad(1,&s[pos+1..]);
            ss = &s[..pos];
            self.open = Some("'".into());

        }
        // String Literal (")
        if let Some(mut pos) = ss.find("\"") {

            // if pos < index {
            let mut slf_len = 1;
            let mut open   = String::from("\"");
            
            // Byte String Literal (b")
            if let Some(new_pos) = preceded_by(ss,pos,"b") {
                pos = new_pos;
                slf_len = 2;
            }
            
            // Raw String Literal   (r#")
            else if let Some(new_pos) = preceded_by(ss,pos,"#") {
                let mut loc_new_pos = new_pos;
                let mut loc_slf_len = 2;
                let mut loc_open = String::from("\"#");
                loop {
                    if let Some(new_pos) = preceded_by(ss,loc_new_pos,"#"){
                        loc_new_pos = new_pos;
                        loc_slf_len += 1;
                        loc_open += "#";
                    } else { break; }
                }   
                if let Some(new_pos) = preceded_by(ss,loc_new_pos,"r") {
                    loc_new_pos = new_pos;
                    loc_slf_len += 1;

                    // RawByteStringLiteral (br#")
                    if let Some(new_pos) = preceded_by(ss,loc_new_pos,"b") { 
                        loc_new_pos = new_pos;
                        loc_slf_len += 1;
                    }
                    pos     = loc_new_pos;
                    slf_len = loc_slf_len;
                    open    = loc_open;
                }
            }
            work = pad(slf_len,&s[pos+slf_len..]);
            ss = &s[..pos];
            self.open = Some(open);
        }
        if len == ss.len() { None } else { Some((ss, work)) }
    }

    fn close_multy_line<'b>( &mut self, s:&'b str, cap: &str ) -> Option<(String,&'b str)> {
        
        if let Some(mut pos) = s.find(cap){
            if cap == "\"" {
                loop {
                    if preceded_by(s,pos,r#"\"#).is_some(){
                        pos += 1;
                        if pos  < s.len(){
                            if let Some(new_pos) = s[pos..].find(cap){
                                pos += new_pos;
                            } else {return None;}
                        } else {return None;}
                    } else {break;}
                }
            }
            pos = pos + cap.len() ;
            self.open = None;
            return Some((pad(pos,""), &s[pos..]));
        }
        None
    }
    
    fn record(&mut self, (i,s): (usize,&str)) -> usize{
        let (len,total) = self.data.unwrap_or((0,0));
        self.data = Some((s.len(),total+len));

        (i*LINE_ENDING.len())+len+total
    } 

    pub fn parse ( &mut self,  (i,s):(usize, String)) -> (usize,String) {
        let index = self.record((i,&s));
        let mut work = s;
        let mut code = String::new();
        loop {
            if let Some(cap) = &self.open {
                if let Some((c,w)) = 
                    self.close_multy_line(&work,&cap.clone()){
                    if w.is_empty(){
                        code += &c;
                        break;
                    } else {
                        code += &c;
                        work = w.into();
                    }   
                } else { 
                    // if is not close than is w_space
                    code += &pad(work.len(),"");
                    break;
                }
                
            } else {
                if let Some((c, w)) = self.open_multy_line(&work){

                    if self.open.is_none() {
                        code += c;
                        code += &pad(w.len(),"");
                        break;
                    }
                    else {
                        code += c;
                        work  = w; 
                    }
                } else {
                    // if is not open than is code 
                    code += &work;
                    break;
                }
            }
        }
        (self.index + index,code)
    }
}


pub struct ItemCodeBlock{
    src  : String,
    index:  usize,

    depth: usize,   
    token: &'static str,
    open :  char,
    close:  char,

    first: Option<usize>,
    start: Option<usize>,
    end  : Option<usize>,
    code_block:   String,
}

impl ItemCodeBlock {

    pub fn new( src: String ) -> Self {
        let src =src.lines()
                            .collect::<Vec<_>>()
                            .join(LINE_ENDING);
        Self{ src, 
            index: 0,

            depth: 0,
            token: "#",
            open : '[',
            close: ']',

            first: None,
            start: None,
            end  : None,
            code_block: String::new(),
        }
    }
    
    fn reset(&mut self, line: Option<(&mut String,usize)>,  item: Option<&Item> ) -> Result<(),String>{

        if let Some((code,end)) = line{
            let space = &code[..=end];
            *code = pad(space.len(),&code[end+1..]);
        }

        self.depth = 0;
        self.first = None; 
        self.start = None;
        self.end   = None;
        self.code_block = String::new();

        if let Some(itm) = item {

            match itm {

                Item::Impl(_) => {
                    self.token = "impl";
                    self.open  = '{';
                    self.close = '}';
                },

                Item::Fn(v) => {
                    self.open  = '{';
                    self.close = '}';
                    match v.vis {
                        Visibility::Inherited => {
                            if let Some(_) = v.sig.asyncness {
                                self.token = "async";
                            }
                            self.token = "fn";
                        },
                        _ => { self.token = "pub"; },
                    }
                },

                Item::Trait(v) => {
                    match v.vis {
                        Visibility::Inherited => {
                            self.token = "trait";
                        },
                        _ => { self.token = "pub"; },
                    }
                },
                _ => {
                    let msg = "Internal Error. `ItemCodeBlock::reset`. Expected Fn, Imbl block or Trait".to_string();
                    return Err(msg);
                },
            }

        } else {

            self.token = "#";
            self.open  = '[';
            self.close = ']';
        }
        Ok(())
    }
    
    fn parse_item(&self,s: Option<&str>) -> Result<Item,String> {

        let body_text = s.unwrap_or(&self.src[self.first.unwrap()..=self.end.unwrap()]); 

        match syn::parse_str::<Item>(body_text) {

            Ok(item) => Ok(item),
            Err(e) => { 
                let msg = format!("Internal Error. `ItemCodeBlock::parse_item`. Could not parse the item! {}",e);
                Err(msg) 
            },
        }
    }
    
    fn check_name(&self, item: &Item) -> bool {
        let (name,_) = 
        match item {
            Item::Fn(_) => { get_name_and_type(&AAExpand::Group, item)},
                          _  => { get_name_and_type(&AAExpand::Actor, item)},
        };
        self.code_block.contains(&name.to_string()) 
    }

    fn first_index(&self, done_attrs: &mut Vec<(usize,Attribute,String)>) -> usize {
        
        if done_attrs.len() > 1 {
            done_attrs.sort_by_key(|item| item.0);
        }
        done_attrs[0].0
    }

    pub fn get_item_code(&mut self, mut attrs: Vec<Attribute>, item: Item ) -> Result<Vec<(usize, Attribute, String)>,String> {

        let org_item = set_attrs(&attrs, &item);
        if attrs.is_empty(){ 
            self.reset(None,Some(&item))?; 
        } 

        let mut done_attrs = Vec::new();
        let mut atp        = ActiveTextParser::new(self.index);
        let mut lines = 
        self.src[self.index..]
            .lines()
            .enumerate()
            .map( |x| (x.0 , x.1.to_string()))
            .collect::<Vec<_>>()
            .into_iter();

        let mut line = lines.next();

        'w1: while  let Some(i_l)  = line {
            // line goes to  atp 
            let (index,mut code) = atp.parse(i_l);

            'l1: loop {

                if self.token != "#" {
                    if code.contains("#"){
                        while let Some((_,a,_)) = done_attrs.pop() {
                            attrs.push(a);
                        }
                        let _ = self.reset(None, None)?;
                    }
                }

                if self.first.is_some(){

                    for (i,c) in code.clone().char_indices() {
                        // add char to future code_block 
                        self.code_block.push(c);

                        if self.start.is_some() {
                            if c == self.open { 
                                self.depth += 1;

                                if self.token != "#" {

                                    if !self.check_name(&item){

                                        // start all over again
                                        while let Some((_,a,_)) = done_attrs.pop() {
                                            attrs.push(a);
                                        }
                                        // reset to search for attr 
                                        let _ = self.reset(Some((&mut code, i)),None)?;
                                        continue 'l1;
                                    }
                                }
                            } else if c == self.close {

                                self.depth -= 1;
                                if self.depth == 0 {
                                    self.end  = Some(index+i);

                                    if self.token == "#" {

                                        let attr_str = &self.src[self.first.unwrap()..=self.end.unwrap()];
                                        let attr = parse_attr(attr_str)?;

                                        'l2: loop {

                                            if let Some(pos)  = attrs.iter().position(|x| x.eq(&attr)){
                                                done_attrs.push((self.first.unwrap(), attrs.remove(pos),self.code_block.clone()));

                                                if attrs.is_empty(){

                                                    let _ = self.reset(Some((&mut code, i)),Some(&item))?;
                                                    continue 'l1;
                                                } else {

                                                    let _ = self.reset(Some((&mut code, i)),None)?;
                                                    continue 'l1;
                                                }
                                            } else {

                                                if !done_attrs.is_empty(){

                                                    attrs.push(done_attrs.remove(0).1);
                                                    continue 'l2;
                                                } else {

                                                    let _ = self.reset(Some((&mut code, i)),None)?;
                                                    continue 'l1;
                                                }
                                            }
                                        }
                                    } else {

                                        if let Ok(out_item) = self.parse_item(None){
                                            if out_item == item {

                                                let first_index    = self.first_index(&mut done_attrs);
                                                let full_str        = &self.src[first_index..=self.end.unwrap()];

                                                if let Ok(out_item) = self.parse_item(Some(full_str)){
                                                    if out_item == org_item {
                                                        self.index = index + i + 1;
                                                        return Ok(done_attrs)
                                                    }
                                                }
                                            }
                                        } 
                                        while let Some((_,a,_)) = done_attrs.pop() {
                                            attrs.push(a);
                                        }
                                        let _ = self.reset(Some((&mut code, i)), None)?;
                                        continue 'l1;
                                    }
                                }
                            } 
                        } else {
                            // DEPTH 
                            if c == self.open {
                                self.start = Some(index+i);
                                self.depth += 1;
                            }
                        } 
                    }
                    // add line ending
                    self.code_block += LINE_ENDING;
                } 

                else if let Some(pos) = code.find(self.token) {
                    self.first = Some(index + pos);
                    continue 'l1;
                } 
                line = lines.next();
                continue 'w1;
            }
        }
        Err("Expected Item not found!".to_string())
    }
}



pub fn split_file( aaf: &AAFile, item: Item, edit: bool ) -> (String,String,) {

    match  get_text(&aaf.path){

        Ok(text) => {
            
            let mut icb = ItemCodeBlock::new(text);

            match icb.get_item_code(aaf.attrs.clone(),item){
                Ok(attrs) => {
                    let (prefix,suffix) = icb.src.split_at(icb.index);

                    let mut prefix  = prefix.to_string();
                    if let Some(pos) = attrs.iter().position(|x| x.1.eq(&aaf.attr)){

                        let index =  attrs[pos].0;
                        let s   = &attrs[pos].2;

                        if edit {
                            let end = index + s.len();
                            prefix.replace_range(index..=end, "");
                        } else {
                            let (start,end) = find_file(index,s,&aaf.attr);
                            prefix.replace_range(start..end, "");
                        }
                        return (prefix,suffix.into());
                    }
                    // no position internal error
                    abort!(Span::call_site(),"Internal Error. 'parse::split_file' . Did not match any Attribute in Attributes !");

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


pub fn edit_write(  aaf: &AAFile, 
                   item: Item, 
                   repl: bool,
                    mac: &AAExpand,  
                   edit: proc_macro2::TokenStream ) {

    let (name, _ )=  crate::name::get_name_and_type(mac, &item);
    let edifile    =  syn::parse2::<syn::File>(edit).unwrap();
    let edifix   =  prettyplease::unparse(&edifile);

    let (mut prefix, suffix) = split_file( aaf, item, repl );
    
    let attr = &aaf.attr;
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


    if let Err(e) = crate::show::write(prefix, &aaf.path){
        proc_macro_error::abort!(proc_macro2::Span::call_site(),e.to_string());
    }

}  


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_find_attr(){

        let s = r#"
        struct Bla(i8);

        #[example(file="src/bla.rs")]
        #[ actor (channel = 2,
             edit(play))
        ] 
        impl Bla {

            fn new(v: i8) -> Self {
                Self(v)
            }
        }

        #[example(
            file="src/bla.rs"
        
        
        )] #[ actor (channel = 2, edit(play))] 
        pub fn actor_exam_play( value: i8 ) -> i8 {
        9
        }"#;


        let data_fn = r#"

        #[example(file="src/bla.rs")] 
        #[ actor (channel = 2, edit(play))]
        // bla 
        pub fn actor_exam_play( value: i8 ) -> i8 {
        9 }
        "#;

        let item_fn: syn::Item = syn::parse_str(data_fn).expect("could not parse item");

        let mut icb = ItemCodeBlock::new(s.to_string());
        let (attrs,item) = get_attrs(&item_fn).unwrap();
        match icb.get_item_code(attrs,item){
            Ok(v) => {println!("Ok({:?})",v);},
            Err(e) => {println!("Err({})",e);},
        }
    }
    
    
    #[test]
    fn catch_nn() {
        // THIS FUNCTION IS NOT USED
        fn find_nn( s: &str) -> usize {
            if let Some(pos) = s.find('\n'){
                // first item is a ','
                if pos > 1 {
                    if (&s[1..pos]).chars().all(|x| x==' '){
                        return pos;
                    } else { return 0;}
                } else { return pos; }
            }
            0
        }
    
    // zero space after comma
    let s = r#",
bla"#;

    // one space after comma
    let ss = r#", 
bla"#;
    // two spaces after comma 
    let sss = r#",  
bla"#;
    // no '\n' at all 
    let ssss = r#",  bla"#;
    

    println!(r#"
s    - {},
ss   - {},
sss  - {},    
ssss - {},    
"#,find_nn(s), find_nn(ss), find_nn(sss), find_nn(ssss));

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