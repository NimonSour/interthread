use crate::error;
use crate::model::{Generics, Channel,Lib,
    EditGroup,get_ident,get_ident_group,get_lit,get_lit_str,
    get_list,to_usize, check_name_conflict};


use std::path::PathBuf;
use proc_macro2::Span;
use proc_macro_error::abort;
use quote::format_ident;
use syn::{Ident,ItemStruct,ItemImpl,Visibility,Type,punctuated::Punctuated,Meta};
use std::collections::BTreeMap;

use super::ActorAttributeArguments;
use crate::model::Debut;


#[derive(Clone)]
pub struct GroupAttributeArguments {

    pub channel :  Channel,
    pub lib     :  Lib,
    pub file    :  Option<PathBuf>,
    pub debut   :  Debut,

    pub name    :  BTreeMap<Ident,Ident>,
    pub assoc   :  Option<BTreeMap<Ident,bool>>,
    pub interact:  Option<BTreeMap<Ident,bool>>,
    pub edit    :  EditGroup,
    pub path    :  BTreeMap<Ident,PathBuf>,
    pub allow   :  BTreeMap<Ident,Meta>,
    
    pub members :  BTreeMap<Ident,(ItemImpl,Visibility,Type,Generics)>,
    pub def_generics: Generics,
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
                    v => abort!(v, error::error_name_type( &meta.path(), "Int (usize)"),; help=error::AVAIL_GROUP ),
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
            // DEBUT
            else if meta.path().is_ident("debut"){

                if let Some(meta_list) = get_list( meta,Some(error::AVAIL_DEBUT) ) {

                    for m in meta_list {
                        if m.path().is_ident("legend"){
                            match m {
                                syn::Meta::Path(_) => { self.debut.legend = Some(true); },
                                _ => { abort!(meta.path(),"Expected an identifier.";help=error::AVAIL_DEBUT); },
                            } 
                        } else {
                            let msg = "Unknown option for argument 'debut'.";
                            abort!(m,msg;help=error::AVAIL_DEBUT);
                        }
                    }
                } else {  self.debut.legend = Some(false);  }
            }
            
            // NAME
            else if meta.path().is_ident("name"){

                if let Some(meta_list) = get_list( meta,Some(error::AVAIL_GROUP) ) { 
                    super::check_path_set(&meta_list);
                    for met in meta_list {

                        let ident = get_ident_group(&met,"name");
                        let name_str = get_lit_str(&met,"name");
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
                                    self.assoc.as_mut().map(|x| x.insert(ident,true));
            
                                } else { 
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

            // INTERACT

            else if meta.path().is_ident("interact"){

                if let Some(meta_list) = get_list( meta,Some(error::AVAIL_GROUP) ) { 
                    for met in meta_list.iter() {
                        let ident = get_ident_group(met,"interact");
                        match met {
                            syn::Meta::Path(_) => { 
                                if self.interact.is_some() {
                                    self.interact.as_mut().map(|x| x.insert(ident,true));
            
                                } else { self.interact = Some(BTreeMap::from([(ident,true)])); }
                            },
                            _ => { abort!(meta.path(), error::expected_path_ident( "interact" )) },
                        }
                    }
                } else { self.interact = Some( BTreeMap::new());  }
                
            }

            // ALLOW
            else if meta.path().is_ident("allow") { 

                if let Some(meta_list) = get_list( meta,Some(error::AVAIL_GROUP) ) { 
                    super::check_path_set(&meta_list);

                    for met in meta_list {
                        let ident = get_ident(&met);
                        self.allow.insert(ident,meta.clone());
                    }

                } else { abort!(meta,error::EXPECT_LIST;help=error::ABOUT_ALLOW); }
            }

            else if meta.path().is_ident("debug") {
                abort!(meta,"Did you mean `debut`?"; help=error::AVAIL_ACTOR);
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

        // checking now to make sure they are different 
        let idents = loc.iter().map(|&(_,i,_)| i).collect::<Vec<_>>();
        check_name_conflict(idents);
        loc
        
    }

    pub fn impl_eq( mut this: ItemImpl, other: &ItemImpl ){
        this.attrs.clear();
        let mut other = other.clone();
        other.attrs.clear();
        if !this.eq(&other){
            abort!(Span::call_site(),error::MISMATCHED_IMPL_BLOCK)
        }
    }

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

            let (group_ident,_,_) = crate::model::get_ident_type_generics(item_impl);
            let (i_strct,i_impl)  = crate::file::find_group_items(file,&group_ident);
            
            // check for equality  impl 
            Self::impl_eq(i_impl,item_impl);

            // definition generics
            self.def_generics = i_strct.generics.clone();

            let fields = self.get_vis_ident_ty(&i_strct);
            for (vis,ident_field, ty) in fields {
                
                // type identifier
                let path_seg = match ty {
                    syn::Type::Path(ty_path) => {
                        ty_path.path.segments.last().unwrap().clone()
                    },
                    _ => {
                        let p = quote::quote!{#ty}.to_string();
                        let msg = format!("Expected identifier found : {}.",p);
                        abort!(Span::call_site(),msg;note=error::GROUP_FIELD_TYPE;help=error::ABOUT_ALLOW);
                    },
                };

                // member file path 
                let mem_path = if let Some(new_path) = self.path.get(ident_field){
                    new_path
                } else { file };
                // get impl block 
                let (i_strct,i_impl) = crate::file::find_group_items(mem_path,&path_seg.ident);
                let def_gen = i_strct.generics.clone();
                self.members.insert(ident_field.clone(),(i_impl,vis.clone(),ty.clone(),def_gen));
            }

        } else { abort!(Span::call_site(),error::REQ_FILE;help=error::AVAIL_ACTOR); }

    }


    pub fn get_aaa(&self, fld: Option<&Ident>) -> ActorAttributeArguments {

        let mut aaa = ActorAttributeArguments::default();
        let slf = & if let Some(s) = fld { s.clone() } else { format_ident!("self") };

        aaa.channel = self.channel.clone();
        aaa.lib = self.lib.clone();
        aaa.file = self.file.clone();
        if fld.is_none(){ aaa.debut = self.debut.clone();}

        aaa.name = self.name.get(slf).cloned();
        aaa.path = self.path.get(slf).cloned();

        if self.assoc.is_some(){
            self.assoc
                .as_ref()
                .map(|x| 
                    if x.is_empty() { aaa.assoc = true; } 
                    else { if x.get(slf).is_some(){ aaa.assoc = true; } }
                );
        }

        if self.interact.is_some(){
            self.interact.as_ref()
                .map(|x| 
                    if x.is_empty() { aaa.interact = true; } 
                    else { if x.get(slf).is_some(){ aaa.interact = true; } }
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


}

impl Default for GroupAttributeArguments {

    fn default() -> GroupAttributeArguments {

        Self {

            channel :  Channel::default(),
            lib     :  Lib::default(),
            file    :  None,
            debut   :  Debut::default(),

            name    :  BTreeMap::new(),
            assoc   :  None,
            interact:  None,
            edit    :  EditGroup::default(),
            path    :  BTreeMap::new(),
            allow   :  BTreeMap::new(),
            members :  BTreeMap::new(),
            def_generics: Generics::default(),
        }
    }
}

