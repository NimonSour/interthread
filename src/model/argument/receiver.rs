

use syn::{parse::Parse,parse_quote,TypePath,FnArg, ImplItemFn, Path,PatIdent,TypeReference, Type, Receiver };
use crate::{error, RWLOCK, MUTEX, ARC, model::{Lib,ConstVars,ModelMethod1}};
use quote::{format_ident, quote, ToTokens};
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use std::fmt::Display;



#[derive(Clone,Eq,PartialEq)]
pub enum ModelReceiver {
    ArcRwLock,
    ArcMutex,
    Slf,
}

impl ModelReceiver{
    
    /// for Type, TypePath
    pub fn type_path_rcvr_variants<T>(&self, ty: &T,lib: &Lib ) -> Vec<T> 
    where T: ToTokens + Parse
    {
        match self {
            Self::ArcRwLock => { Self::get_arc_wraped(RWLOCK,ty, lib)},
            Self::ArcMutex  => { Self::get_arc_wraped(MUTEX,ty, lib) },
            Self::Slf       => { vec![ parse_quote!(#ty) ] },
        }
    }

    pub fn get_arc_wraped<T,D>(lock:D, ty_path: &T, lib: &Lib) -> Vec<T> 
        where T: ToTokens + Parse,
              D: Display,
    {
        let arcs = Self::get_paths(ARC, &Lib::Std);
        let locks  = Self::get_paths(&lock, lib);

        arcs.iter().flat_map(|a|
            locks.iter().map(move |l| 
                parse_quote!(#a<#l<#ty_path>>)
            )
        )
        .collect::<Vec<T>>()
    }
    pub fn get_paths<D:Display>(ty_name: D , lib: &Lib) ->  Vec<Path>{
        let ident_lib = &format_ident!("{lib}");
        let ident_type = &format_ident!("{ty_name}");

        let mut paths = vec![
            
            parse_quote!{ #ident_lib::sync::#ident_type },
            parse_quote!{ sync::#ident_type },
            parse_quote!{ #ident_type }
        ];

        if lib.is_std() {
            let std_spec = std::iter::once(parse_quote!{ ::std::sync::#ident_type });
            paths.extend(std_spec);
        }

        paths
    }


    pub fn get_model_type<T>(&self,ty: &T, lib: &Lib) -> T 
    where T: ToTokens + Parse
    {   
        let crate_ident = &format_ident!("{lib}");
        let lead_colon = if lib.is_std() { quote!{ :: } } else { quote!{} };
        match self{
            Self::ArcRwLock => parse_quote!( ::std::sync::Arc< #lead_colon #crate_ident ::sync::RwLock<#ty>> ),
            Self::ArcMutex  => parse_quote!( ::std::sync::Arc< #lead_colon #crate_ident ::sync::Mutex<#ty>> ),
            Self::Slf => parse_quote!(#ty), 
        }
    }

    pub fn get_model_wrap<T>(&self,ty: &T, lib: &Lib ) -> TokenStream 
    where T: ToTokens 
    {   
        let crate_ident = &format_ident!("{lib}");
        let lead_colon = if lib.is_std() { quote!{ :: } } else { quote!{} };
        match self{
            Self::ArcRwLock => quote!{ ::std::sync::Arc::new( #lead_colon #crate_ident::sync::RwLock::new(#ty)) },
            Self::ArcMutex  => quote!{ ::std::sync::Arc::new( #lead_colon #crate_ident::sync::Mutex::new(#ty)) },
            Self::Slf => quote!{#ty},
        }
    }

    pub fn is_rcvr(&self, check_ty: &Type, act_ty: &Type, lib: &Lib ) -> bool {

        let big_slf: Type = parse_quote!{ Self };

        let rcvr_type_vrnts = self.type_path_rcvr_variants(act_ty,lib);  
        let rcvr_self_vrnts = self.type_path_rcvr_variants( &big_slf,lib); 

        rcvr_type_vrnts.iter().any(|p| p.eq(check_ty)) ||
        rcvr_self_vrnts.iter().any(|p| p.eq(check_ty))
    }



    pub fn second_sort(&self, mut met: ImplItemFn , actor_type: &TypePath, lib: &Lib ) -> ModelMethod1 {

        if let Some(input_rcvr) = met.sig.inputs.first_mut() {

            // this may be a poor design choice as it was not originally intended
            // we will allow family receiver notation for basic actor macros as well
            // (+) it will create a script variant ( which could be handy for TS macros )
            
            // we'll extend some consistency for now 
            // uncomment the condition if it prooves to be an unnecessary or bad idea 
            // if !self.is_slf(){
                if let FnArg::Typed(pat_ty) = input_rcvr.clone() {
                    if let syn::Pat::Ident( PatIdent{ ident,mutability,.. } ) = &*pat_ty.pat {
                        if ident == crate::ACTOR {
                            let actor_type = &Type::Path(actor_type.clone());
                            if let Type::Reference( TypeReference { elem, mutability,.. } ) = &*pat_ty.ty {

                                if let Some(mut_token)  = mutability {
                                    // allowing  family actors to mutate the Arc<Mutex<Actor>>
                                    // will lead to each member of the family serving their own version 
                                    // of wrapped actor
                                    // only references acceptable
                                    // ( actor : &    Arc<Mutex<Actor>> ) <--- acceptable
                                    // ( actor : &mut Arc<Mutex<Actor>> ) <--- not acceptable
                                    if !self.is_slf() {
                                        abort!(mut_token,error::NOT_ALLOW_FAMILY_DIRECT_MUT_REF);
                                    }
                                }

                                if self.is_rcvr(&*elem, actor_type, lib){
                                    *input_rcvr = parse_quote!( & #mutability self );
                                    // ref
                                    return ModelMethod1::Ref( met, mutability.is_some() );
                                }
                            }

                            if self.is_rcvr(&*pat_ty.ty,actor_type, lib){
                                // since the live instance in method is not mutated 'self' is enough
                                // but we keep it consistent adding the mutability if present
                                // which will be removed later by a specialised method 
                                *input_rcvr = parse_quote!( #mutability self );
                                // self consumming   
                                return ModelMethod1::Slf(met);
                            }
                        } 
                    } 
                }
            // }   
        } 

        // static 
        return ModelMethod1::Stat(met);
    
    }

    pub fn remove_mut(&self, met: &mut ImplItemFn ){
        if let Some( input_rcvr ) = met.sig.inputs.first_mut(){
            if let FnArg::Receiver(Receiver{mutability,..}) = input_rcvr  {
               *mutability = None; 
            }
        }
    }

    pub fn is_slf(&self) -> bool {
        if let Self::Slf = self {
            return true;
        }
        false
    }

    pub fn get_lock(&self, lib: &Lib, const_vars: &ConstVars, is_mut: bool , is_stat: bool  ) -> TokenStream {
        if is_stat { return quote!{};}
        let ConstVars{ actor,..} = &const_vars;

        let unwrap_await = if lib.is_std() { quote!{.unwrap()} } else { quote!{ .await } };

        match self {

            Self::ArcMutex => {
                let mut_token = if is_mut { quote!{mut} } else { quote!{} };
                quote!{ let #mut_token #actor = actor.lock() #unwrap_await; }
            },
            Self::ArcRwLock => {
                let (mut_token, read_write) = 
                if is_mut {
                    ( quote!{mut}, quote!{ write } )
                } else {
                    ( quote!{}, quote!{ read } )
                };
                quote!{ let #mut_token #actor = #actor . #read_write () #unwrap_await; }
            },
            Self::Slf => { quote!{} },
        }
    }
}

impl Default for ModelReceiver{
    fn default() -> Self {
        Self::Slf
    }
}

