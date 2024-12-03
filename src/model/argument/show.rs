

use syn::{ parse2, Attribute, File, Ident, ImplItemFn, Item, ItemEnum, ItemStruct};
use proc_macro_error::abort_call_site;
use quote::{ToTokens,quote};
use crate::LINE_ENDING;

#[derive(Debug,Clone)]
pub struct ShowComment{
    pub show: bool,
}

impl ShowComment {

    pub fn parse_model_part<T:ToTokens>(&self, def: &T, mets: &Vec<(Ident,ImplItemFn)>,trts: &Vec<(Ident,Item)>) -> Item {
        let mut item = Self::parse_item(def);
        if self.show {
            let attr = Self::model_part_doc_comment(def,mets,trts);
            match &mut item {
                Item::Struct(ItemStruct{attrs,..})|
                Item::Enum(ItemEnum{attrs,..}) => {attrs.push(attr)},
                _ => abort_call_site!("Internal Error. 'show::parse_model_part'. unexpected syn::Item.")
            }
        }
        item
    }

    pub fn parse_method<T: ToTokens>(&self, tokens: &T) -> ImplItemFn {

        let mut impl_item_fn = Self::parse_fn(tokens);
        let mut attrs = std::mem::take(&mut impl_item_fn.attrs); 
        if self.show {
            
            let mut msg = "### Interthread Generated Code".to_string();
            msg += LINE_ENDING;
            msg += & Self::code_format(&impl_item_fn);
            msg += LINE_ENDING;

            attrs.push( Self::parse_doc_attr(&msg) );
        } 

        impl_item_fn.attrs = attrs;
        impl_item_fn
    }

    fn model_part_doc_comment<T: ToTokens>( def: &T, mets: &Vec<(Ident,ImplItemFn)>,trts: &Vec<(Ident,Item)>) -> Attribute {
        let sigs = mets.iter().map(|(_,met)| met.sig.clone()).collect::<Vec<_>>();

        let mut msg_mets =  "```rust ignore".to_string(); 
        for sig in sigs {
            msg_mets += LINE_ENDING;
            msg_mets += &quote!{ #sig }.to_string();
        }
        msg_mets += LINE_ENDING;
        msg_mets += "```";
        

        let mut msg_trts = "".to_string();

        for (trt,_) in trts {
            msg_trts += LINE_ENDING;
            msg_trts += &format!("- {trt}");
        }

        let mut msg = "### Interthread Generated Code".to_string();
        msg += LINE_ENDING;
        msg += & Self::code_format(def);
        msg += LINE_ENDING;
        msg += "### Available Methods ";
        msg += LINE_ENDING;
        msg += LINE_ENDING;
        msg += &msg_mets;
        msg += LINE_ENDING;
        msg += "### Implemented Traits ";
        msg += LINE_ENDING;
        msg += &msg_trts;
        msg += LINE_ENDING;
        msg += LINE_ENDING;

        Self::parse_doc_attr(&msg)
    }

    fn code_format<T: ToTokens>( tokens: &T ) -> String {

        let file = Self::parse_file(tokens);
        let mut msg_code =  "```rust ignore".to_string();
        msg_code += LINE_ENDING;            
        msg_code += &prettyplease::unparse(&file);
        msg_code += LINE_ENDING; 
        msg_code += "```";
        msg_code += LINE_ENDING; 
        msg_code 

    }

    pub fn parse_item<T: ToTokens>( tokens: &T) -> Item {
        if let Ok(item) =  parse2::<syn::Item>(quote!{#tokens}){
            return item;
        } 
        abort_call_site!( "Internal Error. 'show::parse_item'. Failed to parse TokenStream to syn::Item.");   
    }

    pub fn parse_fn<T: ToTokens>( tokens: &T) -> ImplItemFn {
        if let Ok(item) =  parse2::<ImplItemFn>(quote!{#tokens}){
            return item;
        } 
        abort_call_site!( "Internal Error. 'show::parse_fn'. Failed to parse TokenStream to syn::ItemFn.");   
    }

    fn parse_file<T: ToTokens>( tokens: &T) -> File { 
        if let Ok(file) =  parse2::<File>(quote!{#tokens}){
            return file;
        } 
        abort_call_site!( "Internal Error. 'show::parse_file'. Failed to parse TokenStream to syn::File.");
    }
    fn parse_doc_attr( msg: &str ) -> Attribute {

        let code = quote!{
            #[doc = #msg]
            fn foo (){{}}
        };
    
        if let Ok(item_fn) =  parse2::<syn::ItemFn>(code) {
            if let Some(attr) = item_fn.attrs.into_iter().next(){ return attr; } 
        }
        abort_call_site!("Internal Error.`show::parse_doc_attr`. Could not parse the Attribute.")
    }
}

impl Default for ShowComment {
    fn default() -> ShowComment {
        Self { show: false } 
    }
}