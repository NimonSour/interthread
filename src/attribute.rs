use crate::error;

use std::path::PathBuf;
use proc_macro2::Span;
use proc_macro_error::abort;
use quote::format_ident;


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
    pub script:( bool, Option<Vec<syn::Ident>>, Option<Vec<syn::Ident>> ),
    pub live:  ( bool, Option<Vec<syn::Ident>>, Option<Vec<syn::Ident>> ),
}
impl AAEdit {

    pub fn parse(&mut self,  meta: syn::meta::ParseNestedMeta, sol: bool ) -> Result<(), syn::Error> {
        let (name,mut strct) = 
        if sol {("script",self.script.clone()) } else { ("live",self.live.clone())};

        let parse_names = 
        |    strct_name: &str, 
         strct_prt_name: &str, 
              strct_prt: &mut Option<Vec<syn::Ident>>, 
                   meta: syn::meta::ParseNestedMeta,
        | -> Result<(),syn::Error> {

            if let Some(ident) = meta.path.get_ident(){
                if strct_prt.is_some(){
                    strct_prt.as_mut().map(|x| x.push(ident.clone()));
                    return Ok(());
                } else {
                    *strct_prt = Some( Vec::from([ident.clone()]));
                    return Ok(());
                }
            } else {
                let mot = if strct_prt_name == "imp" { "method"  } else { "trait" };
                let msg = 
                format!("Unsuported 'edit({}({}( ? )))' option! Expected a name of {} .",
                strct_name, strct_prt_name, mot );
                return Err(meta.error(msg));
            }
        };

        if meta.path.is_ident("def"){
            strct.0 = true;
        }
        else if meta.path.is_ident("imp"){
            // if ident
            if meta.input.clone().to_string().is_empty() {
                strct.1 = Some( Vec::new());
            } else {
                if let Err(e) = meta.parse_nested_meta(|meta| {
                    
                    parse_names(name,"imp",&mut strct.1,meta)

                }) {
                    return Err(e); 
                }
            }
        }
        
        else if meta.path.is_ident("trt"){
            if sol {
                return Err(meta.error(crate::error::SCRIPT_NO_TRT));
            } else {

                // if ident
                if meta.input.clone().to_string().is_empty() {
                    strct.2 = Some( Vec::new());
                } else {
                    if let Err(e) = meta.parse_nested_meta(|meta| {

                        parse_names(name,"trt",&mut strct.2,meta)

                    }){
                        return Err(e);
                    }
                }
            }
        }
        else {
            let opts  = if sol { " `def` or `imp` " } else { " `def`,`imp` or `trt` " };
            let msg = format!("Unsuported 'edit({}( ? ))' option! Expected options are {}.",name, opts);
            return Err(meta.error(msg));
        }

        if sol {
            self.script = strct;
            return Ok(());
        } else { 
            self.live = strct;
            return Ok(());
        };

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
        self.live.2 == empty
    } 

    pub fn is_none(&self) -> bool {

        self.live.0 == false && self.script.0 == false &&
        self.live.1 == None  && self.script.1 == None  &&
        self.live.2 == None
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
                    self.edit.set_script_all();
                    self.edit.set_live_all();
                    return Ok(());
                } 

                else { 

                    match meta.parse_nested_meta( |meta| {

                        if meta.path.is_ident("script") {
                            // if ident 
                            if meta.input.clone().to_string().is_empty() {
                                self.edit.set_script_all();
                                return Ok(());
                            } else {

                                meta.parse_nested_meta(|meta|{
                                    return self.edit.parse(meta,true)
                                })
                            }
                        }

                        else if  meta.path.is_ident("live") {
                            // if ident 
                            if meta.input.clone().to_string().is_empty() {
                                self.edit.set_live_all();
                                return Ok(());

                            } else {

                                meta.parse_nested_meta(|meta|{
                                    return self.edit.parse(meta,false)
                                })
                            }
                        }
                        // old args 
                        else if  meta.path.is_ident("direct") {
                            return Err( meta.error(crate::error::OLD_DIRECT_ARG));
                        }
                        else if  meta.path.is_ident("play") {
                            return Err( meta.error(crate::error::OLD_PLAY_ARG));
                        }
                        // wrong opt
                        else { 
                            return Err( meta.error("Unsuported edit option") );
                        }
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

            // ID
            else if meta.path.is_ident("id"){
                if meta.input.clone().to_string().is_empty() {
                    self.id = true;
                } else {
                    let  value = meta.value()?.parse::<syn::Lit>()?;
                    match value.clone() {
                        syn::Lit::Bool(val) => { 
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
                            // one only check 
                            match crate::file::macro_file_count(&path) {
                                Ok((attr,attrs)) => {
                                    self.file = Some(AAFile {
                                                                path: path.clone(),
                                                                attr,
                                                                attrs });
                                    return Ok(());
                                },
                                Err(e) => { abort!(value,e); },
                            }
                        }
                        else {
                            abort!(val, format!("Path - {:?} does not exists.",val.value())); 
                        } 
                    },
                    _ => {
                        return Err( meta.error("Expected a  'str'  value for argument 'file'."));
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

    pub fn cross_check(&self){

        if self.file.is_some() {
            if self.edit.is_none(){
                let msg = "Expected an `edit` argument!";
                abort!(Span::call_site(),msg;help=error::AVAIL_EDIT);
            }
        }
    }
} 







