use crate::attribute::AALib;

pub fn is_imported( name: &str ){

    match proc_macro_crate::crate_name(name) {

        Ok( proc_macro_crate::FoundCrate::Name(_) ) => (),

        _ => {
            let msg  = format!("Crate {} not found.", name);
            let help = format!("This issue can be easily solved by importing the '{}' crate into the project. Simply run the following command in your terminal: $ cargo add {} ",name, name );
            proc_macro_error::abort!( proc_macro2::Span::call_site(), msg; help=help);
        }
    }
}

pub fn channels_import( lib: &AALib ){ 

    match lib {
        AALib::Tokio => (),

        AALib::Std |
        AALib::AsyncStd  => {
            is_imported("oneshot");
        },
        AALib::Smol => { 
            is_imported("async-channel");
            is_imported("oneshot");
        }
    }
}