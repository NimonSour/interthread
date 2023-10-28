
// use crate::error;
// use crate::model::{self,get_ident_type_generics,MpscChannel,Cont,Vars,method,generic,attribute::ActorAttributeArguments,argument::{Lib,Model}};

// use proc_macro_error::abort;
// use syn::{Ident,Type,Signature,ItemImpl,Visibility };
// use quote::{quote,format_ident};
// use proc_macro2::TokenStream;


// pub fn actor_model( aaa: ActorAttributeArguments, item_impl: &ItemImpl, mac: Model, mut new_vis: Option<Visibility> ) 
//     ->  crate::model::ActorModelSdpl{ 
 

//     //-------------(1)
//     let mut cont = model::Cont::new();

//     let (actor_name,
//         actor_type,
//         generics) = get_ident_type_generics(item_impl);
    


//     let ( mut actor_methods, 
//           mut met_new) =
//          method::get_methods( &actor_type,item_impl.clone(),aaa.assoc ,Model::Actor.eq(&mac));


//     let mut model_generics = generics.clone();
//     // let actor_ty_generics  = generics.split_for_impl().1;

//     let ( impl_generics,
//             ty_generics,
//            where_clause ) = {

//         let mut sigs = actor_methods.iter_mut().map(|m| m.get_mut_sig()).collect::<Vec<_>>();

//         if met_new.is_some() {
//             let mut mn = met_new.unwrap();
//             sigs.push(mn.get_mut_sig());
//             generic::get_parts( &mut model_generics, sigs);

//             met_new = Some(mn);

//         } else { generic::get_parts( &mut model_generics, sigs); }
        
//         model_generics.split_for_impl()

//     };
    

//     let vars = &Vars::new(&aaa,&actor_name,Model::Actor,&mac) ;
//     let Vars { script_name,live_name,.. } = vars;
//     let script_type: Type = syn::parse_quote!{ #script_name #ty_generics };
//     let live_type: Type   = syn::parse_quote!{ #live_name #ty_generics };
//     let (oneshot,mpsc) = &model::get_channels_one_mpsc(&aaa,vars,&script_type);
    

//     let async_decl   = 

//         match &aaa.lib {
//             Lib::Std => {
//                 if let Some(pos) = actor_methods.iter().position(|x| x.is_async()){
//                     error::abort_async_no_lib(&actor_name,&actor_methods[pos]);
//                 }
//                 None
//             },
//             _ => { Some(quote!{async}) },
//         };



//     let Vars{actor,play,direct,
//              debut, msg,debut_play,
//              sender,receiver,name,..} = vars;
    

//     method::to_raw_parts( vars,&mut cont,&aaa,actor_methods,oneshot,mpsc );


//     // This is file_path for legend 
//     let ( script_legend_file, live_legend_file ) = 
//     if aaa.debut.is_legend(){
//         let (s,l) = crate::show::check_legend_path(&mac, &vars.cust_name, &aaa.debut.path.as_ref().unwrap());
//         (Some(s),Some(l))
//     } else {
//         (None, None)
//     };


//     //-------------(2)
//     /*
    

//     // for method in actor_methods.clone() {
        
//     //     let (mut sig, script_field_name) = method.get_sig_and_field_name();

//     //     let await_call = sig.asyncness.as_ref().map(|_|quote!{.await});
//     //     method::to_async(&aaa.lib, &mut sig);

//     //     let error_send = error::direct_send(&script_name,&script_field_name);

//     //     // Debug arm
//     //     let add_arm = | debug_arms: &mut Vec<TokenStream>,ident: &Ident | {

//     //         let str_field_name = format!("{}::{}",script_name.to_string() ,ident.to_string());

//     //         let debug_arm = quote! {
//     //             #script_name :: #script_field_name {..} => write!(f, #str_field_name),
//     //         };
//     //         debug_arms.push(debug_arm);
//     //     };

//     //     match method {

//     //         method::ActorMethod::Io   { vis, ident, stat,  arguments, output,.. } => {
//     //             let (args_ident,args_type) = method::arguments_ident_type(&arguments);
                
//     //             if stat {
//     //                 live_static_method(&actor_name,ident, vis, sig, args_ident,&mut live_mets)
//     //             }
//     //             else {
//     //                 // Debug Arm push
//     //                 add_arm(&mut debug_arms, &script_field_name);

//     //                 // Direct Arm
//     //                 let arm_match        = quote! { 
//     //                     #script_field_name { input: #args_ident,  output: send }
//     //                 };
//     //                 let direct_arm       = quote! {
//     //                     #script_name :: #arm_match => {send.send( actor.#ident #args_ident #await_call ) #error_send ;}
//     //                 };
//     //                 direct_arms.push(direct_arm);
                    
//     //                 // Live Method
//     //                 let live_met      = quote! {

//     //                     #vis #sig {
//     //                         #live_meth_send_recv
//     //                         let msg = #script_name :: #arm_match;
//     //                         #live_send_input
//     //                         #live_recv_output
//     //                     }
//     //                 };

//     //                 live_mets.push((ident,live_met));

//     //                 // Script Field Struct
//     //                 let output_type      = (&*script_field_output)(output);

//     //                 let script_field = quote! {
//     //                     #script_field_name {
//     //                         input: #args_type,
//     //                         #output_type
//     //                     }
//     //                 };

//     //                 script_fields.push(script_field);
//     //             }
//     //         },
//     //         method::ActorMethod::I    { vis, ident, arguments ,..} => {
                
//     //             let (args_ident,args_type) = method::arguments_ident_type(&arguments);
                
//     //             // Debug Arm push
//     //             add_arm(&mut debug_arms, &script_field_name);

//     //             // Direct Arm
//     //             let arm_match = quote!{ 
//     //                 #script_field_name{ input: #args_ident }
//     //             };
    
//     //             let direct_arm = quote!{
//     //                 #script_name::#arm_match => {actor.#ident #args_ident #await_call;},
//     //             };
//     //             direct_arms.push(direct_arm);

//     //             // Live Method
//     //             let live_met = quote!{
    
//     //                 #vis #sig {
//     //                     let msg = #script_name::#arm_match ;
//     //                     #live_send_input
//     //                 }
//     //             };
//     //             live_mets.push((ident,live_met));
            


//     //             // Script Field Struct
//     //             let script_field = quote!{
//     //                 #script_field_name {
//     //                     input: #args_type,
//     //                 }
//     //             };
//     //             script_fields.push(script_field);

//     //         },
//     //         method::ActorMethod::O    { vis, ident, stat, output ,..} => {
//     //             let (args_ident,_) = method::arguments_ident_type(&vec![]);

//     //             if stat {
//     //                 live_static_method(&actor_name,ident, vis, sig, args_ident,&mut live_mets)
//     //             }
//     //             else {
                    
//     //                 // Debug Arm push
//     //                 add_arm(&mut debug_arms, &script_field_name);

//     //                 // Direct Arm
//     //                 let arm_match = quote!{ 
//     //                     #script_field_name{  output: send }
//     //                 };
        
//     //                 let direct_arm = quote!{
//     //                     #script_name::#arm_match => {send.send(actor.#ident #args_ident #await_call) #error_send ;}
//     //                 };
//     //                 direct_arms.push(direct_arm);



//     //                 // Live Method
//     //                 let live_met = quote!{
                    
//     //                     #vis #sig {
//     //                         #live_meth_send_recv
//     //                         let msg = #script_name::#arm_match ;
//     //                         #live_send_input
//     //                         #live_recv_output
//     //                     }
//     //                 };
//     //                 live_mets.push((ident, live_met));
                
//     //                 // Script Field Struct
//     //                 let output_type  = (&*script_field_output)(output);

//     //                 let script_field = quote!{
//     //                     #script_field_name {
//     //                         #output_type
//     //                     }
//     //                 };
//     //                 script_fields.push(script_field);
//     //             }
//     //         },
//     //         method::ActorMethod::None { vis, ident ,..} => {

//     //             // Debug Arm push
//     //             add_arm(&mut debug_arms, &script_field_name);

//     //             // Direct Arm
//     //             let arm_match = quote!{ 
//     //                 #script_field_name {} 
//     //             };
    
//     //             let direct_arm = quote!{
//     //                 #script_name::#arm_match => {actor.#ident () #await_call;},
//     //             };
//     //             direct_arms.push(direct_arm);

//     //             // Live Method
//     //             let live_met = quote!{
                
//     //                 #vis #sig {
//     //                     let msg = #script_name::#arm_match ;
//     //                     #live_send_input
//     //                 }
//     //             };
//     //             live_mets.push((ident,live_met));
            
//     //             // Script Field Struct
//     //             let script_field = quote!{
                    
//     //                 #script_field_name {}
//     //             };
//     //             script_fields.push(script_field);
//     //         },
//     //     }
//     // } 

//     */

//     // METHOD NEW OLD
//     /*
    
//     if Model::Actor.eq(&mac) { 

//         if met_new.is_none() {

//             let msg = format!("Can not find public/restricted  method `new` or `try_new` for {:?} object.",actor_name.to_string());
//             let (note,help) = error::met_new_note_help(&actor_name);
//             abort!(item_impl,msg;note=note;help=help);
//         }
        
//         // Change visibility of model methods 
//         new_vis = met_new.as_ref().map(|m| m.vis.clone());

//         let met_new         = met_new.unwrap();
//         let new_sig             = &met_new.new_sig;
//         let func_new_name           = &new_sig.ident;
//         let (args_ident, _ )   = method::arguments_ident_type(&met_new.get_arguments());
//         let live_var                 = format_ident!("inter_{actor}_live");
//         let unwrapped          = met_new.unwrap_sign();
//         let return_statement   = met_new.live_ret_statement(&live_var);
//         let vis                = &met_new.vis.clone();

//         let (init_actor, play_args) = {
//             let id_debut_name = if aaa.debut.active() {quote!{ ,inter_debut,inter_name}} else {quote!{}};
//             ( quote!{ Self{ sender #id_debut_name } }, quote!{ receiver, #actor } )
//         };

//         let spawn = aaa.lib.method_new_spawn(&play_args,script_name);
//         let turbofish = ty_generics.as_turbofish();
//         let (id_debut,id_name)  =  
//         if aaa.debut.active() {
//             (quote!{let inter_debut =  #script_name #turbofish ::debut();},
//                 quote!{let inter_name  = String::from("");})
//         } else { (quote!{}, quote!{}) };
        
//         let func_new_body = quote!{

//             #vis #new_sig {
//                 let #actor = #actor_name:: #func_new_name #args_ident #unwrapped;
//                 #new_live_send_recv
//                 #id_debut
//                 #id_name
//                 let #live_var = #init_actor;
//                 #spawn
//                 #return_statement
//             }
//         };



//         live_mets.insert(0,(new_sig.ident.clone(),func_new_body));
//     };
//      */

//     if Model::Actor.eq(&mac) { 

//         if met_new.is_none() {

//             let msg = format!("Can not find public/restricted  method `new` or `try_new` for {:?} object.",actor_name.to_string());
//             let (note,help) = error::met_new_note_help(&actor_name);
//             abort!(item_impl,msg;note=note;help=help);
//         }
        
//         // Change visibility of model methods 
//         new_vis = met_new.as_ref().map(|m| m.vis.clone());

//         let met_new         = met_new.unwrap();
//         let new_sig             = &met_new.new_sig;
//         let func_new_name           = &new_sig.ident;
//         let (args_ident, _ )   = method::arguments_pat_type(&met_new.get_arguments());
//         let unwrapped          = met_new.unwrap_sign();
//         let vis                = &met_new.vis.clone();

//         let (init_live, play_args) = {
//             if aaa.debut.active() {
//                 (quote!{ Self { #sender,#debut: std::sync::Arc::clone(&#debut), #name : format!("{:?}",* #debut) }} ,
//                  quote!{ #receiver, #actor, #debut_play})
//             } else {

//                 (quote!{ Self{ #sender } }, 
//                  quote!{ #receiver, #actor } )
//             }
//         };

//         let spawn = aaa.lib.method_new_spawn(&play_args,script_name);
//         let turbofish = ty_generics.as_turbofish();

//         let vars_debut = 
//         if aaa.debut.active() {
//             quote!{let #debut =  #script_name #turbofish ::#debut();
//                    let #debut_play = *std::sync::Arc::clone(&#debut); }
//         } else {quote!{}};

//         let return_statement   = met_new.live_ret_statement(&init_live);
        
//         let MpscChannel{declaration, ..} = mpsc;
//         let Cont{live_mets,..} = &mut cont;

//         let func_new_body = quote!{

//             #vis #new_sig {
//                 let #actor = #actor_name:: #func_new_name #args_ident #unwrapped;
//                 #declaration
//                 #vars_debut
//                 #spawn
//                 #return_statement
//             }
//         };

//         live_mets.insert(0,(new_sig.ident.clone(),func_new_body));
//     };

     
    

//     // LIVE INTER METHODS AND TRAITS
//     if aaa.debut.active(){
//         aaa.debut.impl_debut( &mut cont, vars, &new_vis, &ty_generics, &where_clause)
//     }
    
//     // SCRIPT DEFINITION
//     let script_def = {
//         let Cont{ script_fields,..} = &mut cont;
//         quote! {
//             #new_vis enum #script_name #ty_generics #where_clause {
//                 #(#script_fields),*
//             }
//         }
//     };        


//     // DIRECT
//     {
//         let Cont{script_mets,direct_arms,..} = &mut cont;
//         script_mets.push((direct.clone(),
//         quote!{
//             #new_vis #async_decl fn #direct (self, #actor: &mut #actor_type /*#actor_ty_generics*/ ) {
//                 match self {
//                     #(#direct_arms)*
//                 }
//             }
//         }));
//     }


//     // PLAY
//     if Model::Actor.eq(&mac) {

//         let await_call  = async_decl.as_ref().map(|_| quote!{.await});
//         // let recv_await    =  play_async_decl.as_ref().map(|_| quote!{.await});
//         let end_of_play = error::end_of_life( &actor_name, &aaa.debut.clone() ); // <- include 
      
//         let debut_pat_type = if aaa.debut.active(){quote!{,#debut: std::time::SystemTime }} else { quote!{} };

//         let MpscChannel{pat_type_receiver,..}      = mpsc;
//         let Cont{script_mets,..} = &mut cont;
//         let play_method = {
        
//             let ok_or_some = match aaa.lib {
//                 Lib::Tokio => quote!{Some},
//                 _ => quote!{Ok}
//             };
//             quote! {
//                 #new_vis #async_decl fn #play ( #pat_type_receiver mut #actor: #actor_type /*#actor_ty_generics*/ #debut_pat_type ) {
//                     while let #ok_or_some (#msg) = #receiver.recv() #await_call {
//                         #msg.#direct ( &mut #actor ) #await_call;
//                     }
//                     #end_of_play
//                 }
//             }
//         };
//         script_mets.push(( play.clone(), play_method ));

//     }

//     // SCRIPT TRAIT (Debug)
//     {   
//         let Cont{ script_trts,debug_arms,..} = &mut cont;
//         let str_script_name = script_name.to_string();
//         let body = 
//         if debug_arms.is_empty() { 
//             quote!{ write!(f, #str_script_name )} 
//         } else {
//             quote!{ match self { #(#debug_arms)* } }
//         };
//         script_trts.push((format_ident!("Debug"),
//         quote! {
//             impl #ty_generics std::fmt::Debug for #script_name #ty_generics #where_clause {
        
//                 fn fmt( &self, f: &mut std::fmt::Formatter<'_> ) -> std::fmt::Result {
//                     #body
//                 }
//             }
//         }));
//     }


//     // LIVE DEFINITION
//     let live_def = {
//         let MpscChannel{pat_type_sender,..} = &mpsc;
//         if Model::Actor.eq(&mac) {
//             let (debut_field, name_field) = if aaa.debut.active() {
//                 ( quote!{ pub #debut: std::sync::Arc<std::time::SystemTime>,},
//                 quote!{ pub #name: String,} )
//             } else { (quote!{}, quote!{})};   
            
//             quote!{
//                 #[derive(Clone)]
//                 #new_vis struct #live_name #ty_generics #where_clause {
//                     #pat_type_sender
//                     #debut_field
//                     #name_field
//                 }
//             }
//         } else { 

//             quote!{
//                 #[derive(Clone)]
//                 #new_vis struct #live_name #ty_generics #where_clause {
//                     #pat_type_sender
//                 }
//             }

//         }

//     };

//     //-------------(3)

//     let Vars { cust_name,..} = vars;
//     let Cont { script_mets, script_trts,
//                live_mets, live_trts,..} = cont;

//     crate::model::ActorModelSdpl {
//         name:      cust_name.clone(),
//         asyncness:        async_decl,
//         mac:             mac.clone(),
//         edit:               aaa.edit,
//         generics:     model_generics,
//         vars:           vars.clone(),
//         // script_name: script_name.clone(),
//         // live_name: live_name.clone(),

//         script: ( script_def, script_mets, script_trts ),
//         live:   (   live_def,   live_mets,   live_trts ),
//     }
// }

// pub fn macro_actor_generate_code( aaa: ActorAttributeArguments, item_impl: ItemImpl ) 
//     -> ( TokenStream, TokenStream ) {


//     let mut act_model = actor_model( aaa,&item_impl,Model::Actor,None);
    
//     let (mut code,edit) = act_model.split_edit();
    
//     // abort!(item_impl,code.to_string());
  

//     code = quote!{

//         #item_impl
//         #code
//     };
//     (code,edit)
    
// }
//     // let msg = live_mets.last().unwrap().1.to_string();
//     // let msg = live_def.to_string();
//     // abort!(proc_macro::Span::call_site(),msg );










