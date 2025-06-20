
use crate::model::{generics::{self,turbofish}, name, ActorAttributeArguments as AAA, ActorModelSdpl,
    ConstVars, Cont, ImplVars, Mac, ModelPart, ModelSdpl, MpscChannel };

use syn::{parse_quote, Generics, ItemImpl, Visibility };
use quote::{format_ident,quote};
use super::{FamilyModelSdpl, MethodNew, ModelPhantomData, TySend};


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
    

    generics::add_model_bounds(&mut gen);
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
    let impl_vars = ImplVars::new(&aaa, &item_impl);
    let mut cont = super::Cont::new(&aaa, &impl_vars);

    // generate raw model parts 
    cont.to_raw_parts( &impl_vars, &aaa);


    let ImplVars { 
        vis,
        async_decl, actor_turbo,const_vars, model_actor_type,
        script_name, script_turbo, live_name, mod_gen,
        direct_play_mut_token,oneshot,mpsc,
        not_send_play_gen,.. } = &impl_vars;
    
    let ConstVars{
        actor, play, direct,
        debut, msg,
        sender,receiver,.. } = const_vars;


    let (   s_impl_generics,
            s_ty_generics,
            _s_where_clause  ) =  mod_gen.script_gen.split_for_impl();

    let (   d_impl_generics,
            _d_ty_generics,
            d_where_clause  ) =  mod_gen.private_gen.split_for_impl();

    let (   p_impl_generics,
            p_ty_generics,
            p_where_clause  ) = if let Some(play_gen) = not_send_play_gen{ play_gen.split_for_impl() } else { mod_gen.private_gen.split_for_impl() };

    let (   l_impl_generics,
            _l_ty_generics,
            l_where_clause  ) = mod_gen.live_gen.split_for_impl();
    
    let ModelPhantomData{ phantom_fields,phantom_invoks,phantom_init_vars} = &mod_gen.phantom_data;
    

    // parts to share for 'new' and 'play'
    let mut play_actor_init = quote!{};
    let mut play_fn_args = vec![];
    
    // mpsc::Receiver 
    play_fn_args.push(mpsc.pat_type_receiver.clone());

    // Send
    if aaa.ty_send.is_send() {
        // actor 
        play_fn_args.push(
            syn::parse2(quote!{ #direct_play_mut_token #actor: #model_actor_type }).unwrap()
        )
    } 
    // debut
    if aaa.debut.active {
        play_fn_args.push(
            syn::parse2(quote!{ #debut: ::std::time::SystemTime }).unwrap()
        )
    } 

    // Call for method new 
    {   
        if aaa.mac == Mac::Actor {

            let MethodNew{ ref args_idents,mod_output,mut met,turbo_gen,..} = impl_vars.met_new.clone().unwrap();
            let met_new_ident = &met.sig.ident;
            let unwrapped = mod_output.unwrap_sign();
            let debut_decl_call = if aaa.mod_receiver.is_slf(){ aaa.debut.get_debut_decl_call(script_turbo, const_vars)} else { quote!{} };
            let debut_filds_init = aaa.debut.get_debut_filds_init(&const_vars);
            let init_live = quote!{ Self { #sender, #debut_filds_init #phantom_init_vars } };
        
            let return_statement   = mod_output.return_ok(&init_live);
            let channel_decl = &impl_vars.mpsc.declaration;
            let mut actor_init = quote!{ let #actor = #actor_turbo :: #met_new_ident #turbo_gen (#(#args_idents),*) #unwrapped; };
            let mut not_send_oneshot = quote!{};
            let mut not_send_recv_confirmation = quote!{};

            // !Send
            if aaa.ty_send.is_not_send(){
                // init actor
                actor_init = quote!{};
                play_actor_init = quote! { #actor_turbo :: #met_new_ident #turbo_gen (#(#args_idents),*) };

                // add arguments of 'new' 
                play_fn_args.extend(met.sig.inputs.iter().cloned());

                // add oneshot::Sender type
                play_fn_args.push( TySend::get_oneshot_fn_arg_sender_type( &met.sig, &impl_vars.oneshot ) );

                not_send_oneshot = oneshot.decl(None);
                not_send_recv_confirmation = {
                    let recv_call = oneshot.recv_call(live_name, met_new_ident);
                    quote!{let _ = #recv_call #unwrapped ; }
                }
            }

            let play_arg_idents= (super::args_to_pat_type(&play_fn_args).0).into_iter().map(|mut x| {super::clear_ref_mut(&mut*x);*x});
            let play_args = quote!{ #( #play_arg_idents ),* };

            let spawn = aaa.lib.method_new_spawn(&play_args,script_turbo, &p_ty_generics );
            
            if aaa.mod_receiver.is_slf(){
            
                met.block = syn::parse_quote!{
                    {
                        #actor_init
                        #debut_decl_call
                        #phantom_invoks
                        #channel_decl
                        #not_send_oneshot
                        #spawn
                        #not_send_recv_confirmation
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

        cont.push_script_met(direct,
        & quote!{
            #async_decl fn #direct #d_impl_generics (self, #actor: & #direct_play_mut_token #model_actor_type ) 
            #d_where_clause
            {
                match self {
                    #(#direct_arms),*
                }
            }
        });
    }


    // PLAY
    {   
        let play_actor_init_block = if aaa.ty_send.is_not_send() { TySend::get_play_actor_init_block(&impl_vars, &play_actor_init) } else { quote!{} };
        let Cont{ play_while_block,..} = &cont;
        let ImplVars{ async_decl,..} = &impl_vars;
        let await_call  = impl_vars.get_await_call();
        let ok_or_some = aaa.lib.get_ok_or_some();
        let msg_direct_call = quote!{ #msg.#direct ( & #direct_play_mut_token #actor ) #await_call; };
        let while_block_code = (*play_while_block)(msg_direct_call);

        let play_method = {
    
            quote! {
                #async_decl fn #play #p_impl_generics( #(#play_fn_args),*) 
                #p_where_clause
                {
                    #play_actor_init_block
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

        let MpscChannel{pat_type_sender,..} = mpsc;
        let Cont{ live_clone_attr,..} = &cont;
        let debut_fields = aaa.debut.get_live_fields(&const_vars);

        quote!{
            #live_clone_attr
            #vis struct #live_name #l_impl_generics #l_where_clause {
                #pat_type_sender,
                #debut_fields
                #phantom_fields
            }
        }
    };

   
    ModelSdpl::Actor(
        ActorModelSdpl{
            aaa,
            met_new: impl_vars.met_new.clone(),
            script: cont.get_script_part(&script_def),
            live: cont.get_live_part(&live_def),
        }
    )
}

