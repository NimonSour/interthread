use super::{ActiveTextParser,pad,set_attrs,parse_attr};
use crate::model::get_ident_type_generics;
use crate::LINE_ENDING;

use syn::{Attribute,ItemImpl};

pub struct ItemCodeBlock{
    pub src  : String,
    pub index:  usize,

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
    
    fn reset(&mut self, line: Option<(&mut String,usize)>,  item_impl: Option<&ItemImpl> ) -> Result<(),String>{

        if let Some((code,end)) = line{
            let space = &code[..=end];
            *code = pad(space.len(),&code[end+1..]);
        }

        self.depth = 0;
        self.first = None; 
        self.start = None;
        self.end   = None;
        self.code_block = String::new();

        if let Some(_) = item_impl {

            // match itm {

            //     Item::Impl(_) => {
                    self.token = "impl";
                    self.open  = '{';
                    self.close = '}';
            //     },

            //     Item::Fn(v) => {
            //         self.open  = '{';
            //         self.close = '}';
            //         match v.vis {
            //             Visibility::Inherited => {
            //                 if let Some(_) = v.sig.asyncness {
            //                     self.token = "async";
            //                 }
            //                 self.token = "fn";
            //             },
            //             _ => { self.token = "pub"; },
            //         }
            //     },

            //     Item::Trait(v) => {
            //         match v.vis {
            //             Visibility::Inherited => {
            //                 self.token = "trait";
            //             },
            //             _ => { self.token = "pub"; },
            //         }
            //     },
            //     _ => {
            //         let msg = "Internal Error. `ItemCodeBlock::reset`. Expected Fn, Imbl block or Trait".to_string();
            //         return Err(msg);
            //     },
            // }

        } else {

            self.token = "#";
            self.open  = '[';
            self.close = ']';
        }
        Ok(())
    }
    
    fn parse_item_impl(&self,s: Option<&str>) -> Result<ItemImpl,String> {

        let body_text = s.unwrap_or(&self.src[self.first.unwrap()..=self.end.unwrap()]); 

        match syn::parse_str::<ItemImpl>(body_text) {

            Ok(item) => Ok(item),
            Err(e) => { 
                let msg = format!("Internal Error. `ItemCodeBlock::parse_item_impl`. Could not parse the item! {}",e);
                Err(msg) 
            },
        }
    }
    
    fn check_name(&self, item_impl: &ItemImpl) -> bool {
        let (name,_,_) = get_ident_type_generics(item_impl);
        // match item {
        //     Item::Fn(_)      => { get_name_and_type(&Model::Group, item)},
        //                   _  => { get_name_and_type(&Model::Actor, item)},
        // };
        self.code_block.contains(&name.to_string()) 
    }

    fn first_index(&self, done_attrs: &mut Vec<(usize,Attribute,String)>) -> usize {
        
        if done_attrs.len() > 1 {
            done_attrs.sort_by_key(|item| item.0);
        }
        done_attrs[0].0
    }

    pub fn get_item_code(&mut self, mut attrs: Vec<Attribute>, item_impl: &ItemImpl ) -> Result<Vec<(usize, Attribute, String)>,String> {

        let org_item = set_attrs(&attrs, item_impl);
        if attrs.is_empty(){ 
            self.reset(None,Some(&item_impl))?; 
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

                                    if !self.check_name(&item_impl){

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

                                                    let _ = self.reset(Some((&mut code, i)),Some(&item_impl))?;
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

                                        if let Ok(out_item) = self.parse_item_impl(None){
                                            if out_item.eq(item_impl) {

                                                let first_index    = self.first_index(&mut done_attrs);
                                                let full_str        = &self.src[first_index..=self.end.unwrap()];

                                                if let Ok(out_item) = self.parse_item_impl(Some(full_str)){
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

