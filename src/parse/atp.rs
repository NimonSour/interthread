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

