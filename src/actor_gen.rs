
use crate::attribute::ActorAttributeArguments;
use crate::name;
use crate::method;




pub fn impl_get_name(impl_block: &syn::ItemImpl) -> syn::Ident{
    match &*impl_block.self_ty {
        syn::Type::Path(tp) => tp.path.segments.first().unwrap().ident.clone(),
        _ => proc_macro_error::abort!(impl_block,"Internal Error.'actor_gen::impl_get_name'. Could not get item Impl's name!"),
    }
}
pub struct ActorMacroGeneration{
        
    name:                             syn::Ident,
    cust_name:                        syn::Ident,
    impl_block:                    syn::ItemImpl,
    aaa:                 ActorAttributeArguments,
    met_new:              method::ActorMethodNew, 
    actor_methods:      Vec<method::ActorMethod>,
    direct_arms:   Vec<proc_macro2::TokenStream>,
    live_methods:  Vec<proc_macro2::TokenStream>,
    script_fields: Vec<proc_macro2::TokenStream>,
    direct_async:                           bool,
    play_async:                             bool,
    channels:                           Channels,

}

impl ActorMacroGeneration {

    pub fn new( aaa: ActorAttributeArguments, impl_block: syn::ItemImpl ) -> Self {

        let name = impl_get_name(&impl_block);

        let (actor_methods, met_new) =
        method::get_methods( &name,impl_block.clone(),aaa.assoc );
        
        if met_new.is_none() {

            let msg = format!("Can not find public  method `new` or `try_new` for {:?} object.",name.to_string());
            let (note,help) = crate::error::met_new_note_help(&name);

        proc_macro_error::abort!(impl_block,msg;note=note;help=help);
    }
        
        // Giving a new name if specified 
        let cust_name   = if aaa.name.is_some(){ aaa.name.clone().unwrap() } else { name.clone() }; 
        let direct_async = actor_methods.iter().any(|x| x.is_async());
        let play_async   = Self::is_play_async( direct_async, &aaa.lib, &name);
        let channels = Channels::new( &aaa.lib, &aaa.channel, &name::script(&cust_name));
       
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

    pub fn is_play_async( direct_async: bool, lib: &crate::attribute::AALib, name: &syn::Ident ) -> bool {

        match lib {
            crate::attribute::AALib::Std => {
                if direct_async {
                    let msg = format!("Actor {:?} has 'async' methods but the runtime(lib) is not specified.", name.to_string());
                    proc_macro_error::abort!( proc_macro2::Span::call_site(), msg; help=crate::error::AVAIL_LIB );
                } else {
                    return false;
                }
            },
            _ => {
                return true;
            },
        }
    }

    fn live_static_method(&mut self,  name: syn::Ident, mut sig: syn::Signature, args: proc_macro2::TokenStream ) {
        
        method::change_signature_refer(&mut sig);
        let await_call = Self::await_token(sig.asyncness.is_some());
        let actor_name = &self.name;
        let gen_method = quote::quote! {

            pub #sig {
                #actor_name::#name #args #await_call
            }
        };
        self.live_methods.push(gen_method );
    }

    fn await_token( b: bool ) -> proc_macro2::TokenStream{
        if b { 
            quote::quote!{.await} 
        } else {
            quote::quote!{}
        }
    }
    
    fn async_token( b: bool ) -> proc_macro2::TokenStream{
        if b { 
            quote::quote!{async} 
        } else {
            quote::quote!{}
        }
    }

    pub fn send_in_recv_out(&self, sig: &mut syn::Signature ) -> (&proc_macro2::TokenStream, &proc_macro2::TokenStream)  {
        
        match self.aaa.lib {

            crate::attribute::AALib::Std => (),
            _ => {
                sig.asyncness = Some(syn::Token![async](proc_macro2::Span::call_site()));
            }
        }

        (
            &self.channels.live_send_input ,
            &self.channels.live_recv_output
        )
    }

    pub fn gen_tokio_actor_model_bits(&mut self){
        
        let script_name = name::script(&self.cust_name);
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
                        let arm_match        = quote::quote! { 
                            #script_field_name { input: #args_ident,  output: send }
                        };
                        let direct_arm       = quote::quote! {
                            #script_name::#arm_match => {send.send( actor.#ident #args_ident #await_call ).expect("'direct.send' Channel closed");}
                        };
                        self.direct_arms.push(direct_arm);
                        


                        // Live Method
                        let instant_channel = &self.channels.live_meth_send_recv;
                        let (send_input,recv_output) = self.send_in_recv_out(&mut sig);
                       
                        let live_method      = quote::quote! {

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

                        let script_field = quote::quote! {
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
                    let arm_match = quote::quote!{ 
                        #script_field_name{ input: #args_ident }
                    };
        
                    let direct_arm = quote::quote!{
                        #script_name::#arm_match => {actor.#ident #args_ident #await_call;},
                    };
                    self.direct_arms.push(direct_arm);




                    // Live Method
                    let (send_input,_) = self.send_in_recv_out(&mut sig);

                    let live_method = quote::quote!{
        
                        pub #sig {
                            let msg = #script_name::#arm_match ;
                            #send_input
                        }
                    };
                    self.live_methods.push( live_method );
                


                    // Script Field Struct
                    let script_field = quote::quote!{
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
                        let arm_match = quote::quote!{ 
                            #script_field_name{  output: send }
                        };
            
                        let direct_arm = quote::quote!{
                            #script_name::#arm_match => {send.send(actor.#ident #args_ident #await_call).expect("'direct.send' Channel closed");}
                        };
                        self.direct_arms.push(direct_arm);



                        // Live Method
                        let instant_channel = &self.channels.live_meth_send_recv;
                        let (send_input,recv_output) = self.send_in_recv_out(&mut sig);

                        let live_method = quote::quote!{
                        
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

                        let script_field = quote::quote!{
                            #script_field_name {
                                #output_type
                            }
                        };
                        self.script_fields.push(script_field);
                    }
                },
                method::ActorMethod::None { ident ,..} => {



                    // Direct Arm
                    let arm_match = quote::quote!{ 
                        #script_field_name {} 
                    };
        
                    let direct_arm = quote::quote!{
                        #script_name::#arm_match => {actor.#ident () #await_call;},
                    };
                    self.direct_arms.push(direct_arm);



                    // Live Method
                    let (send_input,_) = self.send_in_recv_out(&mut sig);

                    let live_method = quote::quote!{
                    
                        pub #sig {
                            let msg = #script_name::#arm_match ;
                            #send_input
                        }
                    };
                    self.live_methods.push( live_method );
                


                    // Script Field Struct
                    let script_field = quote::quote!{
                        
                        #script_field_name {}
                    };
                    self.script_fields.push(script_field);
                },
            }
        } 
    }

    fn live_new_spawn(&self, play: proc_macro2::TokenStream ) -> proc_macro2::TokenStream {
        match self.aaa.lib {
            crate::attribute::AALib::Std         => {
                quote::quote!{ std::thread::spawn(|| { #play });}
            },
            crate::attribute::AALib::Smol        => {
                quote::quote!{ smol::spawn( #play ).detach();} 
            },
            crate::attribute::AALib::Tokio       => {
                quote::quote!{ tokio::spawn(#play);}
            },
            crate::attribute::AALib::AsyncStd    => {
                quote::quote!{ async_std::task::spawn(#play);}
            },
        }
    }


    fn gen_script(&mut self) -> proc_macro2::TokenStream {

        let script_name       = name::script(&self.cust_name);
        let fields = self.script_fields.clone();

        quote::quote! {
            #[derive(Debug)]
            pub enum #script_name {
                #(#fields),*
            }
        }
    }


    // DIRECT
    fn gen_impl_direct(&self) -> proc_macro2::TokenStream {

        let name            = self.name.clone();
        let arms = self.direct_arms.clone();

        let script_name     = name::script(&self.cust_name);
        let direct_name     = name::direct(&self.cust_name);
        let decl_async= Self::async_token(self.direct_async);
        quote::quote!{
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
    fn gen_func_play(&mut self) -> proc_macro2::TokenStream {

        let name               = self.name.clone();
        let play_name          = name::play(&self.cust_name);
        let script_name        = name::script(&self.cust_name);
        let direct_name        = name::direct(&self.cust_name);


        let recv_channel  = &self.channels.play_input_receiver; 
        let await_call     = Self::await_token(self.direct_async);
        let async_decl     = Self::async_token(self.play_async);
        
        let end_of_play = self.end_of_play_statment();

        match self.aaa.channel {

            crate::attribute::AAChannel::Unbounded |
            crate::attribute::AAChannel::Buffer(_) => match self.aaa.lib {

                crate::attribute::AALib::Std => {
                    quote::quote! {
        
                        pub fn #play_name ( #recv_channel mut actor: #name ) {
                            while let Ok(msg) = receiver.recv(){
                                msg.#direct_name ( &mut actor );
                            }
                            #end_of_play
                        }
                    }
                },

                crate::attribute::AALib::Tokio => {
                    quote::quote! {
        
                        pub #async_decl fn #play_name ( #recv_channel mut actor: #name ) {
                            while let Some(msg) = receiver.recv().await{
                                msg.#direct_name ( &mut actor ) #await_call;
                            }
                            #end_of_play
                        }
                    }
                },

                _ => { 
                    quote::quote! {
    
                        pub #async_decl fn #play_name ( #recv_channel mut actor: #name ) {
                            while let Ok(msg) = receiver.recv().await {
            
                                msg.#direct_name ( &mut actor ) #await_call;
                            }
                            #end_of_play
                        }
                    }
                },
            },

            crate::attribute::AAChannel::Inter => {

                quote::quote!{
                    pub #async_decl fn #play_name ( #recv_channel mut actor: #name ) {

                        let queuing = || -> Option<Vec< #script_name >> {
                            let mut guard = queue.lock().unwrap();
                            while guard.as_ref().unwrap().is_empty() {
                                if std::sync::Arc::strong_count(&queue) > 1{
                                    guard = condvar.wait(guard).unwrap();
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

    fn end_of_play_statment(&self) -> proc_macro2::TokenStream {
        
        let str_name    = &self.name.to_string() ;
        let msg     = quote::quote!{ #str_name }; 

        quote::quote!{
            eprintln!("{} end of life ...", #msg);
        }
    }

    fn gen_struct_live_method_new(&mut self) -> proc_macro2::TokenStream { 

        let name                         = &self.name;//.clone();
        let play_name                     = name::play(&self.cust_name);
        let new_sig                  = &self.met_new.new_sig;//.clone();
        let (args_ident, _ )        = method::arguments_ident_type(&self.met_new.get_arguments());

        let send_recv_channel     = &self.channels.new_live_send_recv;
        let func_new_name                = new_sig.ident.clone();
        // add a '?' to the end of 'actor' declaration 
        let unwrapped              = self.met_new.unwrap_sign(); // if self.met_new.res_opt.is_none(){ quote::quote!{}} else { quote::quote!{?}};

        // let return_statement =  match self.met_new.res_opt {

        //     Some(true)  =>  quote::quote!{ Ok ( actor_live )},
        //     Some(false) =>  quote::quote!{ Some( actor_live )},
        //     None        =>  quote::quote!{ actor_live },
            
        // };

        let live_var = quote::format_ident!("actor_live");
        let return_statement = self.met_new.live_ret_statement(&live_var);

        let (init_actor, play_args) =
        match  self.aaa.channel {
            crate::attribute::AAChannel::Inter => {
                ( quote::quote!{ Self{ queue: queue.clone(), condvar: condvar.clone() } }, quote::quote!{ queue, condvar, actor  } )
            },
            _  => {
                ( quote::quote!{ Self{ sender} }, quote::quote!{ receiver, actor } )
            },
        };

        let play_call =  quote::quote!{ #play_name(#play_args) }; 
        let spawn     =  self.live_new_spawn(play_call);

        quote::quote!{

            pub #new_sig {
                #send_recv_channel
                let actor = #name:: #func_new_name #args_ident #unwrapped;
                let #live_var = #init_actor;
                #spawn
                #return_statement
            }
        }
    }
    
    fn gen_live_impl_drop(&self) ->  proc_macro2::TokenStream{
        let live_name  = name::live(&self.cust_name);

        match self.aaa.channel{

            crate::attribute::AAChannel::Inter =>{ 
                quote::quote!{
                    impl Drop for #live_name{
                        fn drop(&mut self){
                            self.condvar.notify_one();
                        }
                    }
                }
            },
            _ => quote::quote!{},
        }
    }

    // generate handlestruct
    fn gen_struct_live(&mut self) -> proc_macro2::TokenStream {

        let live_name           = name::live(&self.cust_name);
        let send_channel  = self.channels.live_field_sender.clone(); 
        let fn_new_self   = 
        if  self.aaa.edit.live_new.is_none() {
            self.gen_struct_live_method_new()
        } else { quote::quote!{}};
        let methods = &self.live_methods; 
        let impl_drop     = self.gen_live_impl_drop();
   
        quote::quote!{
            #[derive(Clone,Debug)]
            pub struct #live_name {
                #send_channel
            }

            impl #live_name  {

                #fn_new_self

                #(#methods)*
            }
            #impl_drop
        }
    }

    // this method will generate SDPL parts
    pub fn generate(&mut self) -> proc_macro2::TokenStream {

         
        
        // populate
        self.gen_tokio_actor_model_bits();

        // ACTOR
        let actor       = self.impl_block.clone();

        // SCRIPT 
        let script   = if  self.aaa.edit.script.is_none() {self.gen_script()} else { quote::quote!{}};

        // DIRECT
        let direct   = if  self.aaa.edit.direct.is_none() {self.gen_impl_direct()} else { quote::quote!{}};

        // PLAY 
        let play     = if  self.aaa.edit.play.is_none() {self.gen_func_play()} else { quote::quote!{}};

        // LIVE
        let live     = if  self.aaa.edit.live.is_none() {self.gen_struct_live()} else { quote::quote!{}};


        let res = quote::quote! {

            #actor

            #script

            #direct

            #play

            #live
        };

        res

    }

}




struct Channels {

    live_field_sender:   proc_macro2::TokenStream,
    play_input_receiver: proc_macro2::TokenStream,
    new_live_send_recv:  proc_macro2::TokenStream,
    live_meth_send_recv: proc_macro2::TokenStream,

    script_field_output: std::boxed::Box<dyn Fn(Box<syn::Type>) -> proc_macro2::TokenStream>,

    live_send_input:     proc_macro2::TokenStream,
    live_recv_output:    proc_macro2::TokenStream,
}

impl Channels {

    pub fn new( lib: &crate::attribute::AALib,
            channel: &crate::attribute::AAChannel,
         type_ident: &syn::Ident, 
                            ) -> Self {
        let live_send_error = quote::quote!{"'Live::method.send'. Channel is closed!"};
        let live_recv_error = quote::quote!{"'Live::method.recv'. Channel is closed!"};
        let live_field_sender:   proc_macro2::TokenStream;
        let play_input_receiver: proc_macro2::TokenStream;
        let new_live_send_recv:  proc_macro2::TokenStream;

        let mut live_meth_send_recv = 
            quote::quote!{ let ( send, recv ) = oneshot::channel(); };

        let mut script_field_output: std::boxed::Box<dyn Fn(Box<syn::Type>) -> proc_macro2::TokenStream> =
            std::boxed::Box::new(|out_type: std::boxed::Box<syn::Type>|quote::quote!{ output: oneshot::Sender<#out_type>, }); 
       
        let mut live_send_input: proc_macro2::TokenStream =
            quote::quote!{let _ = self.sender.send(msg).await;};


        let mut live_recv_output: proc_macro2::TokenStream = 
            quote::quote!{ recv.await.expect(#live_recv_error)};

        match  channel {

            crate::attribute::AAChannel::Unbounded    => {

                match  lib { 

                    crate::attribute::AALib::Std      => {
                        live_field_sender   = quote::quote!{ sender: std::sync::mpsc::Sender<#type_ident>, };   
                        play_input_receiver = quote::quote!{ receiver: std::sync::mpsc::Receiver<#type_ident>, }; 
                        new_live_send_recv  = quote::quote!{ let ( sender, receiver ) = std::sync::mpsc::channel(); };
                        live_send_input     = quote::quote!{ let _ = self.sender.send(msg).expect(#live_send_error);};
                        live_recv_output    = quote::quote!{ recv.recv().expect(#live_recv_error)};
                    },

                    crate::attribute::AALib::Tokio    => {
                        live_field_sender   = quote::quote!{ sender: tokio::sync::mpsc::UnboundedSender<#type_ident>, };
                        play_input_receiver = quote::quote!{ mut receiver: tokio::sync::mpsc::UnboundedReceiver<#type_ident>, }; 
                        new_live_send_recv  = quote::quote!{ let ( sender, receiver ) = tokio::sync::mpsc::unbounded_channel(); }; 
                        live_meth_send_recv = quote::quote!{ let ( send, recv ) = tokio::sync::oneshot::channel(); };
                        script_field_output = std::boxed::Box::new(|out_type: std::boxed::Box<syn::Type>|quote::quote!{ output: tokio::sync::oneshot::Sender<#out_type>, });                
                        live_send_input     = quote::quote!{ let _ = self.sender.send(msg).expect(#live_send_error);};
                    },

                    crate::attribute::AALib::AsyncStd  => {
                        live_field_sender   = quote::quote!{ sender: async_std::channel::Sender<#type_ident>, };
                        play_input_receiver = quote::quote!{ receiver: async_std::channel::Receiver<#type_ident>, };
                        new_live_send_recv  = quote::quote!{ let ( sender, receiver ) = async_std::channel::unbounded(); };                    
                    },

                    crate::attribute::AALib::Smol      => {
                        live_field_sender   = quote::quote!{ sender: async_channel::Sender<#type_ident>, };
                        play_input_receiver = quote::quote!{ receiver: async_channel::Receiver<#type_ident>, };
                        new_live_send_recv  = quote::quote!{ let ( sender, receiver ) =  async_channel::unbounded(); }; 
                    },
                }
            },
            crate::attribute::AAChannel::Buffer(val)    => {

                match  lib { 

                    crate::attribute::AALib::Std      => {
                        live_field_sender   = quote::quote!{ sender: std::sync::mpsc::SyncSender<#type_ident>, };
                        play_input_receiver = quote::quote!{ receiver: std::sync::mpsc::Receiver<#type_ident>, };
                        new_live_send_recv  = quote::quote!{ let ( sender, receiver ) = std::sync::mpsc::sync_channel(#val); };
                        live_send_input     = quote::quote!{ let _ = self.sender.send(msg).expect(#live_send_error);};
                        live_recv_output    = quote::quote!{ recv.recv().expect(#live_recv_error)};
                    },
                    crate::attribute::AALib::Tokio    => {
                        live_field_sender   = quote::quote!{ sender: tokio::sync::mpsc::Sender<#type_ident>, };
                        play_input_receiver = quote::quote!{ mut receiver: tokio::sync::mpsc::Receiver<#type_ident>, };
                        new_live_send_recv  = quote::quote!{ let ( sender, receiver ) = tokio::sync::mpsc::channel(#val); }; 
                        live_meth_send_recv = quote::quote!{ let ( send, recv ) = tokio::sync::oneshot::channel(); };
                        script_field_output = std::boxed::Box::new(|out_type: std::boxed::Box<syn::Type>|quote::quote!{ output: tokio::sync::oneshot::Sender<#out_type>, });                
                    },

                    crate::attribute::AALib::AsyncStd  => {
                        live_field_sender   = quote::quote!{ sender: async_std::channel::Sender<#type_ident>, };
                        play_input_receiver = quote::quote!{ receiver: async_std::channel::Receiver<#type_ident>, };
                        new_live_send_recv  = quote::quote!{ let ( sender, receiver ) = async_std::channel::bounded(#val); };
                    },

                    crate::attribute::AALib::Smol      => {
                        live_field_sender   = quote::quote!{ sender: async_channel::Sender<#type_ident>, };
                        play_input_receiver = quote::quote!{ receiver: async_channel::Receiver<#type_ident>, };
                        new_live_send_recv  = quote::quote!{ let ( sender, receiver ) = async_channel::bounded(#val); };
                    },
                }
            },
            crate::attribute::AAChannel::Inter  => {

                live_field_sender   = quote::quote!{ 
                    queue: std::sync::Arc<std::sync::Mutex<Option<Vec<#type_ident>>>>,
                    condvar:                       std::sync::Arc<std::sync::Condvar>,
                };
                play_input_receiver = quote::quote!{ 
                    queue: std::sync::Arc<std::sync::Mutex<Option<Vec<#type_ident>>>>,
                    condvar:                       std::sync::Arc<std::sync::Condvar>,
                };
                new_live_send_recv  = quote::quote!{ 
                    let queue       = std::sync::Arc::new(std::sync::Mutex::new(Some(vec![])));
                    let condvar     = std::sync::Arc::new(std::sync::Condvar::new());
                };


                live_send_input     =  quote::quote!{
                    {
                        let mut guard = self.queue.lock().expect("'Live::method'.Failed to unwrap queue MutexGuard!");
            
                        guard.as_mut()
                        .map(|s| s.push(msg));
                    }
                    self.condvar.notify_one();
                };

                live_recv_output     =  quote::quote!{recv.recv().expect(#live_recv_error)};
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


