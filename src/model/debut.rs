use quote::{quote,format_ident};
use syn::{Visibility,Ident,TypeGenerics,WhereClause};
use proc_macro2::TokenStream;


pub static INTER_GET_DEBUT: &'static str = "inter_get_debut";
pub static INTER_GET_COUNT: &'static str = "inter_get_count";
pub static INTER_SET_NAME: &'static str  = "inter_set_name";
pub static INTER_GET_NAME: &'static str  = "inter_get_name";
pub static INTER_NAME: &'static str      = "InterName";
pub static DEBUT: &'static str           = "debut";

// if aaa.debut.active() {
pub fn debut (
    live_mets:   &mut Vec<(Ident,TokenStream)>,
    live_trts:   &mut Vec<(Ident,TokenStream)>,
    script_mets: &mut Vec<(Ident,TokenStream)>,
    live_name: &Ident,
    new_vis: &Option<Visibility>,
    ty_generics: &TypeGenerics,
    where_clause: &Option<&WhereClause>,
    
){  
    let fn_name = format_ident!("{INTER_GET_DEBUT}");
    let ts  = quote!{
        #new_vis fn #fn_name (&self) -> std::time::SystemTime {
            *self.debut
        }
    };
    live_mets.push((fn_name,ts));
    

    let fn_name = format_ident!("{INTER_GET_COUNT}");
    let ts = quote!{
        #new_vis fn #fn_name (&self) -> usize {
            std::sync::Arc::strong_count(&self.debut)
        }
    };
    live_mets.push((fn_name,ts));


    let fn_name  = format_ident!("{INTER_SET_NAME}");
    let gen_type = format_ident!("{INTER_NAME}");
    let ts = quote!{
        #new_vis fn #fn_name < #gen_type: std::string::ToString>(&mut self, name:  #gen_type) {
            self.name = name.to_string();
        }
    };
    live_mets.push((fn_name,ts));


    let fn_name = format_ident!("{INTER_GET_NAME}");
    let ts = quote!{    
        #new_vis fn #fn_name (&self) -> &str {
            &self.name
        } 
    };
    live_mets.push((fn_name,ts));


    let fn_name = format_ident!("{DEBUT}");
    let ts = quote!{
        // we need this function to return as much an id as it is possible
        // the model will build some options on top of this "id"
        // it MUST be unique 
        pub fn #fn_name ()-> std::sync::Arc<std::time::SystemTime> {
            static LAST: std::sync::Mutex<std::time::SystemTime> = std::sync::Mutex::new(std::time::SystemTime::UNIX_EPOCH);
    
            let mut last_time = LAST.lock().unwrap();
            let mut next_time = std::time::SystemTime::now();
    
            // we check for 'drift'
            // as described in docs 
            while !(*last_time < next_time)  {
                // in case if they are just equal
                // add a nano but don't break the loop yet
                if *last_time == next_time {
                    next_time += std::time::Duration::new(0, 1);
                } else {
                    next_time = std::time::SystemTime::now();
                }
            }
            // update LAST 
            *last_time = next_time.clone();
            std::sync::Arc::new(next_time)
        }
    };
    script_mets.push((fn_name,ts));
    

    live_trts.push((format_ident!("PartialEq"),
    quote!{
        impl #ty_generics std::cmp::PartialEq for #live_name #ty_generics #where_clause{
            fn eq(&self, other: &Self) -> bool {
                *self.debut == *other.debut
            }
        }
    }));

    live_trts.push((format_ident!("Eq"),
    quote!{
        impl #ty_generics std::cmp::Eq for #live_name #ty_generics #where_clause {}
    }));  

    live_trts.push((format_ident!("PartialOrd"),
    quote!{
        impl #ty_generics std::cmp::PartialOrd for #live_name #ty_generics #where_clause{
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                other.debut.partial_cmp(&self.debut)
            }
        }
    }));   

    live_trts.push((format_ident!("Ord"),
    quote!{
        impl #ty_generics std::cmp::Ord for #live_name #ty_generics #where_clause {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                other.debut.cmp(&self.debut)
            }
        }
    }));  
} 
