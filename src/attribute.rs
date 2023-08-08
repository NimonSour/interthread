use crate::error;
use crate::use_macro;

use std::path::PathBuf;
use proc_macro2::Span;
use proc_macro_error::abort;
use quote::{quote,format_ident};


#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Either<L, R> {
    L(L),
    R(R),
}

fn to_usize(value: syn::LitInt) -> usize {
        
    let msg  = format!("Expected a positive integer 1..{:?}.", usize::MAX );
    value.base10_parse::<usize>()
         .unwrap_or_else(|_| abort!(value,msg))   
} 

//-----------------------  EXAMPLE 
#[derive(Debug, Eq, PartialEq)]
pub struct ExampleAttributeArguments {

    pub file     : Option<std::path::PathBuf>,
    pub main     :                       bool,
    pub expand   :              Vec<AAExpand>,  
    /* ADD NEW OPTION */ 
}

impl Default for ExampleAttributeArguments {

    fn default() -> Self {

        let file  = None ;
        let main             = false;
        let expand  = vec![AAExpand::Actor, AAExpand::Group];
        /* ADD NEW OPTION */ 

        Self { file, main, expand }
    }
}

impl ExampleAttributeArguments {

    pub fn parse(&mut self, meta: syn::meta::ParseNestedMeta) -> Result<(), syn::Error> {

        let mut parse_macro_arguments = |meta: syn::meta::ParseNestedMeta| { 
            
            // PATH
            if meta.path.is_ident("file") {

                let value = meta.value()?.parse::<syn::Lit>()?;

                match value.clone() {
                    syn::Lit::Str(val) => {

                        // the path needs to be checked first 
                        let path = std::path::PathBuf::from(val.value());

                        if path.exists() {
                            self.file = Some(path);
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
            else {
                let ident  = meta.path.get_ident().unwrap();
                error::unknown_attr_arg("example", ident);
                Ok(())
            }
        };


        //MAIN
        if meta.path.is_ident("main"){
            self.main = true;
            let _ = meta.parse_nested_meta(parse_macro_arguments );
            self.arguments_cross_check()
        }

        //MOD
        else if meta.path.is_ident("mod") {
            let _ = meta.parse_nested_meta(parse_macro_arguments );
            self.arguments_cross_check()
        }

        // NONE or UNKNOWN
        else {
            let _ = parse_macro_arguments(meta);
            self.arguments_cross_check()
        }
    }

    pub fn arguments_cross_check(&self) -> Result<(),syn::Error>{

        if  self.file.is_none() {
            let msg = "Expected a 'file' argument with a path to a file.  file=\"path\\to\\file.rs\"";
            abort!(Span::call_site(), msg )
        }
        Ok(())
    }

    pub fn get_file(&mut self) -> std::path::PathBuf {

        let file = self.file.clone().unwrap();
        file
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

            val if val == "std".to_string()       =>   AALib::Std,
            val if val == "smol".to_string()      =>   AALib::Smol,
            val if val == "tokio".to_string()     =>   AALib::Tokio,
            val if val == "async_std".to_string() =>   AALib::AsyncStd,
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
    Inter,
}

impl AAChannel {

    pub fn from( arg: Either<syn::LitStr,syn::LitInt> ) -> Self {
    
        match arg {

            Either::L( s) => {

                match s.value() {
        
                    val if val == "unbounded".to_string()   => return AAChannel::Unbounded,
                    val if val == "inter".to_string()       => return AAChannel::Inter,
                    val => {
                        let msg = format!("Unknown option  -  {:?} for 'channel' ", val);
                        abort!( s, msg; help=error::AVAIL_CHANNEL );
                    },
                }
            },

            Either::R( i) => {
                
                let value = to_usize(i.clone());

                if value == 0 { 
                    return AAChannel::Unbounded; 
                } 
                else { 
                    return AAChannel::Buffer(i);
                }
            },
        }
    }
}

impl Default for AAChannel {
    fn default() -> Self {
        AAChannel::Inter
    }
}

//-----------------------  ACTOR EDIT 
#[derive(Debug, Eq, PartialEq, Clone)]

pub struct AAEdit {
    pub live:  ( bool, Option<Vec<syn::Ident>>, Option<Vec<syn::Ident>> ),
    pub script:( bool, Option<Vec<syn::Ident>>, Option<Vec<syn::Ident>> ),
}

impl Default for AAEdit {
    fn default() -> Self {

        let script  = (false,None,None);
        let live    = (false,None,None);

        Self { script, live }
    }
}




//-----------------------  ACTOR  

#[derive(Debug, Eq, PartialEq)]
pub struct ActorAttributeArguments {

    pub name    :  Option<syn::Ident>,
    pub lib     :  AALib,
    pub assoc   :  bool,
    pub channel :  AAChannel,
    pub edit    :  AAEdit,
    pub id      :  bool,
    pub file    :  Option<PathBuf>,
    /* ADD NEW OPTION */
}


impl Default for ActorAttributeArguments {

    fn default() -> ActorAttributeArguments {

        Self { 
            name   : None,
            lib    : AALib::default() ,
            assoc  : true,
            channel: AAChannel::default(),
            edit   : AAEdit::default() ,
            id     : true,
            file   : None,
            /* ADD NEW ATTRIBUTE */
        }  
    }
}

/*
#[derive(Debug, Eq, PartialEq)]
pub struct ParseActorAttributeArguments {

   pub name           : ( Option<syn::LitStr>    , Option<syn::Ident> ),
   pub lib            : ( Option<syn::LitStr>    , AALib              ),
   pub assoc          : ( Option<syn::Ident>     , bool               ),
   pub channel        : ( Option<syn::Lit>       , AAChannel          ),
   pub edit           :                                          AAEdit,
   pub id             : ( Option<syn::Ident>     , bool               ),
   pub file           : ( Option<syn::Lit>, Option<std::path::PathBuf>),
}

impl Default for ParseActorAttributeArguments {

    fn default() -> ParseActorAttributeArguments {

        let name =  ( None, None);
        let lib          =  ( None, AALib::default() );
        let assoc          =  ( None, false);
        let channel     =  ( None, AAChannel::default());
        let edit                          =  AAEdit::default() ;
        let id             =  ( None, false);
        let file  = (None,None);
        ParseActorAttributeArguments { 

                     name,
                      lib,  
                    assoc,  
                  channel,
                     edit,
                       id         
        }  
    }
}
*/

// impl ParseActorAttributeArguments {

impl ActorAttributeArguments {
    
    /*
    
    pub fn get_arguments(&mut self) -> ActorAttributeArguments {

        ActorAttributeArguments { 

                  name: self.name.1.clone(),
                   lib: self.lib.1.clone(),  
                 assoc: self.assoc.1.clone(),  
               channel: self.channel.1.clone(),
                  edit: self.edit.clone(),
                    id: self.id.1.clone()
        } 
    }
     */


    pub fn parse(&mut self, meta: syn::meta::ParseNestedMeta) -> Result<(), syn::Error> {

        
        if let Some(ident) = meta.path.get_ident() {

            // NAME
            if meta.path.is_ident("name"){

                let  value = meta.value()?.parse::<syn::Lit>()?;
                match value.clone() {
                    syn::Lit::Str(val) => {  
                        // self.name.0 = Some(val.clone());
                        let str_name = val.value();

                        if str_name == "".to_string() {
                            abort!(ident,"Attribute field 'name' is empty. Enter a name.") 
                        }
                        else {
                            self.name = Some(format_ident!("{}",val.value()));
                        } 
                        return Ok(());
                    },
                    v => abort!(v, error::error_name_type( ident.clone(), "str".into()); help=error::AVAIL_ACTOR ),
                }
            }

            // LIB
            else if meta.path.is_ident("lib"){

                let  value = meta.value()?.parse::<syn::Lit>()?;

                match value.clone() {
                    syn::Lit::Str(val) => {
                        // self.lib.0 = Some(val.clone()); 
                        self.lib = AALib::from(&val);
                        return Ok(());
                    },
                    v => abort!(v, error::error_name_type( ident.clone(), "str".into()),; help=error::AVAIL_ACTOR ),
                }
            }

            // STATIC
            else if meta.path.is_ident("assoc"){
                if meta.input.clone().to_string().is_empty() {
                    // self.assoc.0 = Some(ident.clone());
                    self.assoc = true;
                } else {
                    let  value = meta.value()?.parse::<syn::Lit>()?;
                    match value.clone() {
                        syn::Lit::Bool(val) => { 
                            // self.assoc.0 = Some(ident.clone());
                            self.assoc = val.value();
                            return Ok(());
                        },
                        v => abort!(v, error::error_name_type( ident.clone(), "bool".into()); help=error::AVAIL_ACTOR ),
                    }
                }
            }
          
            // CHANNEL
            else if meta.path.is_ident("channel"){

                let  value = meta.value()?.parse::<syn::Lit>()?;

                    // self.channel.0 = Some(value.clone());

                match value {
                    syn::Lit::Int(val) => { 
                        self.channel = AAChannel::from(Either::R(val));
                    },
                    syn::Lit::Str(val) => {

                        self.channel = AAChannel::from(Either::L(val));
                    },
                    v => abort!(v, error::error_name_type( ident.clone(), "int | str".into()),; help=error::AVAIL_ACTOR ),
                }
                return Ok(());
            }

            // EDIT
            else if meta.path.is_ident("edit"){
                
                if meta.input.clone().to_string().is_empty() {
                    //if ident 
                    self.edit.script.0 = true;
                    self.edit.script.1 = Some(Vec::new());
                    self.edit.script.2 = Some(Vec::new());
                    self.edit.live.0   = true;
                    self.edit.live.1   = Some(Vec::new());
                    self.edit.live.2   = Some(Vec::new());

                } 
                else { 

                    match meta.parse_nested_meta( |meta| {

                        if meta.path.is_ident("script") {
                            // if ident 
                            if meta.input.clone().to_string().is_empty() {
                                self.edit.script.0 = true;
                                self.edit.script.1 = Some(Vec::new());
                                self.edit.script.2 = Some(Vec::new());
                                return Ok(());
                            } else {
                                meta.parse_nested_meta(|meta|{
                             
                                    if meta.path.is_ident("def"){
                                        self.edit.script.0 = true;
                                        return Ok(());
                                    }

                                    else if meta.path.is_ident("imp"){
                                        // if ident
                                        if meta.input.clone().to_string().is_empty() {
                                            self.edit.script.1 = Some( Vec::new());
                                            return Ok(());
                                        } else {

                                            meta.parse_nested_meta(|meta| {
                                                if let Some(ident) = meta.path.get_ident(){
                                                    if self.edit.script.1.is_some(){
                                                        self.edit.script.1.as_mut().map(|x| x.push(ident.clone()));
                                                        return Ok(());
                                                    } else {
                                                        self.edit.script.1 = Some( Vec::from([ident.clone()]));
                                                        return Ok(());
                                                    }
                                                } else {
                                                    return Err(meta.error("Unsuported 'edit(script(impl(?)))' option !"));
                                                }
                                            })
                                        }
                                    }
                                    else if meta.path.is_ident("trt"){
                                        let msg = "As of the current version, the `actor`'s `Script \
                                        struct` does not implement any traits. Consequently, the use of the \
                                        `trt` argument for `script` is not applicable. If your intention is \
                                        to modify derived traits, consider using the `def` option instead.";
                                        return Err(meta.error(msg));
                                    }
                                    else {
                                        return Err(meta.error("Unsuported 'edit(script(?))' option ! "));
                                    }
                                })
                            }
                        }

                        else if  meta.path.is_ident("live") {
                            // if ident 
                            if meta.input.clone().to_string().is_empty() {
                                self.edit.live.0 = true;
                                self.edit.live.1 = Some(Vec::new());
                                self.edit.live.2 = Some(Vec::new());
                                return Ok(());
                            } else {
                                meta.parse_nested_meta(|meta|{
                                    if meta.path.is_ident("def"){
                                        self.edit.live.0 = true;
                                        return Ok(());
                                    }
                                    else if meta.path.is_ident("imp"){
                                        // if ident
                                        if meta.input.clone().to_string().is_empty() {
                                            self.edit.live.1 = Some( Vec::new());
                                            return Ok(());
                                        } else {
                                            meta.parse_nested_meta(|meta| {
                                                if let Some(ident) = meta.path.get_ident(){
                                                    if self.edit.live.1.is_some(){
                                                        self.edit.live.1.as_mut().map(|x| x.push(ident.clone()));
                                                        return Ok(());
                                                    } else {
                                                        self.edit.live.1 = Some( Vec::from([ident.clone()]));
                                                        return Ok(());
                                                    }
                                                } else {
                                                    return Err(meta.error("Unsuported 'edit(live(impl( ? )))' option !"));
                                                }
                                            })
                                        }
                                    }
                                    
                                    else if meta.path.is_ident("trt"){
                                        // if ident
                                        if meta.input.clone().to_string().is_empty() {
                                            self.edit.live.2 = Some( Vec::new());
                                            return Ok(());
                                        } else {
                                            meta.parse_nested_meta(|meta| {
                                                if let Some(ident) = meta.path.get_ident(){
                                                    if self.edit.live.2.is_some(){
                                                        self.edit.live.2.as_mut().map(|x| x.push(ident.clone()));
                                                        return Ok(());
                                                    } else {
                                                        self.edit.live.2 = Some( Vec::from([ident.clone()]));
                                                        return Ok(());
                                                    }
                                                } else {
                                                    return Err(meta.error("Unsuported 'edit(live(impl( ? )))' option !"));
                                                }
                                            })
                                        }
                                    }
                                    else {
                                        return Err(meta.error("Unsuported 'edit(live( ? ))' option !"));
                                    }
                                })
                            }
                        }
                        else { return Err( meta.error("Unsuported edit option") );}
                    }){
                        Ok(_) => (),
                        Err(e) => {
                            let span   = e.span();
                            let msg  = e.to_string();
                            abort!(span,msg;help=error::AVAIL_EDIT );
                        },
                    }
                }
            }

            //ID
            else if meta.path.is_ident("id"){
                if meta.input.clone().to_string().is_empty() {
                    // self.id.0 = Some(ident.clone());
                    self.id = true;
                } else {
                    let  value = meta.value()?.parse::<syn::Lit>()?;
                    match value.clone() {
                        syn::Lit::Bool(val) => { 
                            // self.id.0 = Some(ident.clone());
                            self.id = val.value();
                            return Ok(());
                        },
                        v => abort!(v, error::error_name_type( ident.clone(), "bool".into()); help=error::AVAIL_ACTOR ),
                    }
                }
            }
            
            // FILE
            else if meta.path.is_ident("file") {

                let value = meta.value()?.parse::<syn::Lit>()?;

                match value.clone() {
                    syn::Lit::Str(val) => {

                        // the path needs to be checked first 
                        let path = std::path::PathBuf::from(val.value());

                        if path.exists() {
                            self.file = Some(path);
                            return Ok(());
                        }
                        else {
                            abort!(val, format!("Path - {:?} does not exists.",val.value())); 
                        } 
                    },
                    _ => {
                        return Err( meta.error(format!("Expected a  'str'  value for argument '{}'.", ident.to_string() )));
                    },
                }
            }

            // UNKNOWN ARGUMENT
            else {
                error::unknown_attr_arg("actor",ident )
            }
        }
        Ok(())
    }
} 







