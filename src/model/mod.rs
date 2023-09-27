pub mod channels;
pub mod debut;
pub mod edit;


pub use channels::*;
pub use debut::*;
pub use edit::*;

use proc_macro2::TokenStream;
use syn::{Generics,Ident};
use crate::attribute::{AAEdit,AGEdit};




// actor generate has to return this a vector of this types 
// pub enum Sdpl {
//     Script{ name: Ident, def: TokenStream, imp: Vec<(Ident,TokenStream)>, trt: Vec<(Ident,TokenStream)> },
//     Live  { name: Ident, def: TokenStream, imp: Vec<(Ident,TokenStream)>, trt: Vec<(Ident,TokenStream)> },
// }





// pub struct GroupModelSdpl {
//     pub name:                Ident,
//     pub edit:               AGEdit,
//     pub generics:         Generics,
//     pub parts: Vec<ActorModelSdpl>,
//     pub script: (  TokenStream,  Vec<(Ident,TokenStream)>,  Vec<(Ident,TokenStream)> ),
//     pub live:   (  TokenStream,  Vec<(Ident,TokenStream)>,  Vec<(Ident,TokenStream)> ),
// }


pub struct GroupModelSdpl {

    model: ActorModelSdpl,
    actors: Vec<ActorModelSdpl>,
}


/*
This means there should be:
    a)  struct  AGEdit {
            pub script:( bool, Option<Vec<syn::Ident>>, Option<Vec<syn::Ident>> ),
            pub live:  ( bool, Option<Vec<syn::Ident>>, Option<Vec<syn::Ident>> ),

            pub group: Vec< ( Ident, AAEdit )>

    }

*/




