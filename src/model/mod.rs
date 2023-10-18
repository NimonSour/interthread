
pub mod argument;
pub mod attribute;
pub mod generics;
pub mod method;
pub mod name;
pub mod actor;
pub mod group;


pub use argument::*;
pub use attribute::*;
pub use generics::*;
pub use method::*;
pub use name::*;
pub use actor::*;
pub use group::*;


use proc_macro2::TokenStream;
use proc_macro_error::abort;
use syn::{Generics,Ident};
use quote::quote;
// use crate::attribute::{AAEdit,AGEdit};




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

    // model: ActorModelSdpl,
    pub name:        Ident,
    // pub mac:         Model,
    pub edit:    EditGroup,
    // pub generics: Generics,
    pub actors: Vec<ActorModelSdpl>,
}

impl GroupModelSdpl {

    pub fn get_edit(&self) -> Edit {
        Edit::Group(self.edit.clone())
    }
}


/*
This means there should be:
    a)  struct  AGEdit {
            pub script:( bool, Option<Vec<syn::Ident>>, Option<Vec<syn::Ident>> ),
            pub live:  ( bool, Option<Vec<syn::Ident>>, Option<Vec<syn::Ident>> ),

            pub group: Vec< ( Ident, AAEdit )>

    }

*/


// Sdpl Actor 

pub struct ActorModelSdpl {
    pub name:        Ident,
    pub asyncness: Option<TokenStream>,
    pub mac:         Model,
    pub edit:    EditActor,
    pub generics: Generics,
    pub script: (  TokenStream,  Vec<(Ident,TokenStream)>,  Vec<(Ident,TokenStream)> ),
    pub live:   (  TokenStream,  Vec<(Ident,TokenStream)>,  Vec<(Ident,TokenStream)> ),
}


impl ActorModelSdpl {

    pub fn get_edit(&self) -> Edit {
        Edit::Actor(self.edit.clone())
    }
    pub fn is_empty(&self) -> bool {
        self.script.0.is_empty() && self.live.0.is_empty() &&
        self.script.1.is_empty() && self.live.1.is_empty() &&
        self.script.2.is_empty() && self.live.2.is_empty()
    }

    pub fn split_edit(&mut self) -> (TokenStream,TokenStream){

        let mut edit_script_def  = None;
        let mut edit_script_mets = None;
        let mut edit_script_trts = None;
    
        let mut edit_live_def  = None;
        let mut edit_live_mets = None;
        let mut edit_live_trts = None;



        let (script,live) = 
        match &self.edit {  EditActor{ script, live, ..  } => {(script.clone(),live.clone())}};
        
        let select = 
        |
        edit_cont: (Option<Vec<(Ident,bool)>>,bool),
        model_cont: &mut Vec<(Ident,TokenStream)>,
        | -> Option<Vec<TokenStream>>
        {
            let cont = edit_select(edit_cont,model_cont);
            if cont.is_empty() { None } else { Some(cont) }
        };

        let diff = 
        | ((def,scope_def),mets,trts): ( (bool,bool), (Option<Vec<(syn::Ident,bool)>>,bool), (Option<Vec<(syn::Ident,bool)>>,bool) ),
          model_def:  &mut TokenStream,
          model_mets: &mut Vec<(Ident,TokenStream)>,
          model_trts: &mut Vec<(Ident,TokenStream)>,
          edit_def:   &mut Option<TokenStream>,
          edit_mets:  &mut Option<Vec<TokenStream>>,
          edit_trts:  &mut Option<Vec<TokenStream>>
        |{
            if def {
                let temp_def = Some(model_def.clone());
                *model_def  = quote!{}; 

                if scope_def {
                    *edit_def = temp_def;
                }
            }
            // original 
            *edit_mets = select(mets,model_mets);
            *edit_trts = select(trts,model_trts);
        };

        diff(
            script,
             &mut self.script.0,
            &mut self.script.1,
            &mut self.script.2,
              &mut edit_script_def,
             &mut edit_script_mets,
             &mut edit_script_trts 
        );

        diff(
            live,
             &mut self.live.0,
            &mut self.live.1,
            &mut self.live.2,
              &mut edit_live_def,
             &mut edit_live_mets,
             &mut edit_live_trts 
        );
        
        let coll_token_stream = 
        |coll: &Vec<(Ident,TokenStream)>| -> Vec<TokenStream> 
        { coll.iter().map(|x| x.1.clone()).collect::<Vec<_>>() };
        // Prepare Token Stream Vecs
        let script_def         = &self.script.0;
        let script_methods = coll_token_stream(&self.script.1);
        let script_traits  = coll_token_stream(&self.script.2);

        let live_def           = &self.live.0;
        let live_methods   = coll_token_stream(&self.live.1);
        let live_traits    = coll_token_stream(&self.live.2);
        
        let(impl_generics,ty_generics ,where_clause) = self.generics.split_for_impl();
        let (script_name,live_name) = name::get_actor_names(&self.name, &self.mac);


        let res_code = quote! {
    
            #script_def
            impl #impl_generics #script_name #ty_generics #where_clause {
                #(#script_methods)*
            }
            #(#script_traits)*
    
            #live_def
            impl #impl_generics #live_name #ty_generics #where_clause {
                #(#live_methods)*
            }
            #(#live_traits)*
    
        };
    
    
        let res_edit_script_mets =  
            edit_script_mets.as_ref().map(|mets| 
                quote!{ 
                    impl #impl_generics #script_name #ty_generics #where_clause {
                        #(#mets)* 
                    }
                }
            );

        let res_edit_script_trts = 
            edit_script_trts.as_ref().map(|trts| 
                quote!{ #(#trts)* }
            );
    
        let res_edit_live_mets = 

            edit_live_mets.as_ref().map(|mets| 
                quote!{ 
                    impl #impl_generics #live_name #ty_generics #where_clause {
                        #(#mets)* 
                    }
                }
            ); 

        let res_edit_live_trts = 
        edit_live_trts.as_ref().map(|trts| 
            quote!{ #(#trts)* }
        );

        let res_edit = quote!{
    
            #edit_script_def
            #res_edit_script_mets
            #res_edit_script_trts
    
            #edit_live_def
            #res_edit_live_mets
            #res_edit_live_trts
        };
    
        (res_code, res_edit)
    
    }

}    


pub fn edit_select((edit_idents,scope): (Option<Vec<(Ident,bool)>>,bool), 
    ident_mets: &mut Vec<(Ident,TokenStream)> ) -> Vec<TokenStream> {

    let mut res = Vec::new();

    if let Some(idents) = edit_idents { 

        if idents.is_empty() {

            let temp_ident_mets = std::mem::replace(ident_mets,Vec::new());
            if scope {
                res = temp_ident_mets.into_iter().map(|x| x.1).collect::<Vec<_>>();
            }
        }

        for (ident,scp) in idents {
            if let Some(pos) = ident_mets.iter().position(|x| x.0 == ident){
                let (_,met)  = ident_mets.remove(pos);
                if scope || scp {
                    res.push(met);
                }
            } else {
                let msg = format!("No method named `{}` in Actor's methods.",ident.to_string());
                abort!(ident,msg);
            }
        }
    } 
    res
}





#[test]
fn test_split_edit_group() {

    let attr: syn::Attribute = 
    syn::parse_quote!{#[actor( edit(script(imp), 
                            a::edit(live,script(def))))] };

    let mut edit = EditGroup::default();

    for meta in crate::model::attribute::attr_to_meta_list(&attr){

        if meta.path().is_ident("edit"){
            edit.parse(&meta);
        }
    }
    println!("Edit - {:?}", edit);  
}


