use super::{pad,preceded_by};
use crate::LINE_ENDING;

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
    pub fn open_multy_line<'b>(&mut self,  s:&'b str ) -> Option<(&'b str,String)> {

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

        // possible lifetime catch block 
        if let Some(open) = &self.open {
            if open.eq("'"){
                let pos = ss.len();
                for (i,c) in (&s[(pos+1)..]).chars().enumerate(){
                    match c {
                        ' ' | ',' => {
                            self.open = Some("".into());
                            let (new_ss,new_w)  = s.split_at(pos +i);
                            ss = new_ss;
                            work = new_w.into();
                        },
                        '\'' => break,
                        _ => (),
                    }
                }
            }
        }

        if len == ss.len() { None } else { Some((ss, work)) }
    }

    pub fn close_multy_line<'b>( &mut self, s:&'b str, cap: &str ) -> Option<(String,&'b str)> {
        
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
                if !cap.is_empty() { 

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
                } else { self.open = None; }
                
            } else {
                if let Some((c, w)) = self.open_multy_line(&work){

                    if self.open.is_none() {
                        code += c;
                        code += &pad(w.len(),"");
                        break;

                    } else {
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



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifetime() {

        let inp = 
r#"
fn example❤️<'a,'b,'c>(x: &'a char, y: &'b char, z: &'c char ) -> String {
    let mut s = String::new();
    s.push(*x);
    s.push(*y);
    s.push(*z);
    s
}
example❤️(&'a',&'\n',&'\u{2764}');
"#;
        let exp = 
r#"
fn example❤️<'a,'b,'c>(x: &'a char, y: &'b char, z: &'c char ) -> String {
    let mut s = String::new();
    s.push(*x);
    s.push(*y);
    s.push(*z);
    s
}
example❤️(&   ,&    ,&          );
"#;
        
        let mut atp = ActiveTextParser::new(0);
        let (_,code) = atp.parse((0,inp.to_string()));

        assert_eq!(exp,code);

    }
        

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
