use crate::error;
use crate::model::{Model,ActorModelSdpl, Channel,Lib,EditGroup,Debut,get_ident,get_ident_group,get_lit,get_lit_str,get_list,to_usize};


use std::path::PathBuf;
use proc_macro2::Span;
use proc_macro_error::abort;
use quote::format_ident;
use syn::{Ident,ItemStruct,ItemImpl,Visibility,Type,punctuated::Punctuated,Meta};
use std::collections::BTreeMap;

use super::ActorAttributeArguments;

// GROUP ARGUMENTS 
// pub struct  AGEdit {
//     pub script:( bool, Option<Vec<syn::Ident>>, Option<Vec<syn::Ident>> ),
//     pub live:  ( bool, Option<Vec<syn::Ident>>, Option<Vec<syn::Ident>> ),
//     pub groupart: Option<Vec< (syn::Ident, Edit)>>,
// }

// impl AGEdit {

//     pub fn set_live_all(&mut self){
//         self.live = (true,Some(Vec::new()),Some(Vec::new()));
//     }

//     pub fn set_script_all (&mut self){
//         self.script = (true,Some(Vec::new()),Some(Vec::new()));
//     }

//     pub fn set_groupart_all (&mut self){
//         self.groupart = Some(Vec::new());
//     }
    
//     pub fn is_all(&self) -> bool {
//         let empty = Some(Vec::new());
//         let empty_g = Some(Vec::<(syn::Ident, Edit)>::new());
//         self.live.0 == true  && self.script.0 == true  &&
//         self.live.1 == empty && self.script.1 == empty &&
//         self.live.2 == empty && self.script.1 == empty &&
//         self.groupart == empty_g
//     } 

//     pub fn is_none(&self) -> bool {

//         self.live.0 == false && self.script.0 == false &&
//         self.live.1 == None  && self.script.1 == None  &&
//         self.live.2 == None  && self.script.2 == None  &&
//         self.groupart == None
//     }  

// }

// impl Default for AGEdit {

//     fn default() -> Self {
//         let script  = (false,None,None);
//         let live    = (false,None,None);
//         let groupart = None;
//         Self { script, live, groupart }
//     } 
// }

/*
    There are two types of arguments 
    
    channel
    lib
    file
    debut

    name(..)
    assoc(..)
    edit(..)
    path(..)

*/


#[derive(Clone)]
pub struct GroupAttributeArguments {

    pub channel :  Channel,
    pub lib     :  Lib,
    pub file    :  Option<PathBuf>,

    pub name    :  BTreeMap<Ident,Ident>,
    pub assoc   :  Option<BTreeMap<Ident,bool>>,
    pub edit    :  EditGroup,
    pub path    :  BTreeMap<Ident,PathBuf>,
    pub allow   :  BTreeMap<Ident,Meta>,
    
    pub members :  BTreeMap<Ident,(ItemImpl,Visibility)>,
}



impl GroupAttributeArguments {


    pub fn parse_nested(&mut self, nested: Punctuated::<syn::Meta,syn::Token![,]>) {
        super::check_path_set(&nested);
        for meta in nested.iter(){

            // CHANNEL
            if meta.path().is_ident("channel"){

                match get_lit(meta) {
                    syn::Lit::Int(val) => { 
                        let value = to_usize(&val);
                        if value > 0 {
                            self.channel = Channel::Buffer(val.clone());
                        }
                    },
                    v => abort!(v, error::error_name_type( &meta.path(), "Int (usize)"),; help=error::AVAIL_ACTOR ),
                }
            }

            // LIB
            else if meta.path().is_ident("lib"){
                let lib_str = get_lit_str(&meta,"lib");
                self.lib = Lib::from(&lib_str);
            }
            
            // FILE
            else if meta.path().is_ident(crate::FILE) {
                let file_str = get_lit_str(&meta,crate::FILE);

                let path = std::path::PathBuf::from(&file_str);

                if path.exists() { self.file = Some(path); }
                else { abort!(meta, format!("Path - {file_str:?} does not exist.")); } 
            }
            
            // NAME
            else if meta.path().is_ident("name"){

                if let Some(meta_list) = get_list( meta,Some(error::AVAIL_GROUP) ) { 
                    super::check_path_set(&meta_list);
                    for met in meta_list {

                        let ident = get_ident_group(&met,"name");
                        let name_str = get_lit_str(&met,"name");
                        // self.name.push((ident,name_str));
                        self.name.insert(ident, format_ident!("{name_str}"));
                    }

                } else { abort!(meta,error::EXPECT_LIST;help=error::AVAIL_GROUP); }
            }

            // ASSOC
            else if meta.path().is_ident("assoc"){

                if let Some(meta_list) = get_list( meta,Some(error::AVAIL_GROUP) ) { 
                    for met in meta_list.iter() {
                        let ident = get_ident_group(met,"assoc");
                        match met {
                            
                            syn::Meta::Path(_) => { 
                                if self.assoc.is_some() {
                                    // self.assoc.as_mut().map(|x| x.push((ident,true)));
                                    self.assoc.as_mut().map(|x| x.insert(ident,true));
            
                                } else { 
                                    // self.assoc = Some(vec![(ident,true)]); 
                                    self.assoc = Some(BTreeMap::from([(ident,true)])); 
                                }
                            },

                            _ => { abort!(meta.path(), error::OLD_ARG_ASSOC); },
                        }
                    }

                } else { self.assoc = Some( BTreeMap::new());  }
                
            }

            // EDIT
            else if meta.path().is_ident(crate::EDIT) { 
                self.edit.parse(&meta);
            }

            // PATH 
            else if meta.path().is_ident("path") { 
                if let Some(meta_list) = get_list( meta,Some(error::AVAIL_GROUP) ) { 
                    super::check_path_set(&meta_list); 
                    for met in meta_list {

                        let ident = get_ident_group(&met,"path");
                        let path_str = get_lit_str(&met,"path");
                        let path = std::path::PathBuf::from(&path_str);

                        if path.exists() { self.path.insert(ident,path); }

                        else { abort!(met, format!("Path - {path_str:?} does not exist.")); } 
                    }

                } else { abort!(meta,error::EXPECT_LIST;help=error::AVAIL_GROUP); }
            }

            else if meta.path().is_ident("allow") { 

                if let Some(meta_list) = get_list( meta,Some(error::AVAIL_GROUP) ) { 
                    super::check_path_set(&meta_list);

                    for met in meta_list {
                        let ident = get_ident(&met);
                        self.allow.insert(ident,meta.clone());
                    }

                } else { abort!(meta,error::EXPECT_LIST;help=error::ABOUT_ALLOW); }
            }

            // UNKNOWN ARGUMENT
            else { error::unknown_attr_arg("group",meta.path() ) }
        }
    }



    pub fn get_vis_ident_ty<'a> (&self, strct: &'a ItemStruct ) -> Vec<(&'a Visibility,&'a Ident,&'a Type)> {

        let mut loc = Vec::new();

        for field in strct.fields.iter() {

            if let Some(ident) = &field.ident {

                let private = match &field.vis {
                    Visibility::Inherited  => true,
                    _ => false,
                };
                let exclude = self.allow.get(ident);
                // if let Some(pos) = allow.iter().position(|x| x.0.eq(ident)){
                //     Some(&allow[pos].1)
                // } else { None };

                match (private,exclude) {
                    (true,Some(met)) => {
                        abort!(met,error::PRIVATE_ALLOW_FIELD;note=error::ABOUT_ALLOW);
                    },
                    (true,None)     => (),
                    (false,Some(_)) => (),
                    (false,None)    => { loc.push((&field.vis,ident,&field.ty)); },
                }
                
            } else { abort!(Span::call_site(),error::TUPLE_STRUCT_NOT_ALLOWED); }
        }
        loc
    }

    // pub fn 
    pub fn cross_check(&mut self,item_impl: &ItemImpl){

        // if there if file
        if let Some(file) = &self.file {

            // check edit 
            if self.edit.is_any_active() {

                match crate::file::active_file_count(file) {
                    Ok(edit_attr) => {
                        self.edit.attr = Some(edit_attr);
                    },
                    Err(e) => { abort!(Span::call_site(),e); },
                }
            }

            // -------

            let (group_ident,_,_)  = crate::model::get_ident_type_generics(item_impl);
            let (i_strct,_) = crate::file::find_group_items(file,&group_ident);
            // may be check for equality  impl 

            let fields = self.get_vis_ident_ty(&i_strct);
            for (vis,ident_field, ty) in fields {
                
                // type identifier
                let ident_ty = match ty {
                    syn::Type::Path(ty_path) => {
                        ty_path.path.segments.last().unwrap().ident.clone()
                    },
                    _ => {
                        let p = quote::quote!{#ty}.to_string();
                        let msg = format!("Expected identifier found : {}.",p);
                        abort!(Span::call_site(),msg;note=error::GROUP_FIELD_TYPE);
                    },
                };

                // member file path 
                let mem_path = if let Some(new_path) = self.path.get(ident_field){
                    new_path
                } else { file };
                // get impl block 
                let (_,i_impl) = crate::file::find_group_items(mem_path,&ident_ty);
                self.members.insert(ident_field.clone(),(i_impl,vis.clone()));
            }

        } else { abort!(Span::call_site(),error::REQ_FILE;help=error::AVAIL_ACTOR); }

    }
    
    /*
    pub channel :  Channel,
    pub lib     :  Lib,
    pub file    :  Option<PathBuf>,

    pub name    :  BTreeMap<Ident,String>,//Vec<(Ident,String)>,
    pub assoc   :  Option<BTreeMap<Ident,bool>>,// Option<Vec<(Ident,bool)>>,
    pub edit    :  EditGroup,
    pub path    :  BTreeMap<Ident,PathBuf>,//Vec<(Ident,PathBuf)>,
    pub allow   :  BTreeMap<Ident,Meta>,//Vec<(Ident,Meta)>,
    
    pub members :  BTreeMap<Ident,(ItemImpl,Visibility)>,
     */



    pub fn get_aaa(&self, slf: Option<&Ident>) -> ActorAttributeArguments {

        let slf = & if let Some(s) = slf { s.clone() } else { format_ident!("self") };

        let mut aaa = ActorAttributeArguments::default();

        aaa.channel = self.channel.clone();
        aaa.lib = self.lib.clone();
        aaa.file = self.file.clone();

        aaa.name = self.name.get(slf).cloned();

        if self.assoc.is_some(){
            self.assoc
                .as_ref()
                .map(|x| 
                    if x.is_empty() { aaa.assoc = true; } 
                    else { if let Some(_) = x.get(slf){ aaa.assoc = true; } }
                );
        }

        if let Some(edt) = &self.edit.edits{
            if let Some(mut edit)  = edt.get(slf).cloned(){
                edit.remove = self.edit.remove;
                edit.attr = self.edit.attr.clone();
                aaa.edit = edit;
            }
        }
        aaa
    }

    // pub fn get_all_actors(&self) -> Vec<crate::model::ActorModelSdpl> {
        // let slf = format_ident!("self");
        // let coll = self.members.keys().filter(|&x| slf)



}

impl Default for GroupAttributeArguments {

    fn default() -> GroupAttributeArguments {

        Self {

            channel :  Channel::default(),
            lib     :  Lib::default(),
            file    :  None,
        
            name    :  BTreeMap::new(),
            assoc   :  None,
            edit    :  EditGroup::default(),
            path    :  BTreeMap::new(),
            allow   :  BTreeMap::new(),
            members :  BTreeMap::new(),
        }
    }
}

