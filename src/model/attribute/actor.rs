use crate::error;
use crate::model::{ 
    Mac,FilterSet,Channel,Lib, ModelReceiver, 
    EditActor,Debut,ShowComment,
    generate_model,get_ident,get_lit,get_lit_str,
    get_list,to_usize };

use std::path::PathBuf;
use proc_macro_error::{abort,abort_call_site};
use quote::format_ident;
use syn::{ Ident,File,ItemImpl,punctuated::Punctuated,Meta,Token};


//-----------------------  ACTOR  

#[derive(Clone)]
pub struct ActorAttributeArguments {

    pub name    :  Option<syn::Ident>,
    pub first_name:  Option<syn::Ident>,
    pub lib     :  Lib,
    pub show    :  ShowComment,
    pub channel :  Channel,
    pub edit    :  EditActor,
    pub debut   :  Debut,
    pub file    :  Option<PathBuf>,
    pub interact:  bool,
    pub filter  :  Option<FilterSet>,
    pub members :  Vec<(Ident,Self)>, 
    pub mod_receiver: ModelReceiver, 
    pub trait_debug : bool,
    pub mac   : Mac,
    /* ADD NEW OPTION */
}




impl ActorAttributeArguments {

    pub fn from( nested: Punctuated::<Meta,Token![,]>, mac: Mac) -> Self 
    {
        let mut aaa = ActorAttributeArguments::default();
        aaa.mac = mac;

        if aaa.mac == Mac::Actor {
            aaa.parse_nested_actor(&nested);
        } else {
            aaa.parse_nested_family(&nested);
        }
        aaa

    }

    pub fn parse_nested_actor<'a,I>(&mut self, nested:I) 
    where I: IntoIterator<Item = &'a Meta> + Clone,
    {

        // check if unique options
        super::check_path_set(nested.clone(),None); 

        for meta in nested.into_iter(){

            let ident = &get_ident(meta);

            if self.parse_shared_options(ident,meta,error::AVAIL_ACTOR){
                continue;
            }

            // FIRST NAME
            else if meta.path().is_ident("first_name")  && self.mac == Mac::Family {
                    let str_name = get_lit_str(&meta,"first_name"); 
                if &str_name == "" { abort!(&ident,"Attribute argument 'first_name' value is empty. Enter a name.")} 
                    self.first_name = Some(format_ident!("{str_name}"));  
            } 
        
            // INTERACT
            else if meta.path().is_ident("interact"){
                match meta {
                    syn::Meta::Path(_) => { self.interact = true; },
                    _ => { abort!(meta, error::EXPECT_IDENT ;help=error::AVAIL_ACTOR) },
                }
            }

            // INCLUDE
            else if meta.path().is_ident("include"){

                if let Some(filter) = &self.filter{
                    match filter {
                        FilterSet::Include(_) => abort!(ident,error::double_decl("include")),
                        FilterSet::Exclude(_) => abort!(ident,error::FILTER_CONURENT_USE_OF_OPTIONS; help=error::FILTER_OPTION_USE_HELP),
                    }
                }
                self.filter = Some(FilterSet::parse(&meta,true));
            }

            // EXCLUDE
            else if meta.path().is_ident("exclude"){

                if let Some(filter) = &self.filter{
                    match filter {
                        FilterSet::Include(_) => abort!(ident,error::FILTER_CONURENT_USE_OF_OPTIONS; help=error::FILTER_OPTION_USE_HELP), 
                        FilterSet::Exclude(_) => abort!(ident,error::double_decl("exclude")),
                    }
                }
                self.filter = Some(FilterSet::parse(&meta,false));
            }

            // ALLOW OVERRIDING CHANNEL FOR FAMILY ACTORS

            // CHANNEL
            else if meta.path().is_ident("channel"){
                match get_lit(meta) {
                    syn::Lit::Int(val) => { 
                        let value = to_usize(&val);
                        if value > 0 { self.channel = Channel::Buffer(val.clone()); }
                    },
                    v => abort!(v, error::error_name_type( &meta.path(), "Int (usize)"),; help=error::AVAIL_ACTOR ),
                }
            }

            // SHOW
            else if meta.path().is_ident("show"){
                match meta {
                    syn::Meta::Path(_) => { self.show.show = true; },
                    _ => { abort!(meta.path(),error::EXPECTED_IDENTIFIER_SHOW )},
                }
            }

            // DEBUG TRAIT
            else if meta.path().is_ident("Debug"){
                self.trait_debug = true;
            }

            else if meta.path().is_ident("debug") {
                abort!(meta,"Did you mean `debut` or `Debug`?"; help=error::AVAIL_ACTOR);
            }

            // UNKNOWN ARGUMENT
            else { error::unknown_attr_arg("actor",meta.path() ) }
        }
    }

    fn parse_nested_family<'a, I>(&mut self,nested: I) 
    where I: IntoIterator<Item = &'a Meta> + Clone,
    {

        let mut family_mems = Vec::new();

        // parse all the shared options 
        // accumulate the model actors    
        super::check_path_set(nested.clone(), Some( vec![ crate::ACTOR]));

        for meta in nested.into_iter(){  

            let ident = &get_ident(meta);

            if self.parse_shared_options(ident,meta,error::AVAIL_FAMILY){ continue; }

            else if meta.path().is_ident(crate::ACTOR) { family_mems.push((meta, Mac::Actor)); }

            // LOCK RwLock
            else if meta.path().is_ident(crate::RWLOCK){
                self.mod_receiver = ModelReceiver::ArcRwLock;
            }

            // LOCK Mutex
            else if meta.path().is_ident(crate::MUTEX){
                self.mod_receiver = ModelReceiver::ArcMutex;
            }

            else if meta.path().is_ident("debug") {
                abort!(meta,"Did you mean `debut`?"; help=error::AVAIL_FAMILY);
            }
            // UNKNOWN ARGUMENT
            else { error::unknown_attr_arg("family",meta.path() ) }
        }

        // check receiver and set to default for `family`
        if self.mod_receiver.is_slf(){
            self.mod_receiver = ModelReceiver::ArcRwLock;
        }

        if !family_mems.is_empty() {
            let mut proto = self.clone();
            // clean 
            proto.show = ShowComment::default();
            proto.edit = EditActor::default();

            for (mem,mac) in family_mems {

                let mut other = proto.clone();
                
                if let Some(mem_nested) = get_list(mem, None){
                    other.parse_nested_actor(&mem_nested);
                    if let Some(first_name) = other.first_name.clone() {
                        
                        other.mac = mac;
                        // createa a member 
                        self.members.push((first_name, other));

                    } else {  abort!( mem, "argument `first_name` undefined"; help=error::AVAIL_FAMILY ) }

                } else { abort!( mem, "expected a list of arguments"; help=error::AVAIL_FAMILY ) }
            }
        } else { abort_call_site!( "family is a wrapper"; help=error::AVAIL_FAMILY ) }

    }


    fn parse_shared_options(&mut self, ident: &Ident, meta: &Meta, avail_error: &'static str ) -> bool{

        // NAME
        if meta.path().is_ident("name"){
            let str_name = get_lit_str(&meta,"name"); 
            if &str_name == "" { abort!(&ident,"Attribute argument 'name' value is empty. Enter a name.") }
            self.name = Some(format_ident!("{str_name}")); 
            true
        }

        // LIB
        else if meta.path().is_ident("lib"){
            let lib_str = get_lit_str(&meta,"lib");
            self.lib = Lib::from(&lib_str);
            true
        }

        // SHOW
        else if meta.path().is_ident("show"){
            match meta {
                syn::Meta::Path(_) => { self.show.show = true; },
                _ => { abort!(meta.path(),error::EXPECTED_IDENTIFIER_SHOW )},
            }
            true
        }

        // CHANNEL
        else if meta.path().is_ident("channel"){
            match get_lit(meta) {
                syn::Lit::Int(val) => { 
                    let value = to_usize(&val);
                    if value > 0 { self.channel = Channel::Buffer(val.clone()); }
                },
                v => abort!(v, error::error_name_type( &meta.path(), "Int (usize)"); help=avail_error ),
            }
            true
        }

        // EDIT
        else if meta.path().is_ident(crate::EDIT){
            if self.mac == Mac::Family {
                self.edit.parse_family(&meta);
            } else {
                self.edit.parse(&meta);
            }
            true
        }

        // DEBUT
        else if meta.path().is_ident("debut"){
            match meta {
                syn::Meta::Path(_) => { self.debut.active = true; },
                _ => { abort!(meta, error::EXPECT_IDENT ;help=error::AVAIL_DEBUT) },
            }
            true
        }

        // FILE
        else if meta.path().is_ident(crate::FILE) {
            self.file = Some( super::meta_get_path(meta));
            true 
        }

        else { false }
    }

    pub fn cross_check(&mut self){
        // file count 
        if self.is_active(){
            if let Some(file_path) = &self.file {
                match crate::file::active_file_count(file_path) {
                    Ok(mut edit_attr) => {
                        // set the remove 
                        if self.mac == Mac::Family {
                            edit_attr.remove = self.members.iter().all(|(_,x)| x.edit.remove == true );
                        } else {
                            edit_attr.remove = self.edit.remove;
                        }
                        self.edit.attr = Some(edit_attr);
                    },
                    Err(e) => { abort_call_site!(e); },
                }
            } else { abort_call_site!(error::REQ_FILE;help=error::AVAIL_ACTOR); }
        }
        // 'family' not supported for 'smol'
        if self.mac == Mac::Family  &&  self.lib == Lib::Smol {
            abort_call_site!(error::NOT_ALLOW_FAMILY_IN_SMOL);
        } 
    }

    fn is_active(&self) -> bool {
        if self.edit.is_any_active() { return true;}
        if self.mac == Mac::Family {
            for (_,aaa) in  &self.members {
                if aaa.is_active() { return true; }
            }
        }
        false
    }

    pub fn generate_example_code( mut self, item_impl: &ItemImpl)  -> (File,File){

        // show off
        if self.mac == Mac::Family {
            for (_, mem) in self.members.iter_mut() {
                mem.show.show = false;
            }
        }
        self.show.show = false;

        let mut model_sdpl = generate_model(self, item_impl );
        let (code,edit) = model_sdpl.get_code_edit();
        (code,edit)
    } 

} 

impl Default for ActorAttributeArguments {

    fn default() -> ActorAttributeArguments {

        Self { 
            name    : None,
            first_name: None,
            lib     : Lib::default(),
            show    : ShowComment::default(),
            channel : Channel::default(),
            edit    : EditActor::default(),
            debut   : Debut::default(),
            file    : None,
            interact: false,
            filter  : None,
            members : Vec::new(),
            mod_receiver: ModelReceiver::default(),
            trait_debug : false,
            mac     :  Mac::Actor,
            /* ADD NEW ATTRIBUTE */
        }  
    }
}


