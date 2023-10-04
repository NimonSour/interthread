use std::ops::Range;

use proc_macro_error::abort;
use proc_macro2::Span;


#[derive(Debug,Clone, Copy)]

pub struct NestedArgument {

    depth: usize,
    start: usize,
    equal: Option<usize>, 
     open: Option<usize>,
    close: Option<usize>,
     end : usize,
    comma: Option<usize>, 
}

impl NestedArgument {

    fn new(start:usize,depth:usize) -> Self {
        Self {
            depth: depth,
            start: start,
            equal: None, 
             open: None,
            close: None,
              end: start,
            comma: None, 
        } 
    }

    pub fn is_list(&self)      -> bool {
        self.open.is_some() &&
        self.close.is_some()
    }

    pub fn is_key_value(&self) -> bool {
        self.equal.is_some()
    }

    pub fn is_path(&self)      -> bool {
        self.equal.is_none() &&
        self.open.is_none()  
    }

    fn get_name(&self, str_arg: &str) -> String {
        let end;
        if self.is_list() {
            end = self.open.unwrap();
        }
        else if self.is_key_value() {
            end = self.equal.unwrap();
        }
        else {
            end = self.end;
        } 
        let name = str_arg[self.start..end].trim();
        return name.to_string();
    }
}

impl Default for NestedArgument {

    fn default() -> Self {
        Self {
            depth: 0,
            start: 0,
            equal: None, 
             open: None,
            close: None,
              end: 0,
            comma: None, 
        } 
    }
}



pub fn parse_args(str_arg: &str) -> Vec<NestedArgument> {

    let open  = '(';
    let close = ')';
    let equal = '=';
    let comm  = ',';

    let mut depth = 0;
    let mut args = Vec::new();

    let mut loc = vec![NestedArgument::new(0,0)];

    for (i,c) in str_arg.char_indices(){

        if c == open { 
            loc[depth].open = Some(i);
            depth += 1;
            loc.push( NestedArgument{ depth, start: i+1, ..Default::default() });
        } 

        else if c == close {
            loc[depth].end   = i;
            depth -= 1;
            loc[depth].close = Some(i);
        } 

        else if c == equal { 
            loc[depth].equal = Some(i); 
        }

        else if c == comm {
            loc[depth].end   = i;
            loc[depth].comma = Some(i);
            for _ in 0..loc[depth..].len() {
                if let Some( arg) = loc.pop() {
                    args.push(arg);
                }
            }
            loc.push( NestedArgument{ depth, start: i+1, ..Default::default() });
        } 
    }
    loc.append(&mut args);
    loc
}


pub fn get_edit(attr_str: &str, v: &Vec<NestedArgument>) -> NestedArgument {
    if let Some(pos) = v.iter().position(|x| x.get_name(attr_str).eq(crate::EDIT) && x.depth.eq(&1)) {
        v[pos].clone()
    } else { 
        let msg = "Internal Error.`parse::nested::get_edit`. Argument `edit` not found.";
        abort!(Span::call_site(),msg);
    }
}

pub fn file_in_edit(attr_str: &str, v: &Vec<NestedArgument>) -> Vec<(usize, Range<usize>)> {

    //find edit
    let arg_edit = get_edit(attr_str,v);
    let start = arg_edit.start;
    if let Some(end) = arg_edit.close{
        let files = v.into_iter()
            .filter_map(|n|
                if start < n.start &&  end > n.end {
                    if n.get_name(attr_str).eq(crate::FILE) { 
                        if n.is_list(){
                            Some(n.clone())
                        } else {
                            if  n.depth == 2 {
                                Some(n.clone())
                            } else { None }
                        }
                    } else { None }
                } else { None }
            )
            .collect::<Vec<_>>();
        let mut loc = files.iter().flat_map(|f| get_range( attr_str, &f )).collect::<Vec<_>>();
        loc.sort_by(|a,b| a.0.cmp(&b.0));
    
        loc
    } else {
        let msg = "Internal Error.`parse::nested::file_in_edit`. Expected  some `close` value.";
        abort!(Span::call_site(),msg);
    }
}



pub fn get_range( attr_str: &str, n: &NestedArgument) 
    -> Vec<(usize,std::ops::Range<usize>)>{

    if n.is_list() {

        let msg;
        //list 
        if let Some( open )  = n.open {
            if let Some( close )  = n.close {
                if let Some(mut file_start) = attr_str[n.start..=open].find(crate::FILE){
                    file_start             = file_start + n.start;
                    let head = file_start..open+1;
                    let tail = close..close+1;
                    return vec![(file_start,head),(close,tail)];
                } else { msg = "Internal Error.`parse::nested::get_range`. Expected `file` nested argument.";}
            } else { msg = "Internal Error.`parse::nested::get_range`. Expected  some `close` value.";}
        } else { msg = "Internal Error.`parse::nested::get_range`. Expected some `open` value.";}
        abort!(Span::call_site(),msg);
    } else {
        //path
        if n.comma.is_some(){
            vec![(n.start, n.start..n.comma.unwrap() +1)]
        } else {
            vec![(n.start, n.start..n.end)]
        }
    }
}

pub fn edit_remove_active_file_args(attr_str: &str) -> String {

    let mut new_attr_str = attr_str.to_string();
    let args = parse_args(attr_str);
    let ranges = file_in_edit(attr_str,&args);
    for (_,range) in ranges.into_iter().rev(){
        new_attr_str.replace_range(range, "");
    }
    new_attr_str
}
