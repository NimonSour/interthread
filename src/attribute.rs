use crate::error;
use crate::file::get_ident;

use std::path::PathBuf;
use proc_macro2::Span;
use proc_macro_error::abort;
use quote::format_ident;
use syn::punctuated::Punctuated;


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




/*
needs a check for methods 
if it finds any methods with a name 
`file` return an error saying that  
active 'file' trigger argument
should be renamed to 'inter_file'.
*/


/*
    filter_file  returns  Some(punctuated) if file
                            None 

    this is fo single  Ident(file) options
    applicable for def(file) as well

    in Meta bool ->  out (syn::Ident,bool) 
*/
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

pub struct AAEdit {
    pub attr: Option<EditAttribute>,
    pub script:( (bool,bool), (Option<Vec<(syn::Ident,bool)>>,bool), (Option<Vec<(syn::Ident,bool)>>,bool) ),
    pub live:  ( (bool,bool), (Option<Vec<(syn::Ident,bool)>>,bool), (Option<Vec<(syn::Ident,bool)>>,bool) ),
}

impl AAEdit {

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

impl Default for AAEdit {

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


//-----------------------  ACTOR FILE
#[derive(Debug, Eq, PartialEq, Clone)]

pub struct EditAttribute {
    pub path:              PathBuf,
    pub attr:       syn::Attribute,
    pub attrs: Vec<syn::Attribute>,
    pub new_attr:   syn::Attribute,
}


//-----------------------  ACTOR DEBUT
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct AADebut {
    pub path:      Option<PathBuf>,
    pub legend:       Option<bool>,
}

impl AADebut {

    pub fn active(&self) -> bool {
        self.legend.is_some()
    } 
    pub fn is_legend(&self) -> bool {
        if let Some(bol) = self.legend{
            bol
        } else { false }
    } 

}

impl Default for AADebut {
    fn default() -> AADebut {
        Self{ path: None, legend: None } 
    }
}


//-----------------------  ACTOR  

#[derive(Debug,Clone, Eq, PartialEq)]
pub struct ActorAttributeArguments {

    pub name    :  Option<syn::Ident>,
    pub lib     :  AALib,
    pub assoc   :  bool,
    pub channel :  AAChannel,
    pub edit    :  AAEdit,
    pub debut   :  AADebut,
    // pub file    :  Option<AAFile>,
    pub file    :  Option<std::path::PathBuf>,
    /* ADD NEW OPTION */
}


impl Default for ActorAttributeArguments {

    fn default() -> ActorAttributeArguments {

        Self { 
            name   : None,
            lib    : AALib::default(),
            assoc  : false,
            channel: AAChannel::default(),
            edit   : AAEdit::default(),
            debut  : AADebut::default(),
            file   : None,
            /* ADD NEW ATTRIBUTE */
        }  
    }
}


impl ActorAttributeArguments {
       
    pub fn parse_nested(&mut self, nested: Punctuated::<syn::Meta,syn::Token![,]>) {

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
                    self.edit.parse_nested(&meta);
                }

                // DEBUT

                /*
                
                #[actor( channel=0, debut(legend(path="src")) )]
                AVAIL_DEBUT
                 */
                else if meta.path().is_ident("debut"){

                    if let Some(meta_list) = get_list( meta,Some(error::AVAIL_DEBUT) ) {

                        for m in meta_list {
                            if m.path().is_ident("legend"){
                                if let Some(meta_list) = get_list( meta,Some(error::AVAIL_DEBUT) ) {
                                    for m in meta_list{

                                        if m.path().is_ident("path"){

                                            match get_lit(&m) {
                                                syn::Lit::Str(val) => {
                                                    let path_str = val.value();
                                                    todo!()
                                                },
                                                _ => { abort!(m, error::error_name_type( &ident, "bool"); help=error::AVAIL_ACTOR ) },
                                            }
                                        } else {
                                            let msg = "Unknown option for argument 'debut'.";
                                            abort!(m,msg;help=error::AVAIL_DEBUT);
                                        }
                                    }
                                } else { self.debut.legend = Some(true); }  
                            } else {
                                let msg = "Unknown option for argument 'debut'.";
                                abort!(m,msg;help=error::AVAIL_DEBUT);
                            }
                        }
                    } else {  self.debut.legend = Some(false);  }


                    // match meta {
                    //     syn::Meta::Path(_) => { self.debut.legend = Some(false); },
                    //     syn::Meta::List(_)=> {
                    //         if meta_list.
                    //     },
                    //     _ => {
                    //         match get_lit(meta) {
                    //             syn::Lit::Bool(val) => { self.id = val.value(); },
                    //             v => abort!(v, error::error_name_type( &ident, "bool"); help=error::AVAIL_ACTOR ),
                    //         }
                    //     }
                    // }
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
                                
                                self.file = Some(path);
                            }
                            else {
                                abort!(val, format!("Path - {:?} does not exists.",val.value())); 
                            } 
                        },
                        _ => { abort!(value, error::error_name_type( &ident, "str"); help=error::AVAIL_ACTOR ) },
                    }
                }

                else if meta.path().is_ident("id"){ 
                    // error "id" is "debut" since v2.0.0
                    abort!(ident, error::OLD_ARG_ID);
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


    pub fn cross_check(&mut self){

        if !self.edit.is_any_active(){
            // let msg = format!("script - {:?}, live - {:?}", &self.edit.script, &self.edit.live);
            // abort!(Span::call_site(),msg);
            if let Some(file_path) = &self.file {
                match crate::file::macro_file_count(file_path) {
                    Ok(edit_attr) => {
                        self.edit.attr = Some(edit_attr);
                    },
                    Err(e) => { abort!(Span::call_site(),e); },
                }
            } else {
                // error for using option file active but the path is not specified 
                let msg = r#"Expected a 'file' argument ` file = "path/to/current/file.rs" ` ."#; 
                abort!(Span::call_site(),msg;help=error::AVAIL_ACTOR)
            }
        }
    }
} 




// GROUP ARGUMENTS 
pub struct  AGEdit {
    pub script:( bool, Option<Vec<syn::Ident>>, Option<Vec<syn::Ident>> ),
    pub live:  ( bool, Option<Vec<syn::Ident>>, Option<Vec<syn::Ident>> ),
    pub groupart: Option<Vec< (syn::Ident, AAEdit)>>,
}

impl AGEdit {

    pub fn set_live_all(&mut self){
        self.live = (true,Some(Vec::new()),Some(Vec::new()));
    }

    pub fn set_script_all (&mut self){
        self.script = (true,Some(Vec::new()),Some(Vec::new()));
    }

    pub fn set_groupart_all (&mut self){
        self.groupart = Some(Vec::new());
    }
    
    pub fn is_all(&self) -> bool {
        let empty = Some(Vec::new());
        let empty_g = Some(Vec::<(syn::Ident, AAEdit)>::new());
        self.live.0 == true  && self.script.0 == true  &&
        self.live.1 == empty && self.script.1 == empty &&
        self.live.2 == empty && self.script.1 == empty &&
        self.groupart == empty_g
    } 

    pub fn is_none(&self) -> bool {

        self.live.0 == false && self.script.0 == false &&
        self.live.1 == None  && self.script.1 == None  &&
        self.live.2 == None  && self.script.2 == None  &&
        self.groupart == None
    }  

}

impl Default for AGEdit {

    fn default() -> Self {
        let script  = (false,None,None);
        let live    = (false,None,None);
        let groupart = None;
        Self { script, live, groupart }
    } 
}


pub struct GroupAttributeArguments {

    pub name    :  Option<syn::Ident>,
    pub lib     :  AALib,
    pub assoc   :  bool,
    pub channel :  AAChannel,
    pub file    :  Option<std::path::PathBuf>,
 
}



impl GroupAttributeArguments {

    pub fn parse_nested(&mut self, nested: Punctuated::<syn::Meta,syn::Token![,]>) {
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

                // FILE
                else if meta.path().is_ident("file") {
                    let value = get_lit(meta);

                    match value.clone() {
                        syn::Lit::Str(val) => {

                            // the path needs to be checked first 
                            let path = std::path::PathBuf::from(val.value());

                            if path.exists() {
                                // one only check 
                                self.file = Some(path);
                            }
                            else {
                                abort!(val, format!("Path - {:?} does not exists.",val.value())); 
                            } 
                        },
                        _ => { abort!(value, error::error_name_type( &ident, "str"); help=error::AVAIL_ACTOR ) },
                    }
                }
            } else { 
                abort!(meta,"Unknown configuration option!"; help=error::AVAIL_ACTOR); 
            }
        }
    }
}


impl Default for GroupAttributeArguments {

    fn default() -> GroupAttributeArguments {

        Self { 
            name   : None,
            lib    : AALib::default(),
            assoc  : false,
            channel: AAChannel::default(),
            file   : None,
            // edit   : AAEdit::default(),
            // debut  : AADebut::default(),
            // file   : None,
            /* ADD NEW ATTRIBUTE */
        }  
    }
}



// impl Attribute for GroupAttributeArguments {}
// impl Attribute for ActorAttributeArguments {}

// pub trait Attribute{}

pub fn get_list(meta: & syn::Meta, help: Option<&str>) -> Option<Punctuated::<syn::Meta,syn::Token![,]>> {
    match meta {
        syn::Meta::Path(_) => { None },
        syn::Meta::List(meta_list) => { 
            let list = 
            meta_list.parse_args_with(Punctuated::<syn::Meta,syn::Token![,]>::parse_terminated).unwrap();
            Some(list) 
        },
        syn::Meta::NameValue(_) => { 
            if let Some(help) = help {
                abort!(meta,"Expected a list!"; help=help) 
            } else { None }
        },
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





