
use crate::model::{generics::turbofish, name, ActorAttributeArguments as AAA, ActorModelSdpl,
    ConstVars, Cont, ImplVars, Lib, Mac, ModelPart, ModelSdpl, MpscChannel };

use syn::{parse_quote, Generics, ItemImpl, Visibility };
use quote::{format_ident,quote};
use super::{FamilyModelSdpl, MethodNew, ModelPhantomData, ModelReceiver};


pub fn generate_model( aaa: AAA, item_impl: &ItemImpl) -> ModelSdpl {

    if aaa.mac == Mac::Family { 
        generate_family(aaa,item_impl)
    } else {
        generate_actor(aaa,item_impl)
    }
}

pub fn generate_family(mut aaa: AAA, item_impl: &ItemImpl) -> ModelSdpl {
    let const_vars = &ConstVars::new();
    let ConstVars{ actor, debut, ..} = const_vars;

    let mut member_idents_args = vec![ actor ];
    let mut debut_invoke = None;

    if aaa.debut.active {
        member_idents_args.push(&debut);
        debut_invoke = Some( quote!{ 

            let #debut  = Self :: #debut () ; 
        });
    }
    
    let mut met_new = None;

    let mut member_fields  = vec![];
    let mut member_invoks  = vec![];
    let mut member_init_vars     = vec![];

    let mut actor_mems = vec![];

    for (ident, aaa) in std::mem::take(&mut aaa.members) {

        if let ModelSdpl::Actor(ams) = generate_actor( aaa, item_impl ){

            if met_new.is_none() { met_new = ams.met_new.clone()}

            let field = &name::family_field_name(&ident);

            let (_, member_live_type ) = ams.get_script_live_type();

            member_fields.push( quote!{
                #field : #member_live_type 
            });

            let member_live_turbo = turbofish::from_type_path(&member_live_type);

            member_invoks.push(quote!{
                let #field = #member_live_turbo :: new( #(#member_idents_args .clone()),* ); 
            });

            member_init_vars.push( field.clone() );

            actor_mems.push(ams);
        }
    }

    let MethodNew{ ref args_idents,mod_output,mut met,
        turbo_gen,..} = met_new.unwrap();
    let vis = met.vis.clone();
    
    let(actor_name,type_path,mut gen) = crate::model::get_ident_type_generics(item_impl);
    
    let actor_turbo = turbofish::from_type_path(&type_path);
    let family_name  = &if let Some(name) = &aaa.name { name.clone() } else { actor_name }; 
    let family_name = &name::family(family_name);
    

    crate::model::generics::add_model_bounds(&mut gen);
    let (  f_impl_generics,
           f_ty_generics,
           f_where_clause ) = gen.split_for_impl();

    let family_impl_block: ItemImpl  = parse_quote!( impl #f_impl_generics  #family_name #f_ty_generics #f_where_clause {} );

    // Defenition 
    let family_def = 
        
    &quote!{ 
        #vis struct #family_name #f_impl_generics #f_where_clause {
            #( pub #member_fields ),*
        }
    };

    // Method new 
    let met_new_ident          = &met.sig.ident;
    let unwrapped = mod_output.unwrap_sign();
    let arc_wraped_actor  = aaa.mod_receiver.get_model_wrap(&quote!{#actor}, &aaa.lib);
    let init_live         = quote!{  Self { #(#member_init_vars),* } };
    let return_statement  = mod_output.return_ok(&init_live);

    met.block = parse_quote! {
        {
            let #actor = #actor_turbo :: #met_new_ident #turbo_gen (#(#args_idents),*) #unwrapped;
            let #actor = #arc_wraped_actor ;
            #debut_invoke

            #(#member_invoks;)*
            #return_statement
        } 
    };

    let mut family_mets = vec![]; 

    family_mets.push((met_new_ident.clone(),aaa.show.parse_method(&met)));

    // generate method debut 
    if aaa.debut.active { 
        family_mets.push((debut.clone(),aaa.show.parse_method(&aaa.debut.get_method_debut(const_vars))));
    }

    let mod_part = 
    ModelPart::new(
        Some(aaa.show.parse_model_part( family_def, &family_mets, &vec![])),
        family_mets.clone(),
        vec![],
        family_impl_block.clone(),
    );

    ModelSdpl::Family(
        FamilyModelSdpl {
            aaa,
            live: mod_part,
            actors: actor_mems,
        }
    )
}

pub fn generate_actor( aaa: AAA, item_impl: &ItemImpl) -> ModelSdpl {

    // mac model values 
    let mut impl_vars = ImplVars::new(&aaa, &item_impl);
    let met_new = impl_vars.met_new.take();
    let mut cont = super::Cont::new(&aaa, &impl_vars);

    // generate raw model parts 
    cont.to_raw_parts( &impl_vars, &aaa);


    let ImplVars { 
        vis,
        async_decl, actor_turbo,const_vars, model_actor_type,
        script_name, script_turbo, live_name, mod_gen,
        direct_play_mut_token,.. } = &impl_vars;
    
    let ConstVars{
        actor, play, direct,
        debut, msg,
        sender,receiver,name, .. } = const_vars;


    let (  s_impl_generics,
           s_ty_generics,
           _s_where_clause  ) =  mod_gen.script_gen.split_for_impl();

    let (  p_impl_generics,
           p_ty_generics,
           _p_where_clause  ) =  mod_gen.private_gen.split_for_impl();

    let (  l_impl_generics,
           _l_ty_generics,
           l_where_clause  ) = mod_gen.live_gen.split_for_impl();
    
    let ModelPhantomData{ phantom_fields,phantom_invoks,phantom_init_vars} = &mod_gen.phantom_data;

    // Call for method new 
    {   
        if aaa.mac == Mac::Actor {
    
            let MethodNew{ ref args_idents,mod_output,mut met,
                turbo_gen,..} = met_new.clone().unwrap();
            let met_new_ident       = &met.sig.ident;
            let unwrapped      = mod_output.unwrap_sign();
            let mut debut_decl_call = None;
            let mut debut_decls = None;
            let mut debut_ident = None;

            // debut variables 
            if aaa.debut.active {
        
                if aaa.mod_receiver == ModelReceiver::Slf {
                    debut_decl_call = Some( quote!{
                        let #debut = #script_turbo ::#debut();
                    });
                }
            
                debut_ident = Some( quote!{
                    #debut
                });
            
                debut_decls = Some( quote!{
                    #debut : ::std::sync::Arc::new( #debut ),
                    #name  : ::std::string::String::new(),
                });
            }
        
            let init_live = quote!{
                Self { #sender ,  #debut_decls #phantom_init_vars  }
            };
        
            let play_args = quote!{ #receiver, #actor, #debut_ident };
            let spawn = aaa.lib.method_new_spawn(&play_args,script_turbo, &p_ty_generics );
            let return_statement   = mod_output.return_ok(&init_live);
            let channel_decl = &impl_vars.mpsc.declaration;


            if aaa.mod_receiver.is_slf(){
            
                met.block = syn::parse_quote!{
                    {
                        let #actor = #actor_turbo :: #met_new_ident #turbo_gen (#(#args_idents),*) #unwrapped;
                        #debut_decl_call
                        #phantom_invoks
                        #channel_decl
                        #spawn
                        #return_statement
                    }
                };
            
            } else {
        
                met.block = syn::parse_quote!{
            
                    {
                        #phantom_invoks
                        #channel_decl
                        #spawn
                        #return_statement
                    }
                };
            
                // change vis
                met.vis = Visibility::Inherited;
            
                met.sig.inputs.clear();
                met.sig.inputs.push( parse_quote!( #actor : #model_actor_type ) );
            
                if aaa.debut.active {
                    met.sig.inputs.push( parse_quote!( #debut: ::std::time::SystemTime ) );
                }

                met.sig.generics = Generics::default();
            }
    
        cont.insert_live_met(met_new_ident,&met);
    
        }
    }


    // LIVE INTER METHODS AND TRAITS
    if aaa.debut.active(){
        aaa.debut.impl_debut(&mut cont, &impl_vars,&aaa);
    }
    

    // SCRIPT DEFINITION
    let script_def = {
        let Cont{ script_fields,..} = &cont;
        quote! {
            enum #script_name #s_impl_generics {
                #(#script_fields),*
            }
        }
    };        


    // DIRECT
    {
        let Cont{direct_arms,..} = &cont;

        let mut_token = if aaa.mod_receiver.is_slf(){ quote!{ mut } } else { quote!{} };

        cont.push_script_met(direct,
        & quote!{
            #async_decl fn #direct #p_impl_generics (self, #actor: & #mut_token #model_actor_type ) {
                match self {
                    #(#direct_arms),*
                }
            }
        });
    }


    // PLAY
    {
        let debut_pat_type = if aaa.debut.active {quote!{,#debut: ::std::time::SystemTime }} else { quote!{} };

        let Cont{ play_while_block,..} = &cont;
        let ImplVars{ async_decl,mpsc, model_actor_type,..} = &impl_vars;
        let MpscChannel{pat_type_receiver,..} = mpsc;
        let await_call  = impl_vars.get_await_call();
        let play_method = {
        
            let ok_or_some = match aaa.lib {
                Lib::Tokio => quote!{::std::option::Option::Some},
                _ => quote!{::std::result::Result::Ok}, 
            };
            let msg_direct_call = quote!{ #msg.#direct ( & #direct_play_mut_token #actor ) #await_call; };
            let while_block_code = (*play_while_block)(msg_direct_call);
            quote! {
                #async_decl fn #play #p_impl_generics( #pat_type_receiver #direct_play_mut_token #actor: #model_actor_type #debut_pat_type ) {
                    while let #ok_or_some (#msg) = #receiver.recv() #await_call {
                        #while_block_code
                    }
                }
            }
        };
        cont.push_script_met( play, &play_method );
    }

    // TRAIT DEBUG for Script
    if aaa.trait_debug {   

        let str_script_name = script_name.to_string();
        let body = 
        if cont.debug_arms.is_empty() { 
            quote!{ write!(f, #str_script_name )} 
        } else {
            let debug_arms = &cont.debug_arms;
            quote!{ match self { #(#debug_arms)* } }
        };
        cont.push_script_trt(&format_ident!("Debug"),
        &quote! {
            impl #s_impl_generics std::fmt::Debug for #script_name #s_ty_generics {
            
                fn fmt( &self, f: &mut std::fmt::Formatter<'_> ) -> std::fmt::Result {
                    #body
                }
            }
        });
    }

    // LIVE DEFINITION
    let live_def = {

        let MpscChannel{pat_type_sender,..} = &impl_vars.mpsc;
        let Cont{ live_clone_attr,..} = &cont;

        let debut_fields = if aaa.debut.active {
            quote!{
                debut : ::std::sync::Arc<::std::time::SystemTime>,
                name  : ::std::string::String,
            }
        } else { quote!{} };


        quote!{
            #live_clone_attr
            #vis struct #live_name #l_impl_generics #l_where_clause {
                #pat_type_sender
                #debut_fields
                #phantom_fields
            }
        }
    };

   
    ModelSdpl::Actor(
        ActorModelSdpl{
            aaa,
            met_new,
            script: cont.get_script_part(&script_def),
            live: cont.get_live_part(&live_def),
        }
    )
}

