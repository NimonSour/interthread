use crate::attribute::{ActorAttributeArguments,AALib,AAChannel};
use crate::name;
use crate::method;
use crate::error;

use proc_macro_error::{abort};
use syn::{Ident,ItemImpl,Signature,Type,Token};
use quote::{quote,format_ident};
use proc_macro2::{Span,TokenStream};




pub fn impl_get_name(impl_block: &ItemImpl) -> Ident{
    match &*impl_block.self_ty {
        Type::Path(tp) => tp.path.segments.first().unwrap().ident.clone(),
        _ => abort!(impl_block,"Internal Error.'actor_gen::impl_get_name'. Could not get item Impl's name!"),
    }
}
pub struct ActorMacroGeneration{
        
    name:                             Ident,
    cust_name:                        Ident,
    impl_block:                    ItemImpl,
    aaa:                 ActorAttributeArguments,
    met_new:              method::ActorMethodNew, 
    actor_methods:      Vec<method::ActorMethod>,
    direct_arms:   Vec<TokenStream>,
    live_methods:  Vec<TokenStream>,
    script_fields: Vec<TokenStream>,
    direct_async:                           bool,
    play_async:                             bool,
    channels:                           Channels,

}

impl ActorMacroGeneration {

    pub fn new( aaa: ActorAttributeArguments, impl_block: ItemImpl ) -> Self {

        let name = impl_get_name(&impl_block);

        let (actor_methods, met_new) =
        method::get_methods( &name,impl_block.clone(),aaa.assoc );
        
        if met_new.is_none() {

            let msg = format!("Can not find public  method `new` or `try_new` for {:?} object.",name.to_string());
            let (note,help) = error::met_new_note_help(&name);

        abort!(impl_block,msg;note=note;help=help);
    }
        
        // Giving a new name if specified 
        let cust_name   = if aaa.name.is_some(){ aaa.name.clone().unwrap() } else { name.clone() }; 
        let direct_async = actor_methods.iter().any(|x| x.is_async());
        let play_async   = Self::is_play_async( direct_async, &aaa.lib, &name);
        let channels = Channels::new( &aaa.lib, &aaa.channel, &cust_name);
       
        Self {
            name,
            cust_name,
            impl_block,
            aaa,
            met_new:   met_new.unwrap(),
            actor_methods,
            direct_arms:         vec![],
            live_methods:        vec![],
            script_fields:       vec![],
            direct_async,
            play_async,
            channels,
        }
    }

    pub fn is_play_async( direct_async: bool, lib: &AALib, name: &Ident ) -> bool {

        match lib {
            AALib::Std => {
                if direct_async {
                    let msg = format!("Actor {:?} has 'async' methods but the runtime(lib) is not specified.", name.to_string());
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

    fn live_static_method(&mut self,  name: Ident, mut sig: Signature, args: TokenStream ) {
        
        method::change_signature_refer(&mut sig);
        let await_call = Self::await_token(sig.asyncness.is_some());
        let actor_name = &self.name;
        let gen_method = quote! {

            pub #sig {
                #actor_name::#name #args #await_call
            }
        };
        self.live_methods.push(gen_method );
    }

    fn await_token( b: bool ) -> TokenStream{
        if b { 
            quote!{.await} 
        } else {
            quote!{}
        }
    }
    
    fn async_token( b: bool ) -> TokenStream{
        if b { 
            quote!{async} 
        } else {
            quote!{}
        }
    }

    pub fn send_in_recv_out(&self, sig: &mut Signature ) -> (&TokenStream, &TokenStream)  {
        
        match self.aaa.lib {

            AALib::Std => (),
            _ => {
                sig.asyncness = Some(Token![async](Span::call_site()));
            }
        }

        (
            &self.channels.live_send_input ,
            &self.channels.live_recv_output
        )
    }

    pub fn gen_tokio_actor_model_bits(&mut self){
        
        let script_name = name::script(&self.cust_name);
        let error_send = error::direct_send(&self.cust_name); 
        for method in self.actor_methods.clone() {
            
            let (mut sig, script_field_name) = method.get_sig_and_field_name();
            let await_call = Self::await_token(sig.asyncness.is_some());

            match method {

                method::ActorMethod::Io   { stat, ident, arguments, output,.. } => {
                    let (args_ident,args_type) = method::arguments_ident_type(&arguments);
                    
                    if stat {
                        self.live_static_method(ident, sig, args_ident)
                    }
                    else {
                        
                        // Direct Arm
                        let arm_match        = quote! { 
                            #script_field_name { input: #args_ident,  output: send }
                        };
                        let direct_arm       = quote! {
                            #script_name::#arm_match => {send.send( actor.#ident #args_ident #await_call ).expect(#error_send);}
                        };
                        self.direct_arms.push(direct_arm);
                        


                        // Live Method
                        let instant_channel = &self.channels.live_meth_send_recv;
                        let (send_input,recv_output) = self.send_in_recv_out(&mut sig);
                       
                        let live_method      = quote! {

                            pub #sig {
                                #instant_channel
                                let msg = #script_name::#arm_match;
                                #send_input
                                #recv_output
                            }
                        };

                        self.live_methods.push(live_method);



                        // Script Field Struct
                        let output_type      = (&*self.channels.script_field_output)(output);

                        let script_field = quote! {
                            #script_field_name {
                                input: #args_type,
                                #output_type
                            }
                        };

                        self.script_fields.push(script_field);
                    }
                },

                method::ActorMethod::I    { ident, arguments ,..} => {
                    
                    let (args_ident,args_type) = method::arguments_ident_type(&arguments);
                    


                    // Direct Arm
                    let arm_match = quote!{ 
                        #script_field_name{ input: #args_ident }
                    };
        
                    let direct_arm = quote!{
                        #script_name::#arm_match => {actor.#ident #args_ident #await_call;},
                    };
                    self.direct_arms.push(direct_arm);




                    // Live Method
                    let (send_input,_) = self.send_in_recv_out(&mut sig);

                    let live_method = quote!{
        
                        pub #sig {
                            let msg = #script_name::#arm_match ;
                            #send_input
                        }
                    };
                    self.live_methods.push( live_method );
                


                    // Script Field Struct
                    let script_field = quote!{
                        #script_field_name {
                            input: #args_type,
                        }
                    };
                    self.script_fields.push(script_field);

                },
                method::ActorMethod::O    {stat, ident, output ,..} => {
                    let (args_ident,_) = method::arguments_ident_type(&vec![]);

                    if stat {
                        self.live_static_method(ident, sig, args_ident)
                    }
                    else {



                        // Direct Arm
                        let arm_match = quote!{ 
                            #script_field_name{  output: send }
                        };
            
                        let direct_arm = quote!{
                            #script_name::#arm_match => {send.send(actor.#ident #args_ident #await_call).expect(#error_send);}
                        };
                        self.direct_arms.push(direct_arm);



                        // Live Method
                        let instant_channel = &self.channels.live_meth_send_recv;
                        let (send_input,recv_output) = self.send_in_recv_out(&mut sig);

                        let live_method = quote!{
                        
                            pub #sig {
                                #instant_channel
                                let msg = #script_name::#arm_match ;
                                #send_input
                                #recv_output
                            }
                        };
                        self.live_methods.push( live_method );
                    


                        // Script Field Struct
                        let output_type      = (&*self.channels.script_field_output)(output);

                        let script_field = quote!{
                            #script_field_name {
                                #output_type
                            }
                        };
                        self.script_fields.push(script_field);
                    }
                },
                method::ActorMethod::None { ident ,..} => {



                    // Direct Arm
                    let arm_match = quote!{ 
                        #script_field_name {} 
                    };
        
                    let direct_arm = quote!{
                        #script_name::#arm_match => {actor.#ident () #await_call;},
                    };
                    self.direct_arms.push(direct_arm);



                    // Live Method
                    let (send_input,_) = self.send_in_recv_out(&mut sig);

                    let live_method = quote!{
                    
                        pub #sig {
                            let msg = #script_name::#arm_match ;
                            #send_input
                        }
                    };
                    self.live_methods.push( live_method );
                


                    // Script Field Struct
                    let script_field = quote!{
                        
                        #script_field_name {}
                    };
                    self.script_fields.push(script_field);
                },
            }
        } 
    }

    fn live_new_spawn(&self, play: TokenStream ) -> TokenStream {
        match self.aaa.lib {
            AALib::Std      => {
                quote!{ std::thread::spawn(|| { #play });}
            },
            AALib::Smol     => {
                quote!{ smol::spawn( #play ).detach();} 
            },
            AALib::Tokio    => {
                quote!{ tokio::spawn(#play);}
            },
            AALib::AsyncStd => {
                quote!{ async_std::task::spawn(#play);}
            },
        }
    }

    fn gen_impl_debut(&self) -> TokenStream {
        let script_name = name::script(&self.cust_name);
        if self.aaa.id {
            quote!{
                impl #script_name {

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
                }
            }
        }
        else {
            quote!{}
        }
    }

    fn gen_impl_eq_ord(&self) -> TokenStream {
        let live_name = name::live(&self.cust_name);
        if self.aaa.id {
            quote!{
                impl PartialEq for #live_name {
                    fn eq(&self, other: &Self) -> bool {
                        *self.debut == *other.debut
                    }
                }
                
                impl Eq for #live_name {}
                
                impl PartialOrd for #live_name {
                    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                        other.debut.partial_cmp(&self.debut)
                    }
                }
                
                impl Ord for #live_name {
                    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                        other.debut.cmp(&self.debut)
                    }
                }
            }
        }
        else {
            quote!{}
        }
    }

    fn gen_script(&mut self) -> TokenStream {

        let script_name        = name::script(&self.cust_name);
        let fields = &self.script_fields;

        quote! {
            #[derive(Debug)]
            pub enum #script_name {
                #(#fields),*
            }
        }
    }


    // DIRECT
    fn gen_impl_direct(&self) -> TokenStream {

        let name            = &self.name;
        let arms = &self.direct_arms;

        let script_name     = name::script(&self.cust_name);
        let direct_name     = name::direct(&self.cust_name);
        let decl_async= Self::async_token(self.direct_async);
        quote!{
            impl #script_name {
                pub #decl_async fn #direct_name (self, actor: &mut #name ) {
                    match self {
                        #(#arms)*
                    }
                }
            }
        }
    }

    // PLAY
    fn gen_func_play(&mut self) -> TokenStream {

        let name              = &self.name;
        let play_name          = name::play(&self.cust_name);
        let script_name        = name::script(&self.cust_name);
        let direct_name        = name::direct(&self.cust_name);


        let recv_channel  = &self.channels.play_input_receiver; 
        let await_call     = Self::await_token(self.direct_async);
        let async_decl     = Self::async_token(self.play_async);
        
        let end_of_play = error::end_of_life(&self.name); 

        match self.aaa.channel {

            AAChannel::Unbounded |
            AAChannel::Buffer(_) => match self.aaa.lib {

                AALib::Std => {
                    quote! {
        
                        pub fn #play_name ( #recv_channel mut actor: #name ) {
                            while let Ok(msg) = receiver.recv(){
                                msg.#direct_name ( &mut actor );
                            }
                            #end_of_play
                        }
                    }
                },

                AALib::Tokio => {
                    quote! {
        
                        pub #async_decl fn #play_name ( #recv_channel mut actor: #name ) {
                            while let Some(msg) = receiver.recv().await{
                                msg.#direct_name ( &mut actor ) #await_call;
                            }
                            #end_of_play
                        }
                    }
                },

                _ => { 
                    quote! {
    
                        pub #async_decl fn #play_name ( #recv_channel mut actor: #name ) {
                            while let Ok(msg) = receiver.recv().await {
            
                                msg.#direct_name ( &mut actor ) #await_call;
                            }
                            #end_of_play
                        }
                    }
                },
            },

            AAChannel::Inter => {
                let error_msg = error::play_guard(&self.name);
                quote!{
                    pub #async_decl fn #play_name ( #recv_channel mut actor: #name ) {

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
                                msg.#direct_name (&mut actor) #await_call;
                            }
                        }

                        #end_of_play
                    }
                }
            },
        }
    }

    fn gen_struct_live_method_new(&mut self) -> TokenStream { 

        let name                    = &self.name;
        let play_name                = name::play(&self.cust_name);
        let script_name              = name::script(&self.cust_name);
        let new_sig             = &self.met_new.new_sig;
        let (args_ident, _ )   = method::arguments_ident_type(&self.met_new.get_arguments());
        let live_var                 = format_ident!("actor_live");
        let send_recv_channel = &self.channels.new_live_send_recv;
        let func_new_name           = &new_sig.ident;
        let unwrapped          = self.met_new.unwrap_sign();
        let return_statement   = self.met_new.live_ret_statement(&live_var);

        let (init_actor, play_args) = {
            let id_debut = if self.aaa.id {quote!{ ,debut,name}} else {quote!{}};
            match  self.aaa.channel {
                AAChannel::Inter => {
                    ( quote!{ Self{ queue: queue.clone(), condvar: condvar.clone() #id_debut } }, quote!{ queue, condvar, actor  } )
                },
                _  => {
                    ( quote!{ Self{ sender #id_debut } }, quote!{ receiver, actor } )
                },
            }
        };

        let play_call =  quote!{ #play_name(#play_args) }; 
        let spawn     =  self.live_new_spawn(play_call);
        let id_debut  =  if self.aaa.id {quote!{let debut =  #script_name ::debut();}} else { quote!{}};
        let id_name   =  if self.aaa.id {quote!{let name = String::from("");}} else { quote!{}};

        quote!{

            pub #new_sig {
                #send_recv_channel
                let actor = #name:: #func_new_name #args_ident #unwrapped;
                #id_debut
                #id_name
                let #live_var = #init_actor;
                #spawn
                #return_statement
            }
        }
    }
    
    fn gen_live_impl_drop(&self) ->  TokenStream{
        let live_name  = name::live(&self.cust_name);

        match self.aaa.channel{

            AAChannel::Inter =>{ 
                quote!{
                    impl Drop for #live_name{
                        fn drop(&mut self){
                            self.condvar.notify_one();
                        }
                    }
                }
            },
            _ => quote!{},
        }
    }

    // generate handlestruct
    fn gen_struct_live(&mut self) -> TokenStream {

        let live_name           = name::live(&self.cust_name);
        let send_channel  = self.channels.live_field_sender.clone(); 
        let fn_new_self   = 
        if  self.aaa.edit.live_new.is_none() {
            self.gen_struct_live_method_new()
        } else { quote!{}};
        let methods = &self.live_methods; 
        let impl_drop     = self.gen_live_impl_drop();
        
        let id_debut = if self.aaa.id { quote!{ pub debut: std::sync::Arc<std::time::SystemTime>,}} else { quote!{}};
        let id_name = if self.aaa.id { quote!{pub name: String,}} else { quote!{}};
        quote!{
            #[derive(Clone,Debug)]
            pub struct #live_name {
                #send_channel
                #id_debut
                #id_name
            }

            impl #live_name  {

                #fn_new_self

                #(#methods)*
            }
            #impl_drop
        }
    }

    // this method will generate SDPL parts
    pub fn generate(&mut self) -> TokenStream {

         
        
        // populate
        self.gen_tokio_actor_model_bits();
        let impl_debut = self.gen_impl_debut();
        let id_impl_traits = self.gen_impl_eq_ord();
       
        // ACTOR
        let actor       = self.impl_block.clone();

        // SCRIPT 
        let script   = if  self.aaa.edit.script.is_none() {self.gen_script()} else { quote!{}};

        // DIRECT
        let direct   = if  self.aaa.edit.direct.is_none() {self.gen_impl_direct()} else { quote!{}};

        // PLAY 
        let play     = if  self.aaa.edit.play.is_none() {self.gen_func_play()} else { quote!{}};

        // LIVE
        let live     = if  self.aaa.edit.live.is_none() {self.gen_struct_live()} else { quote!{}};
        

        let res = quote! {

            #actor

            #script
                #impl_debut
            #direct

            #play

            #live
                #id_impl_traits
        };

        res

    }

}




struct Channels {

    live_field_sender:   TokenStream,
    play_input_receiver: TokenStream,
    new_live_send_recv:  TokenStream,
    live_meth_send_recv: TokenStream,

    script_field_output: std::boxed::Box<dyn Fn(Box<Type>) -> TokenStream>,

    live_send_input:     TokenStream,
    live_recv_output:    TokenStream,
}

impl Channels {

    pub fn new( lib: &AALib,
            channel: &AAChannel,
          cust_name: &Ident, 
                            ) -> Self {
        let live_field_sender:   TokenStream;
        let play_input_receiver: TokenStream;
        let new_live_send_recv:  TokenStream;
                                
        let type_ident = &name::script(cust_name);
        let (error_live_send,error_live_recv) = error::live_send_recv(cust_name);
        
        let mut live_meth_send_recv = 
            quote!{ let ( send, recv ) = oneshot::channel(); };

        let mut script_field_output: std::boxed::Box<dyn Fn(Box<Type>) -> TokenStream> =
            std::boxed::Box::new(|out_type: std::boxed::Box<Type>|quote!{ output: oneshot::Sender<#out_type>, }); 
       
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
                        script_field_output = std::boxed::Box::new(|out_type: std::boxed::Box<Type>|quote!{ output: tokio::sync::oneshot::Sender<#out_type>, });                
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
                        script_field_output = std::boxed::Box::new(|out_type: std::boxed::Box<Type>|quote!{ output: tokio::sync::oneshot::Sender<#out_type>, });                
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

        Self {

            live_field_sender,
            play_input_receiver, 
            new_live_send_recv , 
            live_meth_send_recv, 
            script_field_output, 
            live_send_input,
            live_recv_output,
        }
    }
}




