
use crate::model::{Cont,ImplVars,MpscChannel,Vars};
use quote::{quote,format_ident};
use proc_macro2::TokenStream;
use syn::{Visibility,Ident,ImplGenerics,TypeGenerics,WhereClause};

use std::path::PathBuf;

//-----------------------  ACTOR DEBUT
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Debut {
    pub path:      Option<PathBuf>,
    pub legend:       Option<bool>,
    pub leg_script: Option<PathBuf>,   
    pub leg_live:   Option<PathBuf>,   
    
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
        Self{ 
              path: None, 
            legend: None,
        leg_script: None,   
          leg_live: None,
        } 
    }
}


impl Debut {

    pub fn impl_debut(&self,
        Cont{

            live_mets,
            live_trts,
            script_mets,..
        }: &mut Cont,
        Vars{
            live_name,
            inter_get_debut,
            inter_get_name,
            inter_set_name,
            inter_get_count,
            debut,
            intername,..
        }: &Vars,
        new_vis:        &Option<Visibility>,
        impl_generics:        &ImplGenerics,
        ty_generics:          &TypeGenerics,
        where_clause: &Option<&WhereClause>,
        
    ){  

        live_mets.push((inter_get_debut.clone(),
            quote!{
                #new_vis fn #inter_get_debut (&self) -> std::time::SystemTime {
                    *self.debut
                }
            },
            vec![]
        ));
        
        live_mets.push((inter_get_count.clone(),
            quote!{
                #new_vis fn #inter_get_count (&self) -> usize {
                    std::sync::Arc::strong_count(&self.debut)
                }
            },
            vec![]
        ));

        live_mets.push((inter_set_name.clone(),
            quote!{
                #new_vis fn #inter_set_name < #intername: std::string::ToString>(&mut self, name:  #intername) {
                    self.name = name.to_string();
                }
            },
            vec![]
        ));
    
        live_mets.push((inter_get_name.clone(),
            quote!{    
                #new_vis fn #inter_get_name (&self) -> std::string::String {
                    self.name.clone()
                } 
            },
            vec![]
        ));
    
    
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
        script_mets.push((debut.clone(),ts));
        
    
        live_trts.push((format_ident!("PartialEq"),
        quote!{
            impl #impl_generics std::cmp::PartialEq for #live_name #ty_generics #where_clause{
                fn eq(&self, other: &Self) -> bool {
                    *self.debut == *other.debut
                }
            }
        }));
    
        live_trts.push((format_ident!("Eq"),
        quote!{
            impl #impl_generics std::cmp::Eq for #live_name #ty_generics #where_clause {}
        }));  
    
        live_trts.push((format_ident!("PartialOrd"),
        quote!{
            impl #impl_generics std::cmp::PartialOrd for #live_name #ty_generics #where_clause{
                fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                    other.debut.partial_cmp(&self.debut)
                }
            }
        }));   
    
        live_trts.push((format_ident!("Ord"),
        quote!{
            impl #impl_generics std::cmp::Ord for #live_name #ty_generics #where_clause {
                fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                    other.debut.cmp(&self.debut)
                }
            }
        })); 
    } 


    pub fn impl_legend(&self,
        Cont{

            live_mets,
            live_trts,
            script_mets,..
        }: &mut Cont,
        Vars{
            actor,
            live,
            debut_for_play,
            sender,
            name,
            live_name,
            script_name,
            debut,
            intername,
            inter_get_name,
            inter_get_count,
            inter_get_debut,
            inter_set_channel,
            inter_get_channel,
            impl_vars,
            actor_legend,
            live_legend,
            try_old,..
        }: &Vars,
        new_vis:        &Option<Visibility>,
        mpsc:                  &MpscChannel,
        fields:                 Vec<&Ident>,
        s_ty_generics:        &TypeGenerics,
        impl_generics:        &ImplGenerics,
        ty_generics:          &TypeGenerics,
        where_clause: &Option<&WhereClause>,
        spawn:                 &TokenStream,
    
    ) { 

        let ImplVars{actor_type,model_generics,..} = &impl_vars;
        let MpscChannel{ pat_type_sender, type_sender,declaration_call, declaration,..} = mpsc;
        let old_inst_live = format_ident!("old_{actor}_live");
        let s_turbofish = s_ty_generics.as_turbofish();

        let actor_ty:TokenStream;
        let live_ty:TokenStream;
        let actor_bt_ty:TokenStream;
        let live_bt_ty:TokenStream;
        let actor_insert:TokenStream;
        let live_insert:TokenStream;
        let actor_remove_expr:TokenStream;
        let live_remove_expr:TokenStream;
        
        
        if crate::model::is_generic(model_generics){

            actor_ty = quote!{ #actor_type};
            live_ty = quote!{ #live_name #ty_generics};
            actor_bt_ty = quote!{ std::boxed::Box<dyn std::any::Any + Send> };
            live_bt_ty = quote!{ std::boxed::Box<dyn std::any::Any + Send> };
            actor_insert = quote!{ std::boxed::Box::new(#actor)};
            live_insert = quote!{ std::boxed::Box::new(#live)};
            actor_remove_expr = quote!{
                let boxed_actor = collection.remove(&#debut)?;
                boxed_actor.downcast::< #actor_ty >().ok().map(|boxed| *boxed)
            };
            live_remove_expr = quote!{
                let boxed_live = collection.remove(&#name .to_string())?;
                boxed_live.downcast::< #live_ty >().ok().map(|boxed| *boxed)
            };

        } else {
            
            actor_ty = quote!{ #actor_type };
            live_ty = quote!{ #live_name };
            actor_bt_ty = quote!{ #actor_type };
            live_bt_ty = quote!{ #live_name };
            actor_insert = quote!{ #actor };
            live_insert = quote!{ #live };
            actor_remove_expr = quote!{ collection.remove(&#debut) };
            live_remove_expr = quote!{ collection.remove(& #name .to_string()) };
        };


        let live_ty = &live_ty;
        let replace_field  = | field: &Ident|{
            quote!{ self. #field. #sender =  #sender.clone() ; }
        };
        let replace_fields = {
            let mut loc = vec![];
            for f in fields {
                loc.push(replace_field(f));
            } 
            loc.push(quote!{ self. #sender =  #sender ; });
            loc
        };


        script_mets.push((actor_legend.clone(),
            quote!{    
                #new_vis fn #actor_legend(#debut: std::time::SystemTime, #actor: std::option::Option<#actor_ty> ) -> std::option::Option<#actor_ty>
                {
                    static COLLECTION: std::sync::Mutex<std::collections::BTreeMap<std::time::SystemTime, #actor_bt_ty>> = 
                    std::sync::Mutex::new(std::collections::BTreeMap::new());
         
                    let mut collection = COLLECTION.lock().unwrap();
                    if let std::option::Option::Some(#actor) = #actor {
         
                        collection.insert(#debut, #actor_insert);
                        std::option::Option::None 
                    } else {
                        #actor_remove_expr
                    }
                }
            }
        ));



        script_mets.push((live_legend.clone(),
            quote!{    
                #new_vis fn #live_legend <#intername:std::string::ToString>(#name : #intername, #live: std::option::Option<#live_ty> ) -> std::option::Option<#live_ty> {

                    static COLLECTION: std::sync::Mutex<std::collections::BTreeMap<std::string::String, #live_bt_ty>> = 
                    std::sync::Mutex::new(std::collections::BTreeMap::new());
            
                    let mut collection = COLLECTION.lock().unwrap();
            
                    if let std::option::Option::Some(#live) = #live {
            
                        collection.insert(#name .to_string(), #live_insert);
                        std::option::Option::None
                    } else {
                        #live_remove_expr
                    }
                }
            }
        ));


        
        live_trts.push((format_ident!("Drop"),
        quote!{

            impl #impl_generics std::ops::Drop for #live_name #ty_generics #where_clause  {
                fn drop(&mut self) {
                
                    if self. #inter_get_count () < 2 {
                        // this will stop the while loop in play
                        let ( #sender, _ ) = #declaration_call ;
                        self . #inter_set_channel ( #sender );
                        let #name = self. #inter_get_name ();
                        let _ = #script_name :: #live_legend ( #name ,std::option::Option::Some(self.clone()));
                    }
                }
            }
        }));

        live_mets.push( (inter_get_channel.clone(),
            quote!{
                #new_vis  fn #inter_get_channel (&self) -> #type_sender {
                    self . #sender . clone()
                }
            },
            vec![]
        ));

        live_mets.push( (inter_set_channel.clone(),
            quote!{
                #new_vis  fn #inter_set_channel (&mut self, #pat_type_sender){
                    #(#replace_fields;)*
                }
            },
            vec![]
        ));

        live_mets.push( (try_old.clone(),
            quote!{
                #new_vis fn #try_old < #intername :std::string::ToString > (#name : #intername) -> std::option::Option< #live_ty > {
                    //get actor
                    let mut #old_inst_live = #script_name #s_turbofish :: #live_legend (#name, std::option::Option::None)?;
                    let #debut = #old_inst_live. #inter_get_debut();
                    let #debut_for_play = #debut .clone();
                    let #actor = #script_name #s_turbofish :: #actor_legend ( #debut, std::option::Option::None )?;
                    #declaration
                    #old_inst_live . #inter_set_channel ( #sender );

                    #spawn
                    std::option::Option::Some(#old_inst_live)
                }
            },
            vec![]
        ));

    }

}
