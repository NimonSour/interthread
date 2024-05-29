


/*
Impotant the option has to work before 
any checks upon methods are performed, in case the user is 
using the option to exclude some methods that are not meant to be included in the model in first place.
 



[...]
 
 for include 
if vec.contains  && true 

for exclude 
*/

use proc_macro_error::abort;
use syn::{ Ident, Signature, Meta } ;
use crate::{error, model::{get_idents,get_list,check_path_set}};

#[derive(Debug,Clone)]
pub enum FilterSet {
    Include(Vec<Ident>),
    Exclude(Vec<Ident>),
}

impl FilterSet {

    pub fn parse(meta: &Meta, ioe: bool) -> Self {
        
        if let Some(meta_list) = get_list( meta,None ) {
            check_path_set(&meta_list);
            let idents = get_idents(&meta_list);
            if ioe { Self::Include(idents) } else { Self::Exclude(idents) } 
        } else {
            abort!(meta,error::EXPECT_LIST)
        }
    }

    pub fn condition (&mut self, sig: &Signature ) -> bool {
        let remove_fn_name = |list: &mut Vec<Ident>| {
            if let Some(pos) = list.iter().position(|x| x.eq(&sig.ident)){
                list.remove(pos);
            }
        };
        match self {
            Self::Include( list) =>  {
                if list.contains(&sig.ident) {
                    remove_fn_name(list);
                    true
                } else { false }
            },
            Self::Exclude(list) => {
                if list.contains(&sig.ident) {
                    remove_fn_name(list);
                    false
                } else { true }
            },  
        }
    }

    pub fn check( &self ){

        let list = 
        match self {
            Self::Include( list) => list,
            Self::Exclude(list) => list,
        };

        if !list.is_empty(){
            abort!(list[0], "Unknown method name.");
        }
    }

}
