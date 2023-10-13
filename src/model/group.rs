
use crate::model::attribute::GroupAttributeArguments;

use syn::{ItemImpl};
use proc_macro2::TokenStream;



/*
// in other file 
pub struct Cc;

// in this file  
struct Aa;
struct Bb;

pub struct AaBbCc {
    pub a: Aa,
    pub b: Bb, 
    pub c: Cc,
    n: AnyOtherType,
}




1) 'file' argument for current file 
    #[interthread::group(
        file="path/to/this/file.rs",
        Cc = "path/to/other/file.rs"    or c = "path/to/other/file.rs" or     c(path="path/to/other/file.rs")
    )]

    path( a::path("path/to/a.rs"), b::path("path/to/b.rs"))
     

    path( a::path="path/to/a.rs", b::path="path/to/b.rs")


2) Find and get the fields of struct in file.

    a) get the name from item_impl 
    b) find enum or struct with the same name
                if enum return an error  group works for structs only  
    c) get first impl block of the object 
    d)  

    struct_ visibility
    field ( ident, type, visibility )


*/


pub fn group_model( gaa: GroupAttributeArguments, 
                    item_impl: &ItemImpl ) {

    

}

pub fn macro_group_generate_code(
    gaa: GroupAttributeArguments, 
    item_impl: ItemImpl ) 
    -> ( TokenStream, TokenStream ) {

        todo!()

}