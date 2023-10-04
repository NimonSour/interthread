
use crate::model::{
    argument::{Model,EditAttribute},
    attribute::get_list
};

use crate::error;
use crate::model::name;


use quote::quote;
use syn::{punctuated::Punctuated,Ident,Generics,};


use proc_macro2::TokenStream;
use proc_macro_error::abort;



pub fn edit_ident( meta: &syn::Meta, scope: bool ) -> (syn::Ident,bool) {
    if let Some(ident) = meta.path().get_ident(){
        if let Some(list) = get_list(&meta,Some(&format!("Did you mean '{ident}(file)'."))){
            if let Some(new_list) = filter_file(&list){
                if scope {
                    abort!(new_list,"1The option 'file' is overlapped.";help=error::HELP_EDIT_FILE_ACTOR);
                } else {
                    if new_list.is_empty() {
                        (ident.clone(),true)
                    } else { abort!(new_list, "Unexpected option.";help=error::HELP_EDIT_FILE_ACTOR)}
                }
            } else { abort!(list, "Unexpected option.";help=error::HELP_EDIT_FILE_ACTOR) }
        } else { (ident.clone(),false) }
    } else { abort!( meta, "Expected an identation."); }
}


#[derive(Debug, Eq, PartialEq, Clone)]

pub struct Edit {
    pub attr: Option<EditAttribute>,
    pub script:( (bool,bool), (Option<Vec<(syn::Ident,bool)>>,bool), (Option<Vec<(syn::Ident,bool)>>,bool) ),
    pub live:  ( (bool,bool), (Option<Vec<(syn::Ident,bool)>>,bool), (Option<Vec<(syn::Ident,bool)>>,bool) ),
}

impl Edit {

    pub fn edit_structs(&mut self, meta: &syn::Meta, sol: bool){ 
        // tuples: &mut ( (bool,bool), (Option<Vec<(syn::Ident,bool)>>,bool), (Option<Vec<(syn::Ident,bool)>>,bool) ), 

        let tuples = 
        if sol {&self.script } else { &self.live };

        if let Some(list) = get_list(meta,Some(error::AVAIL_EDIT) ){

            // file in script
            if let Some(new_list) = filter_file(&list){
                // let msg = quote::quote!{ #new_list}.to_string();
                // abort!(meta,msg);
                // not declared already 
                if tuples.0.1 || tuples.1.1  || tuples.2.1  {
                    abort!(meta,"2The option 'file' is overlapped.";help=error::HELP_EDIT_FILE_ACTOR);
                } else {
                    if new_list.is_empty() {
                        self.set_script_all_active();
                        self.set_script_all();
                    } else { self.parse_nested_sol(new_list,sol); }
                }
            } else { self.parse_nested_sol(list,sol); }

        } else { self.set_script_all(); }
    }

    pub fn parse_nested(&mut self,  meta: &syn::Meta ) {

        if let Some(mut meta_list) = get_list( meta,Some(error::AVAIL_EDIT) ) {

            if let Some(new_list) = filter_file(&meta_list){
                // let msg = quote::quote!{ #new_list}.to_string();
                // abort!(meta_list,msg);

                self.set_script_all_active();
                self.set_live_all_active();
                // needs a check if is empty
                if new_list.is_empty(){
                    self.set_script_all();
                    self.set_live_all();
                } else {
                    meta_list = new_list;
                }
            }


            for edit_meta in meta_list.iter() {

                if edit_meta.path().is_ident("script"){
                    // is a list
                    self.edit_structs(edit_meta,true);
                } 

                else if edit_meta.path().is_ident("live"){

                    // edit_structs(&mut self.script,edit_meta,false);
                    self.edit_structs(edit_meta,false);
                    /*
                     
                    // is a list
                    // if let Some(mut list) = get_list(edit_meta,Some(error::AVAIL_EDIT) ){
                    //     // file in script
                    //     if let Some(new_list) = filter_file(&list){
                    //         // not declared already 
                    //         if self.edit.script.0.1 || self.edit.script.1.1  || self.edit.script.2.1  {
                    //             abort!(edit_meta,"Option 'file' has already been declared!" );
                    //         } else {
                    //             if new_list.is_empty() {

                    //                 self.edit.set_live_all_active();
                    //                 self.edit.set_live_all();

                    //             } else { self.edit.parse_nested(new_list,false); }
                    //         }
                    //     } else { self.edit.parse_nested(list,false); }

                    // } else { self.edit.set_live_all(); }
                    */
                } 

                // old args 
                else if  edit_meta.path().is_ident("direct") {
                    abort!( meta, crate::error::OLD_DIRECT_ARG);
                }
                else if  edit_meta.path().is_ident("play") {
                    abort!( meta, crate::error::OLD_PLAY_ARG);
                }
                // wrong opt
                else {
                    abort!(edit_meta,"Unexpected 'edit' option!";help=error::AVAIL_EDIT );
                } 
            }
        } else {
            self.set_script_all();
            self.set_live_all();
        }
    }

    pub fn parse_nested_sol(&mut self, nested: Punctuated::<syn::Meta,syn::Token![,]>, sol: bool ){

        let (name,mut strct) = 
        if sol {("script",self.script.clone()) } else { ("live",self.live.clone())};

        let list_idents = |tuple: &mut (Option<Vec<(syn::Ident,bool)>>,bool), meta: &syn::Meta|{
            if let Some(list) = get_list(meta, Some(error::HELP_EDIT_FILE_ACTOR)) {
                if let Some(new_list) = filter_file(&list){
                    if !tuple.1 { tuple.1 = true; } else { abort!(meta,"3The option 'file' is overlapped.";help=error::HELP_EDIT_FILE_ACTOR);}
                    tuple.0 = Some(new_list.iter().map(|x| edit_ident(x,tuple.1)).collect::<Vec<_>>());
                } else {
                    tuple.0 = Some(list.iter().map(|x| edit_ident(x,tuple.1)).collect::<Vec<_>>());
                }
            } else { tuple.0 = Some(Vec::new()); }
        };

        for meta in nested.iter() {

            if meta.path().is_ident("def"){
                let (_,scope) = edit_ident(meta,strct.0.1);
                if scope {
                    if !strct.0.1 { strct.0.1 = scope; } else { abort!(meta,"4The option 'file' is overlapped.";help=error::HELP_EDIT_FILE_ACTOR);}
                } 
                strct.0.0 = true;
            }
            
            else if meta.path().is_ident("imp"){
                list_idents(&mut strct.1, &meta);
                // abort!(Span::call_site(),"After Imp");
            }
            
            else if meta.path().is_ident("trt"){
                list_idents(&mut strct.2, &meta);
            }

            else {
                let msg = format!("Unexpected 'edit({}( ? ))' option! Expected options are `def`,`imp` or `trt` .",name);
                abort!(meta, msg);
            }
        }

        if sol { self.script = strct; } else { self.live = strct; };

    }

    pub fn is_any_active(&self) -> bool {

        let any_active = 
        | 
            tuples: &((bool,bool), (Option<Vec<(syn::Ident,bool)>>,bool), (Option<Vec<(syn::Ident,bool)>>,bool) )
        |{
            if tuples.0.1 || tuples.1.1 || tuples.2.1 {
                true
            } else {
                let mut imp_bol = false;
                let mut trt_bol = false;
                if let Some(imp) = &tuples.1.0{
                    imp_bol = imp.iter().any(|m|m.1 == true);
                }

                if let Some(imp) = &tuples.2.0{
                    trt_bol = imp.iter().any(|m|m.1 == true);
                }
                imp_bol || trt_bol
            }
        };
        any_active(&self.script) || any_active(&self.live)
    }

    pub fn set_live_all(&mut self){
        self.live.0.0 = true;
        self.live.1.0 = Some(Vec::new());
        self.live.2.0 = Some(Vec::new());
    }

    pub fn set_script_all (&mut self){
        self.script.0.0 = true;
        self.script.1.0 = Some(Vec::new());
        self.script.2.0 = Some(Vec::new());
    }

    pub fn set_live_all_active(&mut self){
        self.live.0.1 = true;
        self.live.1.1 = true;
        self.live.2.1 = true;
    }

    pub fn set_script_all_active(&mut self){
        self.script.0.1 = true;
        self.script.1.1 = true;
        self.script.2.1 = true;
    }

    pub fn is_all(&self) -> bool {
        let empty = Some(Vec::new());

        self.live.0.0 == true  && self.script.0.0 == true  &&
        self.live.1.0 == empty && self.script.1.0 == empty &&
        self.live.2.0 == empty && self.script.1.0 == empty
    } 

    pub fn is_none(&self) -> bool {

        self.live.0.0 == false && self.script.0.0 == false &&
        self.live.1.0 == None  && self.script.1.0 == None  &&
        self.live.2.0 == None  && self.script.2.0 == None
    }  
    pub fn is_none_active(&self) -> bool {

        self.live.0.1 == false && self.script.0.1 == false &&
        self.live.1.1 == false && self.script.1.1 == false &&
        self.live.2.1 == false && self.script.2.1 == false
    }  

}

impl Default for Edit {

    fn default() -> Self {
        let attr = None;
        let script = 
        ((false,false),(None,false),(None,false));
        let live   = 
        ((false,false),(None,false),(None,false));
        Self { attr, script, live }
    } 
}

pub fn filter_file(meta_list: &Punctuated::<syn::Meta,syn::Token![,]>) 
    -> Option<Punctuated::<syn::Meta,syn::Token![,]>>{

    let filtered_list: Punctuated::<syn::Meta,syn::Token![,]> = 
        meta_list.clone()
              .into_iter()
              .filter(|m| !m.path().is_ident("file"))
              .collect();

    if meta_list.len() == filtered_list.len() {
        None
    } else {
        Some(filtered_list)
    }
}





pub struct ActorModelSdpl {
    pub name:        Ident,
    pub mac:         Model,
    pub edit:         Edit,
    pub generics: Generics,
    pub script: (  TokenStream,  Vec<(Ident,TokenStream)>,  Vec<(Ident,TokenStream)> ),
    pub live:   (  TokenStream,  Vec<(Ident,TokenStream)>,  Vec<(Ident,TokenStream)> ),
}


impl ActorModelSdpl {

    pub fn split_edit(&mut self) -> (TokenStream,TokenStream){

        let mut edit_script_def  = None;
        let mut edit_script_mets = None;
        let mut edit_script_trts = None;
    
        let mut edit_live_def  = None;
        let mut edit_live_mets = None;
        let mut edit_live_trts = None;



        let (script,live) = 
        match &self.edit {  Edit{ script, live, ..  } => {(script.clone(),live.clone())}};
        
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

                if scope_def{
                    *edit_def = temp_def;
                }
            }
            *edit_mets = Some(edit_select(mets,model_mets));
            *edit_trts = Some(edit_select(trts,model_trts));

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
        
    
        // Prepare Token Stream Vecs
        let script_def         = &self.script.0;
        let script_methods = self.script.1.iter().map(|x| x.1.clone()).collect::<Vec<_>>();
        let script_traits  = self.script.2.iter().map(|x| x.1.clone()).collect::<Vec<_>>();

        let live_def           = &self.live.0;
        let live_methods   = self.live.1.iter().map(|x| x.1.clone()).collect::<Vec<_>>();
        let live_traits    = self.live.2.iter().map(|x| x.1.clone()).collect::<Vec<_>>();
        
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

// OLD EDIT
/*

    // Create and Select Edit Parts

    let mut edit_script_def   = quote!{};
    let edit_script_mets ;
    let edit_script_trts ;

    let mut edit_live_def  = quote!{};
    let edit_live_mets ;
    let edit_live_trts ;


    match aaa.edit {

        crate::attribute::AAEdit  { live, script } => {
            match script {

                ( def , mets, trts) => {
                    if def {
                        edit_script_def = script_def.clone();
                        script_def      = quote!{}; 
                    }
                    edit_script_mets = edit_select(mets,&mut script_mets);
                    edit_script_trts = edit_select(trts,&mut script_trts);
                },
            }

            match live {

                ( def , mets, trts) => {
                    if def {
                        edit_live_def = live_def.clone();
                        live_def      = quote!{}; 
                    }
                    edit_live_mets = edit_select(mets,&mut live_mets);
                    edit_live_trts = edit_select(trts,&mut live_trts);
                },
            }
        }
    }

    // Prepare Token Stream Vecs
    let script_methods = script_mets.iter().map(|x| x.1.clone()).collect::<Vec<_>>();
    let script_traits  = script_trts.iter().map(|x| x.1.clone()).collect::<Vec<_>>();
    let live_methods   = live_mets.iter().map(|x| x.1.clone()).collect::<Vec<_>>();
    let live_traits    = live_trts.iter().map(|x| x.1.clone()).collect::<Vec<_>>();
    

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
    if  edit_script_mets.is_empty() { quote!{} }
    else { quote!{ 
        impl #impl_generics #script_name #ty_generics #where_clause {
            #(#edit_script_mets)* 
        }
    }};

    let res_edit_script_trts =  
    if  edit_script_trts.is_empty() { quote!{} }
    else { quote!{ #(#edit_script_trts)* }};

    let res_edit_live_mets =  
    if  edit_live_mets.is_empty() { quote!{} }
    else { quote!{ 
        impl #impl_generics #live_name #ty_generics #where_clause { 
            #(#edit_live_mets)* 
        }
    }};

    let res_edit_live_trts =  
    if  edit_live_trts.is_empty() { quote!{} }
    else { quote!{ #(#edit_live_trts)* }};


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

*/


pub fn edit_select((edit_idents,scope): (Option<Vec<(Ident,bool)>>,bool), 
    ident_mets: &mut Vec<(Ident,TokenStream)> ) -> Vec<TokenStream> {

    let mut res = Vec::new();

    if let Some(idents) = edit_idents { 

        if idents.is_empty() {
            // let temp_ident_mets = ident_mets.clone();
            let temp_ident_mets = std::mem::replace(ident_mets,Vec::new());
            if scope {
                res = temp_ident_mets.into_iter().map(|x| x.1).collect::<Vec<_>>();
            }
            // ident_mets.clear();
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

