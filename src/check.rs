

pub fn is_imported( name: &str ){

    match proc_macro_crate::crate_name(name) {

        Ok( proc_macro_crate::FoundCrate::Name(_) ) => (),

        _ => {
            let msg  = format!("Error. Crate {} not found.", name);
            let help = format!("This issue can be easily solved by importing the '{}' crate into the project. Simply run the following command in your terminal: $ cargo add {} ",name, name );
            proc_macro_error::abort!( proc_macro2::Span::call_site(), msg; help=help);
        }
    }
}

pub fn channels_import( lib: &crate::attribute::AALib ){ 

    match lib {
        crate::attribute::AALib::Tokio => (),
        crate::attribute::AALib::Std   => {
            crate::check::is_imported("oneshot");
        },
        _ => { 
            crate::check::is_imported("async-channel");
            crate::check::is_imported("oneshot");
        }
    }
}