
use crate::model::{ EditAttribute,get_list};
use crate::error;
use syn::{punctuated::Punctuated,Ident};
use proc_macro_error::abort;
use std::collections::BTreeMap;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct EditActor {
    pub attr: Option<EditAttribute>,
    pub remove:  bool,
    pub script:( (bool,bool), (Option<Vec<(syn::Ident,bool)>>,bool), (Option<Vec<(syn::Ident,bool)>>,bool) ),
    pub live:  ( (bool,bool), (Option<Vec<(syn::Ident,bool)>>,bool), (Option<Vec<(syn::Ident,bool)>>,bool) ),
}


impl EditActor {

    pub fn abort_if_is_file( meta: &syn::Meta ){
        if meta.path().is_ident(crate::FILE) { 
            abort!(meta, error::NESTED_FILE; help=error::HELP_EDIT_FILE_ACTOR);
        }  
    }

    pub fn get_file_list( meta: &syn::Meta ) -> Punctuated::<syn::Meta,syn::Token![,]>{
        if let Some(meta_list) = get_list( meta,Some(error::AVAIL_EDIT) ) { 
            return meta_list; 
        } else { abort!(meta,"Expected a list!"; note=error::NOTE_SPECIAL_FILE_EDIT; help=error::HELP_SPECIAL_FILE_EDIT); }
    }

    pub fn parse(&mut self,  meta: &syn::Meta ){

        if let Some(meta_list) = get_list( meta,Some(error::AVAIL_EDIT) ) {
            
            if meta_list.len() == 1 {

                if let Some(meta_value) = meta_list.first(){

                    if meta_value.path().is_ident( crate::FILE ){

                        if let Some(list) = get_list( meta_value,Some(error::AVAIL_EDIT) ) {

                            for m in list.iter(){
                                Self::abort_if_is_file(m);
                                self.parse_sol(m,true);
                            }

                        } else { 

                            self.set_script_all();
                            self.set_script_all_active();
                            self.set_live_all();
                            self.set_live_all_active();
                            self.remove = true;
                        }

                    } else { self.parse_sol(meta_value,false); }
                } 
            } else {

                for meta in meta_list.iter() {

                    if meta.path().is_ident(crate::FILE){

                        for m in Self::get_file_list(meta).iter() { 
                            Self::abort_if_is_file(m);
                            self.parse_sol(m,true);
                        }

                    } else { self.parse_sol(meta,false); }
                }
            }
        } else {
            self.set_script_all();
            self.set_live_all();
        }
    }

    pub fn parse_sol( &mut self, meta: &syn::Meta, file:bool ){

        let sol:bool;

        if meta.path().is_ident("script") {
            if self.is_script_none() {
                sol = true;
            } else { abort!(meta,error::double_decl("script");help=error::HELP_EDIT_FILE_ACTOR); }
        } 

        else if meta.path().is_ident("live") {

            if self.is_live_none() {
                sol = false;
            } else { abort!(meta,error::double_decl("live");help=error::HELP_EDIT_FILE_ACTOR); }
        }

        // wrong opt
        else { abort!(meta,"Unexpected 'edit' option!"; help=error::AVAIL_EDIT ); } 

        if let Some(meta_list) = get_list( meta,Some(error::AVAIL_EDIT) ) { 
                
            for met in meta_list.iter() {

                if met.path().is_ident(crate::FILE) {
                    if file { abort!(met, error::NESTED_FILE; help=error::HELP_EDIT_FILE_ACTOR); }
                    
                    for m in Self::get_file_list(met).iter() { 
                        Self::abort_if_is_file(m);
                        self.parse_sol_nested(m,sol,true); 
                    }

                } else { self.parse_sol_nested(met,sol,file); }
            }

        } else {

            if sol {
                if file { self.set_script_all_active(); }
                self.set_script_all();

            } else {
                if file { self.set_live_all_active(); }
                self.set_live_all();
            }
        } 
    }


    pub fn parse_sol_nested(&mut self, meta: &syn::Meta, sol: bool, file:bool ){

        let (name,mut tuples) = 
        if sol {("script",self.script.clone()) } else { ("live",self.live.clone())};

        let mut iot = None;
        if meta.path().is_ident("def"){

            if !tuples.0.0 {
                if file {
                    tuples.0.1 = true; 
                } 
                tuples.0.0 = true; 
                
            } else { abort!(meta, error::double_decl(&format!("{name}::def")); help=error::HELP_EDIT_FILE_ACTOR); }
        }
        
        else if meta.path().is_ident("imp") {
            if tuples.1.0.is_none() {
                iot = Some(&mut tuples.1);
            } else { abort!(meta, error::double_decl(&format!("{name}::imp")); help=error::HELP_EDIT_FILE_ACTOR); }
        }
        
        else if meta.path().is_ident("trt") {
            if tuples.2.0.is_none(){
                iot = Some(&mut tuples.2);
            } else { abort!(meta, error::double_decl(&format!("{name}::trt")); help=error::HELP_EDIT_FILE_ACTOR); }
        }

        else {
            let msg = format!("Unexpected 'edit({}( ? ))' option! Expected options are `def`,`imp` or `trt` .",name);
            abort!(meta, msg);
        }
        
        if let Some(iot) = iot {

            Self::parse_sol_nested_idents(iot, &meta, file);
        }

        if sol { self.script = tuples; } else { self.live = tuples; };

    }


    pub fn parse_sol_nested_idents( 
        (opt,scope): &mut(Option<Vec<(syn::Ident,bool)>>,bool), 
        meta: &syn::Meta,
        file:bool 
        ){
        
        if let Some(meta_list) = get_list(meta, Some(error::HELP_EDIT_FILE_ACTOR)) {
            
            for m in meta_list.iter(){
                if m.path().is_ident(crate::FILE){
                    if let Some(file_list) = get_list(m, Some(error::HELP_EDIT_FILE_ACTOR)) {
                        
                        for fm in file_list {

                            if file {

                                abort!(fm, error::NESTED_FILE; help=error::HELP_EDIT_FILE_ACTOR); 

                            } else { Self::add_if_unique(opt,&fm, true); }
                        }
                    } else { Self::add_if_unique(opt, m, file); } 

                } else { Self::add_if_unique(opt, m, file); }   
            }
        } else { 
            if file { *scope = true; } 
            *opt = Some(Vec::new());
        }
    }


    pub fn add_if_unique(vec: &mut Option<Vec<(syn::Ident,bool)>>, meta:&syn::Meta, file: bool ){
        let ident = crate::model::attribute::get_ident(meta);
        if let Some(v) = &vec {
            if v.iter().any(|(i,_)| ident.eq(i) ){
                abort!(meta,crate::error::double_decl(&ident.to_string()));
            } else {
                vec.as_mut().map(|v| v.push((ident,file)));
            }
        } else {
            *vec = Some(vec![(ident,file)]);
        }
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

    // pub fn is_live_all(&self)-> bool{
    //     self.live.0.0 == true &&
    //     self.live.1.0 == Some(Vec::new()) &&
    //     self.live.2.0 == Some(Vec::new()) 
    // }

    // pub fn is_script_all (&self)-> bool{
    //     self.script.0.0 == true &&
    //     self.script.1.0 == Some(Vec::new()) &&
    //     self.script.2.0 == Some(Vec::new()) 
    // }

    // pub fn is_all(&self) -> bool {
    //     self.is_live_all() &&
    //     self.is_script_all()
    // } 

    // pub fn is_live_all_active(&self)-> bool{
    //     self.live.0.1 == true &&
    //     self.live.1.1 == true &&
    //     self.live.2.1 == true 
    // }

    // pub fn is_script_all_active(&self)-> bool{
    //     self.script.0.1 == true &&
    //     self.script.1.1 == true &&
    //     self.script.2.1 == true 
    // }

    // pub fn is_all_active(&self) -> bool {
    //     self.is_live_all_active() &&
    //     self.is_script_all_active()
    // } 

    pub fn is_live_none(&self) -> bool {
        self.live.0.0 == false &&
        self.live.1.0 == None  &&
        self.live.2.0 == None  
    } 

    pub fn is_script_none(&self) -> bool {
        self.script.0.0 == false &&
        self.script.1.0 == None  &&
        self.script.2.0 == None 
    } 

    // pub fn is_none(&self) -> bool {
    //     self.is_live_none() &&
    //     self.is_script_none()
    // } 

    // pub fn is_live_none_active(&self) -> bool {
    //     self.live.0.1 == false &&  
    //     self.live.1.1 == false &&  
    //     self.live.2.1 == false 
    // } 

    // pub fn is_script_none_active(&self) -> bool {
    //     self.script.0.1 == false &&  
    //     self.script.1.1 == false &&  
    //     self.script.2.1 == false 
    // } 

    // pub fn is_none_active(&self) -> bool {
    //     self.is_live_none_active() && 
    //     self.is_script_none_active()
    // } 
}

impl Default for EditActor {

    fn default() -> Self {
        let attr = None;
        let script = 
        ((false,false),(None,false),(None,false));
        let live   = 
        ((false,false),(None,false),(None,false));
        Self { attr,remove: false, script, live }
    } 
}





#[derive(Debug, Eq, PartialEq, Clone)]
pub struct EditGroup {
    pub attr:      Option<EditAttribute>,
    pub remove:                     bool,
    pub edits: Option<BTreeMap<Ident,EditActor>>, 
}

impl EditGroup {

    pub fn parse(&mut self, meta: &syn::Meta ) {


        if let Some(meta_list) = get_list( meta,Some(error::AVAIL_EDIT_GROUP) ) {
            
            if meta_list.len() == 1 { 
                if let Some(meta_value) = meta_list.first(){

                    // check for `file`
                    if meta_value.path().is_ident(crate::FILE){ 
                        if let Some(_) = get_list( meta_value,Some(error::AVAIL_EDIT_GROUP) ) {
                            // if is a list raise an error
                            abort!(meta_value,error::EDIT_GROUP_FILE_OUTSIDE;note=error::AVAIL_EDIT_GROUP);
                        } else {

                            self.edits = Some(BTreeMap::new());
                            self.remove = true;
                        }
                    } else {
                        /* check if is valid, parse  */
                        self.parse_meta(meta_value);
                    }

                } else { /* internal error  */ }

            } else {
                // check for ..::edit
                crate::model::check_path_set(&meta_list);
                for m in meta_list.iter() {
                    if m.path().is_ident(crate::FILE){ 
                        abort!(m,error::EDIT_GROUP_FILE_OUTSIDE;note=error::AVAIL_EDIT_GROUP);
                    } else {
                        self.parse_meta(m);
                    }
                }
            }

        } else { self.edits = Some(BTreeMap::new()); }

    }


    pub fn parse_meta(&mut self, meta: &syn::Meta ){

        let ident = crate::model::get_ident_group(&meta,"edit");
        let mut new_edit = EditActor::default();
        new_edit.parse(meta);
        if self.edits.is_some() {
            self.edits.as_mut().map(|x| x.insert(ident,new_edit));
        } else { 
            self.edits = Some(BTreeMap::from([(ident,new_edit)]));
        }
    } 

    pub fn is_any_active(&self) -> bool {
        let mut bol = false;
        self.edits
            .as_ref()
            .map(|v| {
                bol = v.iter()
                       .any(|e| e.1.is_any_active() == true );
                }
            );
        bol
    }


}

impl Default for EditGroup {

    fn default() -> Self {

        let attr = None;
        let edits = None;
        let remove = false;
        Self { attr,remove, edits}
    } 
}




