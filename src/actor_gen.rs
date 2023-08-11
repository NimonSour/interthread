use crate::attribute::{ActorAttributeArguments,AALib,AAChannel,AAExpand};
use crate::name;
use crate::method;
use crate::error;

use proc_macro_error::abort;
use std::boxed::Box;
use syn::{Ident,Signature,Item,Type };
use quote::{quote,format_ident};
use proc_macro2::{Span,TokenStream};


// returns  (code,edit) TokenStreams 
pub fn actor_macro_generate_code( aaa: ActorAttributeArguments, item: Item, mac: &AAExpand ) -> (TokenStream, TokenStream){

    let (actor_name,actor_type) = name::get_name_and_type(mac,&item,);
    
    let (actor_methods, 
         met_new) =
         method::get_methods( &actor_type,item.clone(),aaa.assoc );

    let met_new = if met_new.is_none() {
        if method::is_trait(&actor_type) {
            let (msg,note) = error::trait_new_sig(&actor_type,false);
            abort!(item,msg;note=note);
        } else {
            let msg = format!("Can not find public/restricted  method `new` or `try_new` for {:?} object.",actor_name.to_string());
            let (note,help) = error::met_new_note_help(&actor_name);
            abort!(item,msg;note=note;help=help);
        }
    } else { met_new.unwrap() };
    
    // Giving a new name if specified 
    let cust_name   = if aaa.name.is_some(){ aaa.name.clone().unwrap() } else { actor_name.clone() }; 
    
    let script_name = name::script(&cust_name);
    let live_name   = name::live(&cust_name);
    
    
    let direct_async = actor_methods.iter().any(|x| x.is_async());
    let play_async   = is_play_async( direct_async, &aaa.lib, &actor_name);



    let (live_field_sender,
        play_input_receiver, 
        new_live_send_recv , 
        live_meth_send_recv, 
        script_field_output, 
        live_send_input,
        live_recv_output ) = channels( &aaa.lib, &aaa.channel, &cust_name);

    let mut direct_arms   = vec![];
    let mut script_fields = vec![];
    
    let mut live_def;
    let mut live_mets    = vec![];
    let mut live_trts    = vec![];

    let mut script_def;
    let mut script_mets  = vec![];
    // let mut script_trts  = vec![];



    pub fn is_play_async( direct_async: bool, lib: &AALib, actor_name: &Ident ) -> bool {

        match lib {
            AALib::Std => {
                if direct_async {
                    let msg = format!("Actor {:?} has 'async' methods but the runtime (lib) is not specified.", actor_name.to_string());
                    abort!( Span::call_site(), msg; help=crate::error::AVAIL_LIB );
                } else {
                    return false;
                }
            },
            _ => {
                return true;
            },
        }
    }

    fn live_static_method( actor_name: &Ident,
                                ident: Ident, 
                                vis: syn::Visibility,
                            mut sig: Signature,
                                args: TokenStream,
                        live_mets: &mut Vec<(Ident,TokenStream)> ) {
        
        method::change_signature_refer(&mut sig);
        let await_call = await_token(sig.asyncness.is_some());
        let stat_met = quote! {
            #vis #sig {
                #actor_name::#ident #args #await_call
            }
        };
        live_mets.push((ident,stat_met));
    }

    fn await_token( b: bool ) -> TokenStream {
        if b { 
            quote!{.await} 
        } else {
            quote!{}
        }
    }

    fn async_token( b: bool ) -> TokenStream {
        if b { 
            quote!{async} 
        } else {
            quote!{}
        }
    }

    let new_vis = met_new.vis.clone();

    let error_send = error::direct_send(&cust_name); 

    for method in actor_methods.clone() {
        
        let (mut sig, script_field_name) = method.get_sig_and_field_name();
        let await_call = await_token(sig.asyncness.is_some());
        method::to_async(&aaa.lib, &mut sig);

        match method {

            method::ActorMethod::Io   { vis, ident, stat,  arguments, output,.. } => {
                let (args_ident,args_type) = method::arguments_ident_type(&arguments);
                
                if stat {
                    live_static_method(&actor_name,ident, vis, sig, args_ident,&mut live_mets)
                }
                else {
                    
                    // Direct Arm
                    let arm_match        = quote! { 
                        #script_field_name { input: #args_ident,  output: send }
                    };
                    let direct_arm       = quote! {
                        #script_name::#arm_match => {send.send( actor.#ident #args_ident #await_call ).expect(#error_send);}
                    };
                    direct_arms.push(direct_arm);
                    
                    // Live Method
                    let live_met      = quote! {

                        #vis #sig {
                            #live_meth_send_recv
                            let msg = #script_name::#arm_match;
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

                    // Direct Arm
                    let arm_match = quote!{ 
                        #script_field_name{  output: send }
                    };
        
                    let direct_arm = quote!{
                        #script_name::#arm_match => {send.send(actor.#ident #args_ident #await_call).expect(#error_send);}
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
    { 
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
            let id_debut_name = if aaa.id {quote!{ ,debut,name}} else {quote!{}};
            match  aaa.channel {
                AAChannel::Inter => {
                    ( quote!{ Self{ queue: queue.clone(), condvar: condvar.clone() #id_debut_name } }, quote!{ queue, condvar, actor  } )
                },
                _  => {
                    ( quote!{ Self{ sender #id_debut_name } }, quote!{ receiver, actor } )
                },
            }
        };

        let spawn = live_new_spawn(play_args);
        let (id_debut,id_name)  =  
        if aaa.id {
            (quote!{let debut =  #script_name ::debut();},
                quote!{let name  = String::from("");})
        } else { (quote!{}, quote!{}) };

        let func_new_body = quote!{

            #vis #new_sig {
                #new_live_send_recv
                let actor = #actor_name:: #func_new_name #args_ident #unwrapped;
                #id_debut
                #id_name
                let #live_var = #init_actor;
                #spawn
                #return_statement
            }
        };
        // live_mets.push((new_sig.ident.clone(),func_new_body));
        live_mets.insert(0,(new_sig.ident.clone(),func_new_body));
    };

    // INTER METHODS AND TRAITS
    if aaa.id {

        live_mets.push((format_ident!("inter_get_debut"),
        quote!{
            #new_vis fn inter_get_debut(&self) -> std::time::SystemTime {
                *self.debut
            }
        }));
        
        live_mets.push((format_ident!("inter_get_count"),
        quote!{
            #new_vis fn inter_get_count(&self) -> usize {
                std::sync::Arc::strong_count(&self.debut)
            }
        }));

        live_mets.push((format_ident!("inter_set_name"),
        quote!{
            #new_vis fn inter_set_name<T: std::string::ToString>(&mut self, name: T) {
                self.name = name.to_string();
            }
        }));


        live_mets.push((format_ident!("inter_get_name"),
        quote!{    
            #new_vis fn inter_get_name(&self) -> &str {
                &self.name
            } 
        }));

        script_mets.push((format_ident!("debut"),
        quote!{
            pub fn debut() -> std::sync::Arc<std::time::SystemTime>{
                static FLAG: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
                loop {
                    let current = FLAG.load(std::sync::atomic::Ordering::SeqCst);
                    let time = std::sync::Arc::new(std::time::SystemTime::now());
                    if let Ok(_) = FLAG.compare_exchange(
                        current,
                        !current,
                        std::sync::atomic::Ordering::SeqCst,
                        std::sync::atomic::Ordering::Relaxed,
                    ){ return time }
                }
            }
        }));
        
        live_trts.push((format_ident!("PartialEq"),
        quote!{
            impl PartialEq for #live_name {
                fn eq(&self, other: &Self) -> bool {
                    *self.debut == *other.debut
                }
            }
        }));

        live_trts.push((format_ident!("Eq"),
        quote!{
            impl Eq for #live_name {}
        }));  

        live_trts.push((format_ident!("PartialOrd"),
        quote!{
            impl PartialOrd for #live_name {
                fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                    other.debut.partial_cmp(&self.debut)
                }
            }
        }));   

        live_trts.push((format_ident!("Ord"),
        quote!{
            impl Ord for #live_name {
                fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                    other.debut.cmp(&self.debut)
                }
            }
        }));  
    } 

    script_def = {

        quote! {
            #[derive(Debug)]
            #new_vis enum #script_name {
                #(#script_fields),*
            }
        }
    };

    // DIRECT
    {

        let decl_async= async_token(direct_async);
        script_mets.push((format_ident!("direct"),
        quote!{
            #new_vis #decl_async fn direct (self, actor: &mut #actor_type ) {
                match self {
                    #(#direct_arms)*
                }
            }
        }));
    }


    // PLAY
    {
        let await_call  = await_token(direct_async);
        let async_decl  = async_token(play_async);
        let end_of_play = error::end_of_life(&actor_name); 

        // needs to be pushed into script_mets
        let play_met = match aaa.channel {

            AAChannel::Unbounded |
            AAChannel::Buffer(_) => match aaa.lib {

                AALib::Std => {
                    quote! {
                        #new_vis fn play ( #play_input_receiver mut actor: #actor_type ) {
                            while let Ok(msg) = receiver.recv(){
                                msg.direct ( &mut actor );
                            }
                            #end_of_play
                        }
                    }
                },

                AALib::Tokio => {
                    quote! {
                        #new_vis #async_decl fn play ( #play_input_receiver mut actor: #actor_type ) {
                            while let Some(msg) = receiver.recv().await{
                                msg.direct ( &mut actor ) #await_call;
                            }
                            #end_of_play
                        }
                    }
                },

                _ => { 
                    quote! {
                        #new_vis #async_decl fn play ( #play_input_receiver mut actor: #actor_type ) {
                            while let Ok(msg) = receiver.recv().await {
            
                                msg.direct ( &mut actor ) #await_call;
                            }
                            #end_of_play
                        }
                    }
                },
            },

            AAChannel::Inter => {
                //impl drop for live while here
                live_trts.push((format_ident!("Drop"),
                quote!{
                    impl Drop for #live_name{
                        fn drop(&mut self){
                            self.condvar.notify_one();
                        }
                    }
                }));

                let error_msg = error::play_guard(&actor_name);

                quote!{
                    #new_vis #async_decl fn play ( #play_input_receiver mut actor: #actor_type ) {

                        let queuing = || -> Option<Vec< #script_name >> {
                            let mut guard = queue.lock().expect(#error_msg);
                            while guard.as_ref().unwrap().is_empty() {
                                if std::sync::Arc::strong_count(&queue) > 1{
                                    guard = condvar.wait(guard).expect(#error_msg);
                                } else { return None }
                            }
                            let income = guard.take();
                            *guard = Some(vec![]);
                            income
                        };
                        while let Some(msgs)  = queuing(){
                            for msg in msgs {
                                msg.direct (&mut actor) #await_call;
                            }
                        }
                        #end_of_play
                    }
                }
            },
        };
        script_mets.push(( format_ident!("play"), play_met ));
    }

    live_def = {
        let (debut_field, name_field) = if aaa.id {
            ( quote!{ pub debut: std::sync::Arc<std::time::SystemTime>,},
            quote!{ pub name: String,} )
        } else { (quote!{}, quote!{})};   

        quote!{
            #[derive(Clone,Debug)]
            #new_vis struct #live_name {
                #live_field_sender
                #debut_field
                #name_field
            }
        }
    };

    // Create and Select Edit Parts

    let mut edit_script_def   = quote::quote!{};
    let edit_script_mets ;
    // let edit_script_trts ;

    let mut edit_live_def  = quote::quote!{};
    let edit_live_mets ;
    let edit_live_trts ;


    match aaa.edit {

        crate::attribute::AAEdit  { live, script } => {
            match script {

                ( def , mets, _trts) => {
                    if def {
                        edit_script_def = script_def.clone();
                        script_def      = quote::quote!{}; 
                    }
                    edit_script_mets = edit_select(mets,&mut script_mets);
                    // edit_script_trts = edit_select(trts,&mut script_trts);
                },
            }

            match live {

                ( def , mets, trts) => {
                    if def {
                        edit_live_def = live_def.clone();
                        live_def      = quote::quote!{}; 
                    }
                    edit_live_mets = edit_select(mets,&mut live_mets);
                    edit_live_trts = edit_select(trts,&mut live_trts);
                },
            }
        }
    }


    // Prepare Token Stream Vecs
    let script_methods = script_mets.iter().map(|x| x.1.clone()).collect::<Vec<_>>();
    // let script_traits    = script_trts.iter().map(|x| x.1).collect::<Vec<_>>();
    let live_methods   = live_mets.iter().map(|x| x.1.clone()).collect::<Vec<_>>();
    let live_traits    = live_trts.iter().map(|x| x.1.clone()).collect::<Vec<_>>();

    let res_code = quote! {

        #item

        #script_def
        impl #script_name {
            #(#script_methods)*
        }
        #live_def
        impl #live_name {
            #(#live_methods)*
        }
        #(#live_traits)*

    };

    let res_edit_script_mets =  
    if  edit_script_mets.is_empty() { quote!{} }
    else { quote!{ impl #script_name { #(#edit_script_mets)* }}};

    let res_edit_live_mets =  
    if  edit_live_mets.is_empty() { quote!{} }
    else { quote!{ impl #live_name { #(#edit_live_mets)* }}};

    let res_edit_live_trts =  
    if  edit_live_trts.is_empty() { quote!{} }
    else { quote!{ #(#edit_live_trts)* }};


    let res_edit = quote!{
        #edit_script_def
        #res_edit_script_mets
        #edit_live_def
        #res_edit_live_mets
        #res_edit_live_trts
    };

    (res_code, res_edit)

}



pub fn edit_select(edit_idents: Option<Vec<syn::Ident>>, 
    ident_mets: &mut Vec<(syn::Ident,proc_macro2::TokenStream)> ) 
                -> Vec<proc_macro2::TokenStream> {

    let mut res = Vec::new();

    if let Some(idents) = edit_idents { 

        if idents.is_empty() {
        res = ident_mets.iter().map(|x| x.1.clone()).collect::<Vec<_>>();
        ident_mets.clear();
        }

        for ident in idents {
            if let Some(pos) = ident_mets.iter().position(|x| x.0 == ident){
            let (_,trt)  = ident_mets.remove(pos);
            res.push(trt);
            } else {
            let msg = format!("No method named `{}` in Actor's methods.",ident.to_string());
            abort!(ident,msg);
            }
        }
    } 
    res
}


pub fn channels( lib: &AALib,
             channel: &AAChannel,
           cust_name: &Ident ) -> ( 
                                    TokenStream,
                                    TokenStream,
                                    TokenStream,
                                    TokenStream,
                                    Box<dyn Fn(Box<Type>) -> TokenStream>,
                                    TokenStream,
                                    TokenStream ){

    let live_field_sender:   TokenStream;
    let play_input_receiver: TokenStream;
    let new_live_send_recv:  TokenStream;
                            
    let type_ident = &name::script(cust_name);
    let (error_live_send,error_live_recv) = error::live_send_recv(cust_name);
    
    let mut live_meth_send_recv = 
        quote!{ let ( send, recv ) = oneshot::channel(); };

    let mut script_field_output: Box<dyn Fn(Box<Type>) -> TokenStream> =
        Box::new(|out_type: Box<Type>|quote!{ output: oneshot::Sender<#out_type>, }); 
    
    let mut live_send_input: TokenStream =
        quote!{let _ = self.sender.send(msg).await;};


    let mut live_recv_output: TokenStream = 
        quote!{ recv.await.expect(#error_live_recv)};

    match  channel {

        AAChannel::Unbounded    => {

            match  lib { 

                AALib::Std      => {
                    live_field_sender   = quote!{ sender: std::sync::mpsc::Sender<#type_ident>, };   
                    play_input_receiver = quote!{ receiver: std::sync::mpsc::Receiver<#type_ident>, }; 
                    new_live_send_recv  = quote!{ let ( sender, receiver ) = std::sync::mpsc::channel(); };
                    live_send_input     = quote!{ let _ = self.sender.send(msg).expect(#error_live_send);};
                    live_recv_output    = quote!{ recv.recv().expect(#error_live_recv)};
                },

                AALib::Tokio    => {
                    live_field_sender   = quote!{ sender: tokio::sync::mpsc::UnboundedSender<#type_ident>, };
                    play_input_receiver = quote!{ mut receiver: tokio::sync::mpsc::UnboundedReceiver<#type_ident>, }; 
                    new_live_send_recv  = quote!{ let ( sender, receiver ) = tokio::sync::mpsc::unbounded_channel(); }; 
                    live_meth_send_recv = quote!{ let ( send, recv ) = tokio::sync::oneshot::channel(); };
                    script_field_output = Box::new(|out_type: Box<Type>|quote!{ output: tokio::sync::oneshot::Sender<#out_type>, });                
                    live_send_input     = quote!{ let _ = self.sender.send(msg).expect(#error_live_send);};
                },

                AALib::AsyncStd  => {
                    live_field_sender   = quote!{ sender: async_std::channel::Sender<#type_ident>, };
                    play_input_receiver = quote!{ receiver: async_std::channel::Receiver<#type_ident>, };
                    new_live_send_recv  = quote!{ let ( sender, receiver ) = async_std::channel::unbounded(); };                    
                },

                AALib::Smol      => {
                    live_field_sender   = quote!{ sender: async_channel::Sender<#type_ident>, };
                    play_input_receiver = quote!{ receiver: async_channel::Receiver<#type_ident>, };
                    new_live_send_recv  = quote!{ let ( sender, receiver ) =  async_channel::unbounded(); }; 
                },
            }
        },
        AAChannel::Buffer(val)    => {

            match  lib { 

                AALib::Std      => {
                    live_field_sender   = quote!{ sender: std::sync::mpsc::SyncSender<#type_ident>, };
                    play_input_receiver = quote!{ receiver: std::sync::mpsc::Receiver<#type_ident>, };
                    new_live_send_recv  = quote!{ let ( sender, receiver ) = std::sync::mpsc::sync_channel(#val); };
                    live_send_input     = quote!{ let _ = self.sender.send(msg).expect(#error_live_send);};
                    live_recv_output    = quote!{ recv.recv().expect(#error_live_recv)};
                },
                AALib::Tokio    => {
                    live_field_sender   = quote!{ sender: tokio::sync::mpsc::Sender<#type_ident>, };
                    play_input_receiver = quote!{ mut receiver: tokio::sync::mpsc::Receiver<#type_ident>, };
                    new_live_send_recv  = quote!{ let ( sender, receiver ) = tokio::sync::mpsc::channel(#val); }; 
                    live_meth_send_recv = quote!{ let ( send, recv ) = tokio::sync::oneshot::channel(); };
                    script_field_output = Box::new(|out_type: Box<Type>|quote!{ output: tokio::sync::oneshot::Sender<#out_type>, });                
                },

                AALib::AsyncStd  => {
                    live_field_sender   = quote!{ sender: async_std::channel::Sender<#type_ident>, };
                    play_input_receiver = quote!{ receiver: async_std::channel::Receiver<#type_ident>, };
                    new_live_send_recv  = quote!{ let ( sender, receiver ) = async_std::channel::bounded(#val); };
                },

                AALib::Smol      => {
                    live_field_sender   = quote!{ sender: async_channel::Sender<#type_ident>, };
                    play_input_receiver = quote!{ receiver: async_channel::Receiver<#type_ident>, };
                    new_live_send_recv  = quote!{ let ( sender, receiver ) = async_channel::bounded(#val); };
                },
            }
        },
        AAChannel::Inter  => {

            live_field_sender   = quote!{ 
                queue: std::sync::Arc<std::sync::Mutex<Option<Vec<#type_ident>>>>,
                condvar:                       std::sync::Arc<std::sync::Condvar>,
            };
            play_input_receiver = quote!{ 
                queue: std::sync::Arc<std::sync::Mutex<Option<Vec<#type_ident>>>>,
                condvar:                       std::sync::Arc<std::sync::Condvar>,
            };
            new_live_send_recv  = quote!{ 
                let queue       = std::sync::Arc::new(std::sync::Mutex::new(Some(vec![])));
                let condvar     = std::sync::Arc::new(std::sync::Condvar::new());
            };

            let error_msg = error::live_guard(cust_name);
            live_send_input     =  quote!{
                {
                    let mut guard = self.queue.lock().expect(#error_msg);
        
                    guard.as_mut()
                    .map(|s| s.push(msg));
                }
                self.condvar.notify_one();
            };

            live_recv_output     =  quote!{recv.recv().expect(#error_live_recv)};
        },
    }

    
    (
        live_field_sender,
        play_input_receiver, 
        new_live_send_recv , 
        live_meth_send_recv, 
        script_field_output, 
        live_send_input,
        live_recv_output,
    )
}





