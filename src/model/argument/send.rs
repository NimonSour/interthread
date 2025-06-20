use crate::model::{ add_bounds, ImplVars, OneshotChannel, ConstVars, ModelOutput, MethodNew, ModelGenerics ,GenWork};
use quote::quote;
use proc_macro2::TokenStream;
use syn::{Generics,ReturnType,Type,parse_quote,FnArg,Signature };
use proc_macro_error::abort;

#[derive(Clone)]
pub struct TySend {
    send: Option<syn::Meta>,
}

impl TySend {

    pub fn set_send(&mut self, meta: syn::Meta ){
        self.send = Some(meta); 
    }

    pub fn is_send(&self)-> bool{
        !self.send.is_some()
    }

    pub fn is_not_send(&self)-> bool{
        self.send.is_some()
    }
    
    pub fn get_send(&mut self) -> Option<syn::Meta> {
        self.send.clone()
    }

    pub fn get_play_actor_init_block( impl_vars: &ImplVars, play_actor_init: &TokenStream ) -> TokenStream {

        let ImplVars{oneshot, met_new, const_vars, script_name,
            direct_play_mut_token,..} = impl_vars;
        let ConstVars{play,short_actor,short_error,actor,.. } = const_vars;
        if let Some(met_new) = met_new {

             let init_block = match &met_new.mod_output {

                ModelOutput::Result(path,_ ) => {
                    let ok_msg = quote!{ #path :: Ok (())};
                    let ok_send_call = oneshot.send_call(ok_msg,script_name,play);

                    let err_msg = quote!{ #path :: Err (#short_error)};
                    let err_send_call = oneshot.send_call(err_msg,script_name,play);

                    quote!{

                        let #direct_play_mut_token #actor = 

                            match  #play_actor_init {
                                #path :: Ok ( #short_actor ) => { #ok_send_call ; #short_actor },
                                #path :: Err (#short_error ) => { let _ = #err_send_call ; return }
                            };

                    }

                },
                ModelOutput::Option(path)   => {
                    let some_msg = quote!{ #path :: Some (())};
                    let some_send_call = oneshot.send_call(some_msg,script_name,play);

                    let none_msg = quote!{ #path :: None };
                    let none_send_call = oneshot.send_call(none_msg,script_name,play);

                    quote!{

                        let #direct_play_mut_token #actor = 

                            match  #play_actor_init {
                                #path :: Some ( #short_actor ) => { #some_send_call ; #short_actor },
                                #path :: None  => { let _ = #none_send_call ; return }
                            };

                    }
                },
                ModelOutput::None       => {

                    let send_call = oneshot.send_call(quote!{()},script_name,play);

                    quote!{

                        let #direct_play_mut_token #actor = #play_actor_init ;
                        let _ = #send_call ;

                    }
                }
            };
            return init_block;
        } else { 
            panic!("InternalError model::argument::send::TySend::get_play_actor_init Expecting this call when 'met_new' is Some.")
        }


    }
    
    /// for signature method 'new'; extract ReturnType and return a Type for oneshot::Sender ( when model is !Send ) 
    pub fn get_oneshot_fn_arg_sender_type( met_new_sig: &Signature, oneshot: &OneshotChannel ) -> FnArg {

        let ty = match &met_new_sig.output {
            ReturnType::Default => {abort!(met_new_sig,"InternalError.'argument::send::TySend::get_oneshot_fn_arg_sender_type'. Unexpected return type for method 'new'.")},
            ReturnType::Type(_,ty) => {*ty.clone()},
        };

        let old: Type = parse_quote!(Self);
        let new: Type = parse_quote!(());
        let new_ty = crate::model::replace(&ty,&old,&new);

        syn::parse2(oneshot.pat_type_send(&new_ty)).unwrap()
    }

    /// Returns updated Generics for method 'play'
    pub fn add_model_gen_bounds_and_update_play_gen(&self, met_new: Option<&mut MethodNew>, mod_gen: &mut ModelGenerics ) -> Option<Generics> {

        if self.is_not_send(){
            let mut play_gen = mod_gen.private_gen.clone();
            let met_new = met_new.unwrap();
            
            // extend params
            play_gen.params.extend( met_new.met.sig.generics.params.iter().cloned());
            // extend predicates
            if let Some(w_c) = &met_new.met.sig.generics.where_clause {
                play_gen.make_where_clause().predicates.extend(w_c.predicates.iter().cloned());
            }

            let new_sig = met_new.met.sig.clone();
            // add model bounds for 'live_gen' and 'met_new_gen' if they are present in method arguments
            for gen in [&mut met_new.met.sig.generics, &mut mod_gen.live_gen ]{
                let mut gen_work = GenWork::new(gen,self.is_send());
                gen_work.retain_i(&new_sig);
                for param in &gen_work.difference(){
                    add_bounds(param,gen);
                }
            }
            return Some(play_gen);
        }
        None
    }

}



impl Default for TySend {
    fn default()  -> TySend {
        Self{ send: None } 
    }
}