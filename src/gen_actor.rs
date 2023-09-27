use crate::attribute::{ActorAttributeArguments,AALib,AAExpand};
use crate::name;
use crate::method;
use crate::error;
use crate::generics;
use crate::model;

use proc_macro_error::abort;
use syn::{Ident,Signature,ItemImpl,Visibility };
use quote::{quote,format_ident};
use proc_macro2::TokenStream;

pub fn live_static_method( 
    actor_name: &Ident,
         ident: Ident, 
           vis: Visibility,
       mut sig: Signature,
          args: TokenStream,
     live_mets: &mut Vec<(Ident,TokenStream)> ) {

    method::change_signature_refer(&mut sig);
    let await_call = sig.asyncness.as_ref().map(|_|quote!{.await});
    let stat_met = quote! {
        #vis #sig {
            #actor_name::#ident #args #await_call
        }
    };
    live_mets.push((ident,stat_met));
}


// returns  (code,edit) TokenStreams 
pub fn actor_model( aaa: ActorAttributeArguments, item_impl: &ItemImpl, mac: AAExpand, mut new_vis: Option<Visibility> ) 
->  crate::model::ActorModelSdpl{ //(TokenStream, TokenStream){
    
    let mut script_def;
    let mut script_mets = vec![];
    let mut script_trts = vec![];
  
    let mut live_def;
    let mut live_mets = vec![];
    let mut live_trts = vec![];


    let mut script_fields   = vec![];
    let mut direct_arms     = vec![];
    let mut debug_arms      = vec![];


    let (actor_name,
        actor_type,
        generics) = name::get_ident_type_generics(item_impl);
    


    let ( mut actor_methods, 
          mut met_new) =
         method::get_methods( &actor_type,item_impl.clone(),aaa.assoc ,&mac);


    
    let mut model_generics = generics.clone();
    let actor_ty_generics  = generics.split_for_impl().1;

    let ( impl_generics,
            ty_generics,
           where_clause ) = {

        let mut sigs = actor_methods.iter_mut().map(|m| m.get_mut_sig()).collect::<Vec<_>>();

        if met_new.is_some() {
            let mut mn = met_new.unwrap();
            sigs.push(mn.get_mut_sig());
            generics::get_parts( &mut model_generics, sigs);

            met_new = Some(mn);

        } else {
            generics::get_parts( &mut model_generics, sigs);
        }
        model_generics.split_for_impl()

    };
    


    // Giving a new name if specified 
    let cust_name   = if aaa.name.is_some(){ aaa.name.clone().unwrap() } else { actor_name.clone() }; 
    
    let script_name = &name::script(&cust_name);
    let live_name   = &name::live(&cust_name);
    

    let (live_field_sender,
        play_input_receiver, 
        new_live_send_recv , 
        live_meth_send_recv, 
        script_field_output, 
        live_send_input,
        live_recv_output ) = model::channels( &aaa.lib, &aaa.channel, &script_name,&live_name, &ty_generics);


    
    let direct_async_decl = 
    if actor_methods.iter().any(|x| x.is_async()) { 
        Some(quote!{async})
    } else { None };

    let play_async_decl   = 

        match &aaa.lib {
            AALib::Std => {
                if direct_async_decl.is_some(){ 
                    let pos = actor_methods.iter().position(|x| x.is_async()).unwrap();
                    error::abort_async_no_lib(&actor_name,&actor_methods[pos]);
                } 
                None
            },
            _ => { Some(quote!{async}) },
        };



    for method in actor_methods.clone() {
        
        let (mut sig, script_field_name) = method.get_sig_and_field_name();

        let await_call = sig.asyncness.as_ref().map(|_|quote!{.await});
        method::to_async(&aaa.lib, &mut sig);

        let error_send = error::direct_send(&script_name,&script_field_name);

        // Debug arm
        let add_arm = | debug_arms: &mut Vec<TokenStream>,ident: &Ident | {

            let str_field_name = format!("{}::{}",script_name.to_string() ,ident.to_string());

            let debug_arm = quote! {
                #script_name :: #script_field_name {..} => write!(f, #str_field_name),
            };
            debug_arms.push(debug_arm);
        };

        match method {

            method::ActorMethod::Io   { vis, ident, stat,  arguments, output,.. } => {
                let (args_ident,args_type) = method::arguments_ident_type(&arguments);
                
                if stat {
                    live_static_method(&actor_name,ident, vis, sig, args_ident,&mut live_mets)
                }
                else {
                    // Debug Arm push
                    add_arm(&mut debug_arms, &script_field_name);

                    // Direct Arm
                    let arm_match        = quote! { 
                        #script_field_name { input: #args_ident,  output: send }
                    };
                    let direct_arm       = quote! {
                        #script_name :: #arm_match => {send.send( actor.#ident #args_ident #await_call ) #error_send ;}
                    };
                    direct_arms.push(direct_arm);
                    
                    // Live Method
                    let live_met      = quote! {

                        #vis #sig {
                            #live_meth_send_recv
                            let msg = #script_name :: #arm_match;
                            #live_send_input
                            #live_recv_output
                        }
                    };

                    live_mets.push((ident,live_met));

                    // Script Field Struct
                    let output_type      = (&*script_field_output)(output);

                    let script_field = quote! {
                        #script_field_name {
                            input: #args_type,
                            #output_type
                        }
                    };

                    script_fields.push(script_field);
                }
            },
            method::ActorMethod::I    { vis, ident, arguments ,..} => {
                
                let (args_ident,args_type) = method::arguments_ident_type(&arguments);
                
                // Debug Arm push
                add_arm(&mut debug_arms, &script_field_name);

                // Direct Arm
                let arm_match = quote!{ 
                    #script_field_name{ input: #args_ident }
                };
    
                let direct_arm = quote!{
                    #script_name::#arm_match => {actor.#ident #args_ident #await_call;},
                };
                direct_arms.push(direct_arm);

                // Live Method
                let live_met = quote!{
    
                    #vis #sig {
                        let msg = #script_name::#arm_match ;
                        #live_send_input
                    }
                };
                live_mets.push((ident,live_met));
            


                // Script Field Struct
                let script_field = quote!{
                    #script_field_name {
                        input: #args_type,
                    }
                };
                script_fields.push(script_field);

            },
            method::ActorMethod::O    { vis, ident, stat, output ,..} => {
                let (args_ident,_) = method::arguments_ident_type(&vec![]);

                if stat {
                    live_static_method(&actor_name,ident, vis, sig, args_ident,&mut live_mets)
                }
                else {
                    
                    // Debug Arm push
                    add_arm(&mut debug_arms, &script_field_name);

                    // Direct Arm
                    let arm_match = quote!{ 
                        #script_field_name{  output: send }
                    };
        
                    let direct_arm = quote!{
                        #script_name::#arm_match => {send.send(actor.#ident #args_ident #await_call) #error_send ;}
                    };
                    direct_arms.push(direct_arm);



                    // Live Method
                    let live_met = quote!{
                    
                        #vis #sig {
                            #live_meth_send_recv
                            let msg = #script_name::#arm_match ;
                            #live_send_input
                            #live_recv_output
                        }
                    };
                    live_mets.push((ident, live_met));
                
                    // Script Field Struct
                    let output_type  = (&*script_field_output)(output);

                    let script_field = quote!{
                        #script_field_name {
                            #output_type
                        }
                    };
                    script_fields.push(script_field);
                }
            },
            method::ActorMethod::None { vis, ident ,..} => {

                // Debug Arm push
                add_arm(&mut debug_arms, &script_field_name);

                // Direct Arm
                let arm_match = quote!{ 
                    #script_field_name {} 
                };
    
                let direct_arm = quote!{
                    #script_name::#arm_match => {actor.#ident () #await_call;},
                };
                direct_arms.push(direct_arm);

                // Live Method
                let live_met = quote!{
                
                    #vis #sig {
                        let msg = #script_name::#arm_match ;
                        #live_send_input
                    }
                };
                live_mets.push((ident,live_met));
            
                // Script Field Struct
                let script_field = quote!{
                    
                    #script_field_name {}
                };
                script_fields.push(script_field);
            },
        }
    } 


    // METHOD NEW

    if AAExpand::Actor.eq(&mac) { 

        if met_new.is_none() {

            let msg = format!("Can not find public/restricted  method `new` or `try_new` for {:?} object.",actor_name.to_string());
            let (note,help) = error::met_new_note_help(&actor_name);
            abort!(item_impl,msg;note=note;help=help);
        }
        
        // Change visibility of model methods 
        new_vis = met_new.as_ref().map(|m| m.vis.clone());

        let met_new         = met_new.unwrap();
        let new_sig             = &met_new.new_sig;
        let func_new_name           = &new_sig.ident;
        let (args_ident, _ )   = method::arguments_ident_type(&met_new.get_arguments());
        let live_var                 = format_ident!("actor_live");
        let unwrapped          = met_new.unwrap_sign();
        let return_statement   = met_new.live_ret_statement(&live_var);
        let vis                = &met_new.vis.clone();

        let live_new_spawn = |play_args:TokenStream| {
            match aaa.lib {
                AALib::Std      => {
                    quote!{ std::thread::spawn(|| { #script_name :: play(#play_args) } );}
                },
                AALib::Smol     => {
                    quote!{ smol::spawn( #script_name :: play(#play_args) ).detach();} 
                },
                AALib::Tokio    => {
                    quote!{ tokio::spawn( #script_name :: play(#play_args) );}
                },
                AALib::AsyncStd => {
                    quote!{ async_std::task::spawn( #script_name :: play(#play_args) );}
                },
            }
        };

        let (init_actor, play_args) = {
            let id_debut_name = if aaa.debut.active() {quote!{ ,debut,name}} else {quote!{}};
            ( quote!{ Self{ sender #id_debut_name } }, quote!{ receiver, actor } )
        };

        let spawn = live_new_spawn(play_args);
        let turbofish = ty_generics.as_turbofish();
        let (id_debut,id_name)  =  
        if aaa.debut.active() {
            (quote!{let debut =  #script_name #turbofish ::debut();},
                quote!{let name  = String::from("");})
        } else { (quote!{}, quote!{}) };
        
        let func_new_body = quote!{

            #vis #new_sig {
                let actor = #actor_name:: #func_new_name #args_ident #unwrapped;
                #new_live_send_recv
                #id_debut
                #id_name
                let #live_var = #init_actor;
                #spawn
                #return_statement
            }
        };
        live_mets.insert(0,(new_sig.ident.clone(),func_new_body));
    };

    // LIVE INTER METHODS AND TRAITS
    // model::debut()
    if aaa.debut.active(){
        model::debut(
            &mut live_mets,
            &mut live_trts,
            &mut script_mets,
            &live_name,
            &new_vis,
            &ty_generics,
            &where_clause
        )
    }

    // SCRIPT DEFINITION
    script_def = {

        quote! {
            #new_vis enum #script_name #ty_generics #where_clause {
                #(#script_fields),*
            }
        }
    };

    // DIRECT
    {

        script_mets.push((format_ident!("direct"),
        quote!{
            #new_vis #direct_async_decl fn direct (self, actor: &mut #actor_type #actor_ty_generics ) {
                match self {
                    #(#direct_arms)*
                }
            }
        }));
    }


    // PLAY
    {
        let direct_await  = direct_async_decl.as_ref().map(|_| quote!{.await});
        let recv_await=  play_async_decl.as_ref().map(|_| quote!{.await});
        let end_of_play = error::end_of_life(&actor_name); 


        let play_method = {
        
            let ok_or_some = match aaa.lib {
                AALib::Tokio => quote!{Some},
                _ => quote!{Ok}
            };
            quote! {
                #new_vis #play_async_decl fn play ( #play_input_receiver mut actor: #actor_type #actor_ty_generics ) {
                    while let #ok_or_some (msg) = receiver.recv() #recv_await {
                        msg.direct ( &mut actor ) #direct_await;
                    }
                    #end_of_play
                }
            }
        };
        script_mets.push(( format_ident!("play"), play_method ));
    }
    
    // SCRIPT TRAIT (Debug)
    {   

        let str_script_name = script_name.to_string();
        let body = 
        if debug_arms.is_empty() { 
            quote!{ write!(f, #str_script_name )} 
        } else {
            quote!{ match self { #(#debug_arms)* } }
        };
        script_trts.push((format_ident!("Debug"),
        quote! {
            impl #ty_generics std::fmt::Debug for #script_name #ty_generics #where_clause {
        
                fn fmt( &self, f: &mut std::fmt::Formatter<'_> ) -> std::fmt::Result {
                    #body
                }
            }
        }));
    }


    // LIVE DEFINITION
    live_def = {
        let (debut_field, name_field) = if aaa.debut.active() {
            ( quote!{ pub debut: std::sync::Arc<std::time::SystemTime>,},
            quote!{ pub name: String,} )
        } else { (quote!{}, quote!{})};   

        quote!{
            #[derive(Clone)]
            #new_vis struct #live_name #ty_generics #where_clause {
                #live_field_sender
                #debut_field
                #name_field
            }
        }
    };
    
    if AAExpand::Group.eq(&mac){
        //we have to extract play from the model
        // ???
        let play = format_ident!("play");

    }
    crate::model::ActorModelSdpl {
        name:          cust_name,
        mac:         mac.clone(),
        edit:           aaa.edit,
        generics: model_generics,
        script: ( script_def, script_mets, script_trts ),
        live:   (   live_def,   live_mets,   live_trts ),
    }
}

pub fn macro_actor_generate_code(
        aaa: ActorAttributeArguments, 
        item_impl: ItemImpl ) 
        -> ( TokenStream, TokenStream ) {


    let mut act_model = actor_model( aaa,&item_impl,AAExpand::Actor,None);

    let (mut code,edit) = act_model.split_edit();
    
    // abort!(item_impl,code.to_string());
    
    code = quote!{

        #item_impl
        #code
    };
    (code,edit)
    
}










