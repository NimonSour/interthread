use quote::{quote,format_ident};
use syn::{Visibility,Ident,TypeGenerics,WhereClause};
use proc_macro2::TokenStream;
use std::path::PathBuf;

pub static INTER_GET_DEBUT: &'static str = "inter_get_debut";
pub static INTER_GET_COUNT: &'static str = "inter_get_count";
pub static INTER_SET_NAME: &'static str  = "inter_set_name";
pub static INTER_GET_NAME: &'static str  = "inter_get_name";
pub static INTER_NAME: &'static str      = "InterName";
pub static DEBUT: &'static str           = "debut";


//-----------------------  ACTOR DEBUT
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Debut {
    pub path:      Option<PathBuf>,
    pub legend:       Option<bool>,
}

impl Debut {

    pub fn active(&self) -> bool {
        self.legend.is_some()
    } 
    pub fn is_legend(&self) -> bool {
        if let Some(bol) = self.legend{
            bol
        } else { false }
    } 

}

impl Default for Debut {
    fn default() -> Debut {
        Self{ path: None, legend: None } 
    }
}


impl Debut {

    pub fn impl_debut(&self,
        live_mets:   &mut Vec<(Ident,TokenStream)>,
        live_trts:   &mut Vec<(Ident,TokenStream)>,
        script_mets: &mut Vec<(Ident,TokenStream)>,
        live_name: &Ident,
        new_vis: &Option<Visibility>,
        ty_generics: &TypeGenerics,
        where_clause: &Option<&WhereClause>,
        
    ){  
        let inter_get_debut = format_ident!("{INTER_GET_DEBUT}");
        let ts  = quote!{
            #new_vis fn #inter_get_debut (&self) -> std::time::SystemTime {
                *self.debut
            }
        };
        live_mets.push((inter_get_debut,ts));
        
    
        let inter_get_count = format_ident!("{INTER_GET_COUNT}");
        let ts = quote!{
            #new_vis fn #inter_get_count (&self) -> usize {
                std::sync::Arc::strong_count(&self.debut)
            }
        };
        live_mets.push((inter_get_count,ts));
    
    
        let inter_set_name  = format_ident!("{INTER_SET_NAME}");
        let gen_type = format_ident!("{INTER_NAME}");
        let ts = quote!{
            #new_vis fn #inter_set_name < #gen_type: std::string::ToString>(&mut self, name:  #gen_type) {
                self.name = name.to_string();
            }
        };
        live_mets.push((inter_set_name,ts));
    
    
        let inter_get_name = format_ident!("{INTER_GET_NAME}");
        let ts = quote!{    
            #new_vis fn #inter_get_name (&self) -> &str {
                &self.name
            } 
        };
        live_mets.push((inter_get_name,ts));
    
    
        let debut = format_ident!("{DEBUT}");
        let ts = quote!{
            // we need this function to return as much an id as it is possible
            // the model will build some options on top of this "id"
            // it MUST be unique 
            pub fn #debut ()-> std::sync::Arc<std::time::SystemTime> {
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
        script_mets.push((debut,ts));
        
    
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
}
