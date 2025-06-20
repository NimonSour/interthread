
use crate::model::{ActorAttributeArguments as AAA, ConstVars, Cont, ImplVars, ModelGenerics};
use quote::{quote,format_ident};
use proc_macro2::TokenStream;
use syn::{ parse_quote,Path,TypePath};

use super::Mac;

//-----------------------  ACTOR DEBUT
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Debut {
    pub active: bool
}

impl Debut {
    pub fn active(&self) -> bool {
        self.active
    } 
}


impl Debut {

    pub fn impl_debut(&self, cont: &mut Cont, impl_vars: &ImplVars, aaa: &AAA) {  

        let ImplVars{ vis, live_name,const_vars, ..} = impl_vars;
        let ModelGenerics{ live_gen,..} = &impl_vars.mod_gen;
        let ConstVars{
            inter_get_debut,
            inter_get_name,
            inter_set_name,
            inter_get_count,
            debut,
            intername,
            name,
            ..
        } = const_vars;
        
        if aaa.mac == Mac::Actor {

            if aaa.mod_receiver.is_slf(){

                let ts = self.get_method_debut(const_vars);
                cont.push_script_met(debut,&ts);
            }

            let (impl_generics,ty_generics,where_clause) = live_gen.split_for_impl();
    
            cont.push_live_met(inter_get_debut,
                & quote!{
                    fn #inter_get_debut (&self) -> std::time::SystemTime {
                        *self.#debut
                    }
                }
            );
            
            cont.push_live_met(inter_get_count,
                & quote!{
                    #vis fn #inter_get_count (&self) -> usize {
                        std::sync::Arc::strong_count(&self.#debut)
                    }
                }
            );
    
            cont.push_live_met(inter_set_name,
                & quote!{
                    #vis fn #inter_set_name < #intername: std::string::ToString>(&mut self, #name:  #intername) {
                        self.#name = #name.to_string();
                    }
                }
            );
        
            cont.push_live_met(inter_get_name,
                & quote!{    
                    #vis fn #inter_get_name (&self) -> ::std::string::String {
                        self.#name.clone()
                    } 
                }
            );
        
            cont.push_live_trt(&format_ident!("PartialEq"),
            &quote!{
                impl #impl_generics ::std::cmp::PartialEq for #live_name #ty_generics #where_clause{
                    fn eq(&self, other: &Self) -> bool {
                        *self.#debut == *other.#debut
                    }
                }
            });
        
            cont.push_live_trt(&format_ident!("Eq"),
            &quote!{
                impl #impl_generics ::std::cmp::Eq for #live_name #ty_generics #where_clause {}
            });  
        
            cont.push_live_trt(&format_ident!("PartialOrd"),
            &quote!{
                impl #impl_generics ::std::cmp::PartialOrd for #live_name #ty_generics #where_clause{
                    fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
                        other.#debut.partial_cmp(&self.#debut)
                    }
                }
            });   
        
            cont.push_live_trt(&format_ident!("Ord"),
            &quote!{
                impl #impl_generics ::std::cmp::Ord for #live_name #ty_generics #where_clause {
                    fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                        other.#debut.cmp(&self.#debut)
                    }
                }
            }); 
        }
    } 


    pub fn get_method_debut(&self,const_vars: &ConstVars) -> TokenStream {

        let ConstVars{debut,..} = const_vars;

        quote!{
            // we need this function to return as much an id as it is possible
            // the model will build some options on top of this "id"
            // it MUST be unique 

            fn #debut ()-> ::std::time::SystemTime {
                static LAST: ::std::sync::Mutex<::std::time::SystemTime> = ::std::sync::Mutex::new(::std::time::SystemTime::UNIX_EPOCH);
        
                let mut last_time = LAST.lock().unwrap();
                let mut next_time = ::std::time::SystemTime::now();
        
                // we check for 'drift'
                // as described in docs 
                while !(*last_time < next_time)  {
                    // in case if they are just equal
                    // add a nano but don't break the loop yet
                    if *last_time == next_time {
                        next_time += ::std::time::Duration::new(0, 1);
                    } else {
                        next_time = ::std::time::SystemTime::now();
                    }
                }
                // update LAST 
                *last_time = next_time.clone();
                next_time
            }
        }
    }

    // helper methods 
    pub fn get_debut_path(&self) -> Option<Path> {
        if self.active {
            Some( parse_quote!( ::std::sync::Arc<::std::time::SystemTime> ))
        } else {
            None
        }
    }
    pub fn get_live_fields(&self, const_vars: &ConstVars) -> TokenStream {
        let ConstVars{debut,name,..} = const_vars;
        if self.active {
            quote!{
                #debut : ::std::sync::Arc<::std::time::SystemTime>,
                #name  : ::std::string::String,
            }
        } else { quote!{} }
    }

    pub fn get_debut_filds_init(&self,const_vars: &ConstVars) -> TokenStream { 
        let ConstVars{debut,name,..} = const_vars;
        if self.active {
            quote!{
                #debut : ::std::sync::Arc::new( #debut ),
                #name  : ::std::string::String::new(),
            }
        } else { quote!{} }
    }
    
    pub fn get_debut_decl_call(&self,script_path: &TypePath, const_vars: &ConstVars) -> TokenStream { 
        let ConstVars{debut,..} = const_vars;
        if self.active {
            quote!{ let #debut = #script_path ::#debut(); }
        } else { quote!{} }
    }


}

impl Default for Debut {
    fn default() -> Debut {
        Self{ active: false} 
    }
}

