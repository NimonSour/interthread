

use syn::{ parse2, Attribute, ItemFn, Item, File, ItemStruct, ItemEnum, Ident};
use proc_macro2::TokenStream;
use proc_macro_error::abort_call_site;
use quote::quote;
use crate::LINE_ENDING;

pub struct ShowComment;

impl ShowComment {

    pub fn parse_model_part(  def: TokenStream,  mets: Vec<(Ident,TokenStream,Vec<Attribute>)>,  trts: Vec<(Ident,TokenStream)>,  show: bool, sol: bool ) 
        -> ( Option<Item>, Vec<(Ident,Item)>, Vec<(Ident,Item)> )
    {

        let trts = 
            trts.into_iter()
                .map(|(ident,tokens)| (ident,Self::parse_item(tokens)))
                .collect::<Vec<_>>();
        let mets = 
            mets.into_iter()
                .map(|(ident,tokens ,attrs)| (ident,Self::parse_method(tokens,attrs,show)))
                .collect::<Vec<_>>();

        let attr = if show { Some(Self::model_part_doc_comment(def.clone(),&mets,&trts))} else { None };

        let mets = mets.into_iter().map(|(ident,item_fn)| (ident,Item::Fn(item_fn))).collect::<Vec<_>>();
        
        if sol {
            //script
            let mut item_enum = Self::parse_enum(def);
            if let Some(attr) = attr {
                item_enum.attrs.push(attr);
            }
            (Some(Item::Enum(item_enum)), mets, trts )
        } else {
            //live
            let mut item_struct = Self::parse_struct(def);
            if let Some(attr) = attr {
                item_struct.attrs.push(attr);
            }
            (Some(Item::Struct(item_struct)), mets, trts )
        }
        
    }

    pub fn parse_item( item: TokenStream ) -> Item {
        if let Ok(item) =  parse2::<syn::Item>(item){
            return item;
        } 
        abort_call_site!( "Internal Error. 'show::parse_item'. Failed to parse TokenStream to syn::Item.");   
    }

    pub fn parse_fn( item: TokenStream ) -> ItemFn {
        if let Ok(item) =  parse2::<ItemFn>(item){
            return item;
        } 
        abort_call_site!( "Internal Error. 'show::parse_fn'. Failed to parse TokenStream to syn::ItemFn.");   
    }

    pub fn parse_file( item: TokenStream ) -> File { 
        if let Ok(file) =  parse2::<File>(item){
            return file;
        } 
        abort_call_site!( "Internal Error. 'show::parse_file'. Failed to parse TokenStream to syn::File.");
    }

    pub fn parse_enum( item: TokenStream ) -> ItemEnum {
        if let Ok(file) =  parse2::<ItemEnum>(item){
            return file;
        } 
        abort_call_site!( "Internal Error. 'show::parse_enum'. Failed to parse TokenStream to syn::ItemEnum.");
    }

    pub fn parse_struct( item: TokenStream ) -> ItemStruct {
        if let Ok(file) =  parse2::<ItemStruct>(item){
            return file;
        } 
        abort_call_site!( "Internal Error. 'show::parse_struct'. Failed to parse TokenStream to syn::ItemStruct.");
    }



    fn parse_doc_attr( msg: &str ) -> Attribute {

        let code = quote!{
            #[doc = #msg]
            fn foo (){{}}
        };
    
        match parse2::<syn::ItemFn>(code) {
    
            Ok(item_fn) => {
                if let Some(attr) = item_fn.attrs.into_iter().next(){ return attr; } 
                else { abort_call_site!("Internal Error.`show::parse_doc_attr`. Function `attrs` is empty."); }
            },
            Err(_) => abort_call_site!("Internal Error.`show::parse_doc_attr`. Could not parse the Attribute."),
        }
    }

    fn code_format( item: TokenStream ) -> String {

        let file = Self::parse_file(item);
        let mut msg_code =  "```rust ignore".to_string();
        msg_code += LINE_ENDING;            
        msg_code += &prettyplease::unparse(&file);
        msg_code += LINE_ENDING; 
        msg_code += "```";
        msg_code += LINE_ENDING; 
        msg_code 

    }

    pub fn parse_method( item: TokenStream, mut attrs: Vec<Attribute>, show: bool ) -> ItemFn {

        let mut item_fn = Self::parse_fn(item.clone());

        if show {
            
            let mut msg = "### Interthread Generated Code".to_string();
            msg += LINE_ENDING;
            msg += & Self::code_format(item);
            msg += LINE_ENDING;

            let attr = Self::parse_doc_attr(&msg); 
            attrs.push(attr);
        } 

        item_fn.attrs = attrs;
        item_fn
    }

    fn model_part_doc_comment( def: TokenStream, mets: &Vec<(Ident,ItemFn)>,trts: &Vec<(Ident,Item)>) -> Attribute {
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
}
