


/*
Impotant the option has to work before 
any checks upon methods are performed, in case the user is 
using the option to exclude some methods that are not meant to be included in the model in first place.
*/

use std::collections::HashSet;

use proc_macro_error::abort;
use syn::{ Ident, Signature, Meta } ;
use crate::{error, model::{check_path_set, get_idents, get_list, ActorAttributeArguments, ConstVars}};

#[derive(Debug,Clone)]
pub enum FilterSet {
    Include(Vec<Ident>),
    Exclude(Vec<Ident>),
}

impl FilterSet {

    pub fn parse(meta: &Meta, ioe: bool) -> Self {
        
        if let Some(meta_list) = get_list( meta,None ) {
            check_path_set(&meta_list,None);
            let idents = get_idents(&meta_list);
            // checking for mentions of `new` or `try_new` 
            for ident in  idents.iter(){
                if ident == "new" || ident == "try_new" { abort!(ident,format!("unexpected mention of `{ident}`"))}
            }
            if ioe { Self::Include(idents) } else { Self::Exclude(idents) } 
        } else {
            abort!(meta,error::EXPECT_LIST)
        }
    }

    pub fn condition (&mut self, ident: &Ident ) -> bool {
        let remove_fn_name = |list: &mut Vec<Ident>| {
            if let Some(pos) = list.iter().position(|x| x.eq(ident)){
                list.remove(pos);
            }
        };
        match self {
            Self::Include( list) =>  {
                if list.contains(ident) {
                    remove_fn_name(list);
                    true
                } else { false }
            },
            Self::Exclude(list) => {
                if list.contains(ident) {
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

pub struct ModelFilter {
    cust_set: FilterSet,
    inter_set: HashSet<Ident>,
}

impl ModelFilter {

    pub fn new( aaa: &ActorAttributeArguments ) -> Self {
       let cust_set = if let Some( fs ) = &aaa.filter { fs.clone() } else { FilterSet::Exclude(vec![]) }; 
       let const_vars =  ConstVars::new();
       let inter_set = const_vars.get_inter_mets_set(aaa);
       Self{ cust_set, inter_set }
    }

    pub fn condition(&mut self, sig: &Signature ) -> bool {
        if self.inter_set.contains(&sig.ident){
            let msg = error::var_name_conflict(&sig.ident.to_string(),"method");
            abort!(sig.ident,msg);
        } else {
            self.cust_set.condition(&sig.ident)
        }
    }
    pub fn check(&self){
        self.cust_set.check()
    }


}
