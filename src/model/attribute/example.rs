use crate::error;
use crate::model::argument::Model;


use proc_macro2::Span;
use proc_macro_error::abort;


// #[derive(Debug, Eq, PartialEq, Clone, Copy)]
// pub enum Either<L, R> {
//     L(L),
//     R(R),
// }



//-----------------------  EXAMPLE 
#[derive(Debug, Eq, PartialEq)]
pub struct ExampleAttributeArguments {

    pub path     : Option<std::path::PathBuf>,
    pub main     :                       bool,
    pub expand   :              Vec<Model>,  
    /* ADD NEW OPTION */ 
}

impl Default for ExampleAttributeArguments {

    fn default() -> Self {

        let path  = None ;
        let main             = false;
        let expand  = vec![Model::Actor, Model::Group];
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
                        self.expand.push(Model::Actor);
                        Ok(())
                    }
                    else if meta.path.is_ident("group"){
                        self.expand.push(Model::Group);
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