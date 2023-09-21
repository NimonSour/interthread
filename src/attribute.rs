use crate::error;
use crate::file::get_ident;

use std::path::PathBuf;
use proc_macro2::Span;
use proc_macro_error::abort;
use quote::format_ident;


// #[derive(Debug, Eq, PartialEq, Clone, Copy)]
// pub enum Either<L, R> {
//     L(L),
//     R(R),
// }

fn to_usize(value: &syn::LitInt) -> usize {
        
    let msg  = format!("Expected a positive integer 1..{:?}.", usize::MAX );
    value.base10_parse::<usize>()
         .unwrap_or_else(|_| abort!(value,msg))   
} 

//-----------------------  EXAMPLE 
#[derive(Debug, Eq, PartialEq)]
pub struct ExampleAttributeArguments {

    pub path     : Option<std::path::PathBuf>,
    pub main     :                       bool,
    pub expand   :              Vec<AAExpand>,  
    /* ADD NEW OPTION */ 
}

impl Default for ExampleAttributeArguments {

    fn default() -> Self {

        let path  = None ;
        let main             = false;
        let expand  = vec![AAExpand::Actor, AAExpand::Group];
        /* ADD NEW OPTION */ 

        Self { path, main, expand }
    }
}

impl ExampleAttributeArguments {

    pub fn parse(&mut self, meta: syn::meta::ParseNestedMeta) -> Result<(), syn::Error> {

        let mut parse_macro_arguments = |meta: syn::meta::ParseNestedMeta| { 
            
            // PATH
            if meta.path.is_ident("path") {

                let value = meta.value()?.parse::<syn::Lit>()?;

                match value.clone() {
                    syn::Lit::Str(val) => {

                        // the path needs to be checked first 
                        let path = std::path::PathBuf::from(val.value());

                        if path.exists() {
                            self.path = Some(path);
                            return Ok(());
                        }
                        else {
                            abort!(val, format!("Path - {:?} does not exists.",val.value())); 
                        } 
                    },
                    _ => {
                        let name = meta.path.get_ident().unwrap();
                        return Err( meta.error(format!("Expected a  'str'  value for argument '{}'.", name.to_string() )));
                    },
                }
            }

            // EXPAND
            else if meta.path.is_ident("expand") {
                self.expand = vec![];
                return meta.parse_nested_meta(|meta| {

                    if meta.path.is_ident("actor"){
                        self.expand.push(AAExpand::Actor);
                        Ok(())
                    }
                    else if meta.path.is_ident("group"){
                        self.expand.push(AAExpand::Group);
                        Ok(())
                    }
                    else {
                        let arg  = meta.path.get_ident().unwrap();
                        let msg  = format!("Unknown 'expand' option  -  {:?} .", arg.to_string());
                        abort!(arg, msg; help=error::AVAIL_EXPAND);
                    }
                });
            }
            else if meta.path.is_ident("file") {
                let value = meta.value()?.parse::<syn::Lit>()?;
                match value.clone() {
                    syn::Lit::Str(val) => {
                        let p = val.value();
                        return Err( meta.error(crate::error::old_file_arg(p)));
                    },
                    _ => {
                        let name = meta.path.get_ident().unwrap();
                        return Err( meta.error(format!("Expected a  'str'  value for argument '{}'.", name.to_string() )));
                    },
                }
            }
            else {
                let ident  = meta.path.get_ident().unwrap();
                error::unknown_attr_arg("example", ident);
                Ok(())
            }
        };


        //MAIN
        if meta.path.is_ident("main"){
            self.main = true;
            if let Err(e) = meta.parse_nested_meta(parse_macro_arguments ){
                return Err(e);
            }
            self.arguments_cross_check()
        }

        //MOD
        else if meta.path.is_ident("mod") {
            if let Err(e) = meta.parse_nested_meta(parse_macro_arguments ){
                return Err(e);
            }
            self.arguments_cross_check()
        }

        // NONE or UNKNOWN
        else {
            if let Err(e) = parse_macro_arguments(meta){
                return Err(e);
            }
            self.arguments_cross_check()
        }
    }

    pub fn arguments_cross_check(&self) -> Result<(),syn::Error>{

        if  self.path.is_none() {
            let msg = "Expected a 'path' argument with a path to a file.  file=\"path/to/file.rs\"";
            abort!(Span::call_site(), msg )
        }
        Ok(())
    }

    pub fn get_path(&mut self) -> std::path::PathBuf {
        self.path.clone().unwrap()
    }
}

//-----------------------  EXAMPLE EXPAND
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum AAExpand {
    Actor,
    Group,
}

impl AAExpand{

    pub fn to_str(&self) -> &'static str {

        match self {
            Self::Actor => crate::ACTOR,
            Self::Group => crate::GROUP,
        }
    }
}

//-----------------------  ACTOR LIB

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum AALib {
    Std,
    Smol,
    Tokio,
    AsyncStd,
}

impl AALib {

    pub fn from( s: &syn::LitStr  ) -> Self {

        match s.value() {

            val if val == "std".to_string()       =>  AALib::Std,
            val if val == "smol".to_string()      =>  AALib::Smol,
            val if val == "tokio".to_string()     =>  AALib::Tokio,
            val if val == "async_std".to_string() =>  AALib::AsyncStd,
            val => {
                let msg = format!("Unknown option  -  {:?} for 'channel' ", val);
                abort!( s, msg; help=error::AVAIL_LIB );   
            } 
        }
    }
}

impl Default for AALib {
    fn default() -> Self {
        AALib::Std
    }
}

//-----------------------  ACTOR CHANNEL 

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum AAChannel {

    Unbounded,
    Buffer(syn::LitInt),
}

impl Default for AAChannel {
    fn default() -> Self {
        AAChannel::Unbounded
    }
}

//-----------------------  ACTOR EDIT 
#[derive(Debug, Eq, PartialEq, Clone)]

pub struct AAEdit {
    pub script:( bool, Option<Vec<syn::Ident>>, Option<Vec<syn::Ident>> ),
    pub live:  ( bool, Option<Vec<syn::Ident>>, Option<Vec<syn::Ident>> ),
}
impl AAEdit {

    pub fn parse_nested(&mut self, nested: syn::punctuated::Punctuated::<syn::Meta,syn::Token![,]>, sol: bool ){
        let (name,mut strct) = 
        if sol {("script",self.script.clone()) } else { ("live",self.live.clone())};

        for meta in nested.iter() {

            if meta.path().is_ident("def"){
                strct.0 = true;
            }
            else if meta.path().is_ident("imp"){

                if let Some(list) = get_list(meta, error::AVAIL_EDIT) {
                    strct.1 = Some(list.iter().filter_map(|x| get_ident(x)).collect::<Vec<_>>());
                } else {
                    strct.1 = Some( Vec::new());
                }
            }
            
            else if meta.path().is_ident("trt"){
    
                if let Some(list) = get_list(meta, error::AVAIL_EDIT) {
                    strct.2 = Some(list.iter().filter_map(|x| get_ident(x)).collect::<Vec<_>>());
                } else {
                    strct.2 = Some( Vec::new());
                }
            }
            else {
                let msg = format!("Unsuported 'edit({}( ? ))' option! Expected options are `def`,`imp` or `trt` .",name);
                abort!(meta, msg);
            }
        }

        if sol { self.script = strct; } else { self.live = strct; };

    }

    pub fn set_live_all(&mut self){
        self.live = (true,Some(Vec::new()),Some(Vec::new()));
    }

    pub fn set_script_all (&mut self){
        self.script = (true,Some(Vec::new()),Some(Vec::new()));
    }

    pub fn is_all(&self) -> bool {
        let empty = Some(Vec::new());

        self.live.0 == true  && self.script.0 == true  &&
        self.live.1 == empty && self.script.1 == empty &&
        self.live.2 == empty && self.script.1 == empty
    } 

    pub fn is_none(&self) -> bool {

        self.live.0 == false && self.script.0 == false &&
        self.live.1 == None  && self.script.1 == None  &&
        self.live.2 == None  && self.script.2 == None
    }  

}

impl Default for AAEdit {

    fn default() -> Self {
        let script  = (false,None,None);
        let live    = (false,None,None);
        Self { script, live }
    } 
}


//-----------------------  ACTOR FILE
#[derive(Debug, Eq, PartialEq, Clone)]

pub struct AAFile {
    pub path:              PathBuf,
    pub attr:       syn::Attribute,
    pub attrs: Vec<syn::Attribute>,
}




//-----------------------  ACTOR  

#[derive(Debug,Clone, Eq, PartialEq)]
pub struct ActorAttributeArguments {

    pub name    :  Option<syn::Ident>,
    pub lib     :  AALib,
    pub assoc   :  bool,
    pub channel :  AAChannel,
    pub edit    :  AAEdit,
    pub id      :  bool,
    pub file    :  Option<AAFile>,
    /* ADD NEW OPTION */
}


impl Default for ActorAttributeArguments {

    fn default() -> ActorAttributeArguments {

        Self { 
            name   : None,
            lib    : AALib::default() ,
            assoc  : false,
            channel: AAChannel::default(),
            edit   : AAEdit::default() ,
            id     : false,
            file   : None,
            /* ADD NEW ATTRIBUTE */
        }  
    }
}


impl ActorAttributeArguments {
       
    pub fn parse_nested(&mut self, nested: syn::punctuated::Punctuated::<syn::Meta,syn::Token![,]>) {

        for meta in nested.iter(){

            if let Some(ident) = get_ident(meta) {

                // NAME
                if meta.path().is_ident("name"){

                    match get_lit(meta) {
                        syn::Lit::Str(val) => {  
                            let str_name = val.value();

                            if str_name == "".to_string() {
                                abort!(&ident,"Attribute field 'name' is empty. Enter a name.") 
                            }
                            else {
                                self.name = Some(format_ident!("{}",val.value()));
                            } 
                        },
                        v => abort!(v, error::error_name_type( &ident, "str"); help=error::AVAIL_ACTOR ),
                    }
                }


                // LIB
                else if meta.path().is_ident("lib"){

                    match get_lit(meta) {
                        syn::Lit::Str(val) => {

                            self.lib = AALib::from(&val);
                        },
                        v => abort!(v, error::error_name_type( &ident, "str"); help=error::AVAIL_ACTOR ),
                    }
                }

                // ASSOC
                else if meta.path().is_ident("assoc"){

                    match meta {
                        syn::Meta::Path(_) => { self.assoc = true; },
                        _ => {
                            match get_lit(meta) {
                                syn::Lit::Bool(val) => { self.assoc = val.value(); },
                                v => abort!(v, error::error_name_type( &ident, "bool"); help=error::AVAIL_ACTOR ),
                            }
                        },
                    }
                }


                // CHANNEL
                else if meta.path().is_ident("channel"){

                    match get_lit(meta) {
                        syn::Lit::Int(val) => { 
                            let value = to_usize(&val);
                            if value > 0 {
                                self.channel = AAChannel::Buffer(val.clone());
                            }
                        },
                        v => abort!(v, error::error_name_type( &ident, "Int (usize)"),; help=error::AVAIL_ACTOR ),
                    }
                }


                // EDIT
                else if meta.path().is_ident("edit"){
                    

                    if let Some(meta_list) = get_list( meta,error::AVAIL_EDIT ) {

                        for edit_meta in meta_list.iter() {

                            if edit_meta.path().is_ident("script"){
    
                                if let Some(list) = get_list(edit_meta,error::AVAIL_EDIT ){

                                    self.edit.parse_nested(list,true);

                                } else {
                                    self.edit.set_script_all();
                                }
                            } 

                            else if edit_meta.path().is_ident("live"){
                                
                                if let Some(list) = get_list(edit_meta,error::AVAIL_EDIT ){

                                    self.edit.parse_nested(list,false);

                                } else {
                                    self.edit.set_live_all();
                                }
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
                                abort!(edit_meta,"Unknown 'edit' option!";help=error::AVAIL_EDIT );
                            } 
                        }
                    } else {
                        self.edit.set_script_all();
                        self.edit.set_live_all();
                    }

                }

                // ID
                else if meta.path().is_ident("id"){
                    match meta {
                        syn::Meta::Path(_) => { self.id = true; },
                        _ => {
                            match get_lit(meta) {
                                syn::Lit::Bool(val) => { self.id = val.value(); },
                                v => abort!(v, error::error_name_type( &ident, "bool"); help=error::AVAIL_ACTOR ),
                            }
                        }
                    }
                }

                // FILE
                else if meta.path().is_ident("file") {

                    let value = get_lit(meta);

                    match value.clone() {
                        syn::Lit::Str(val) => {

                            // the path needs to be checked first 
                            let path = std::path::PathBuf::from(val.value());

                            if path.exists() {
                                // one only check 
                                match crate::file::macro_file_count(&path) {
                                    Ok((attr,attrs)) => {
                                        self.file = Some(AAFile {
                                                                    path: path.clone(),
                                                                    attr,
                                                                    attrs });
                                    },
                                    Err(e) => { abort!(value,e); },
                                }
                            }
                            else {
                                abort!(val, format!("Path - {:?} does not exists.",val.value())); 
                            } 
                        },
                        _ => { abort!(value, error::error_name_type( &ident, "str"); help=error::AVAIL_ACTOR ) },
                    }
                }

                // UNKNOWN ARGUMENT
                else {
                    error::unknown_attr_arg("actor",&ident )
                }
            } else { 
                abort!(meta,"Unknown configuration option!"; help=error::AVAIL_ACTOR); 
            }

        }


    }


    pub fn cross_check(&self){

        if self.file.is_some() {
            if self.edit.is_none(){
                let msg = "Expected an `edit` argument!";
                abort!(Span::call_site(),msg;help=error::AVAIL_EDIT);
            }
        }
    }
} 


pub fn get_list(meta: & syn::Meta, help: &str) -> Option<syn::punctuated::Punctuated::<syn::Meta,syn::Token![,]>> {
    match meta {
        syn::Meta::Path(_) => { None },
        syn::Meta::List(meta_list) => { 
            let list = 
            meta_list.parse_args_with(syn::punctuated::Punctuated::<syn::Meta,syn::Token![,]>::parse_terminated).unwrap();
            Some(list) 
        },
        syn::Meta::NameValue(_) => { abort!(meta,"Expected a list!"; help=help) },
    }
}

pub fn get_lit( meta: &syn::Meta ) -> syn::Lit {

    let msg = "Expected a 'name = value' argument !";
    match meta {
        syn::Meta::NameValue(nv) => {
            match &nv.value {
                syn::Expr::Lit(expr_lit) => {
                    expr_lit.lit.clone()
                    
                },
                v => abort!(v, msg),
            }
        },
        m => abort!(m, msg),
    }
}





