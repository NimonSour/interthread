use crate::model::{ Lib,Vars,ActorAttributeArguments};
use crate::error;


// use std::boxed::Box;
use syn::{ Ident,Type };
use quote::quote;
use proc_macro2::TokenStream;



pub struct OneshotChannel {

    send: Ident,
    recv: Ident,
    lib:    Lib,
}

impl OneshotChannel {

    pub fn new( send: &Ident, recv: &Ident, lib: &Lib ) -> Self {
        Self{ send: send.clone(),
              recv: recv.clone(),
              lib: lib.clone()  }
    }
    
    pub fn get_decl(lib: &Lib, ty: Option<&Type>) -> TokenStream {
        let ty = ty.as_ref().map(|&x|  quote!{::<#x>});
        match lib {
            Lib::Tokio => quote!{ tokio::sync::oneshot::channel #ty ()},
                     _ => quote!{ oneshot::channel #ty () },
        }
    }
    pub fn get_send_type(lib: &Lib, ty: &Type) -> TokenStream {
        match lib {
            Lib::Tokio => quote!{ tokio::sync::oneshot::Sender<#ty> },
                     _ => quote!{ oneshot::Sender<#ty> },
        }
    } 
    pub fn get_recv_type(lib: &Lib, ty: &Type) -> TokenStream {
        match lib {
            Lib::Tokio => quote!{ tokio::sync::oneshot::Receiver<#ty> },
                     _ => quote!{ oneshot::Receiver<#ty> },
        }
    } 
    pub fn pat_type_send(&self, ty: &Type) -> TokenStream {
        let  Self{send,lib,..} = self;
        let ty = Self::get_send_type(lib,ty); 
        quote!{ #send : #ty }
    }
    pub fn pat_type_recv(&self, ty: &Type) -> TokenStream {
        let  Self{recv,lib,..} = self;
        let ty = Self::get_recv_type(lib,ty);
        quote!{ #recv : #ty } 
    }
    pub fn decl(&self, ty: Option<&Type>) -> TokenStream {
        let  Self{send,recv,..} = self;
        let decl = Self::get_decl(&self.lib,ty);
        quote!{ let( #send, #recv ) = #decl ; }
    }
    pub fn recv_call(&self, obj: &Ident, met: &Ident) -> TokenStream {
        let  Self{recv,lib,..} = self;
        let error = format!("'{obj}::{met}' from {recv}. Channel is closed!");

        match lib {
            Lib::Std =>  quote!{ #recv .recv().unwrap_or_else(|_error| core::panic!( #error ))} ,
                   _ =>  quote!{ #recv .await.unwrap_or_else(|_error| core::panic!( #error ))} ,
        }
    }
    pub fn send_call(&self,load: TokenStream, obj: &Ident, met: &Ident) -> TokenStream {
        let  Self{send,..} = self;
        let error = format!("'{obj}::{met}' from {send}. Sending on a closed channel!");
        quote!{ #send .send( #load ).unwrap_or_else(|_error| core::panic!( #error )) }
    }
}

pub struct MpscChannel {

    pub pat_type_sender:   TokenStream,    // live_field_sender:   
    pub pat_type_receiver: TokenStream,    // play_input_receiver: 
    pub declaration:       TokenStream,    // new_live_send_recv:  
    pub sender_call:       TokenStream,    // mut live_send_input: 
}

impl MpscChannel {

    pub fn new(
            Vars{
                sender,
              receiver,
             live_name,
                   msg,..
            } : &Vars,
            ActorAttributeArguments{
                channel,
                lib,..
            } : &ActorAttributeArguments,
 
           script_type: &Type ) -> Self {

        let error = format!("'{live_name}::method.send'. Channel is closed!");
        let pat_type_sender:   TokenStream;    // live_field_sender:   
        let pat_type_receiver: TokenStream;    // play_input_receiver: 
        let declaration:       TokenStream;    // new_live_send_recv:  
        let mut sender_call = quote!{ let _ = self.#sender.send(msg).await; };

        match  channel {
    
            Channel::Unbounded    => {
            
               match  lib { 
            
                   Lib::Std      => {
                       pat_type_sender   = quote!{ #sender: std::sync::mpsc::Sender<#script_type>, };   
                       pat_type_receiver = quote!{ #receiver: std::sync::mpsc::Receiver<#script_type>, }; 
                       declaration       = quote!{ let ( #sender, #receiver ) = std::sync::mpsc::channel(); };
                       sender_call       = quote!{ let _ = self.#sender.send(#msg).expect(#error);};
                   },
            
                   Lib::Tokio    => {
                       pat_type_sender   = quote!{ #sender: tokio::sync::mpsc::UnboundedSender<#script_type>, };
                       pat_type_receiver = quote!{ mut #receiver: tokio::sync::mpsc::UnboundedReceiver<#script_type>, }; 
                       declaration       = quote!{ let ( #sender, #receiver ) = tokio::sync::mpsc::unbounded_channel(); };                
                       sender_call       = quote!{ let _ = self.#sender.send(#msg).expect(#error);};
                   },
            
                   Lib::AsyncStd  => {
                       pat_type_sender   = quote!{ #sender: async_std::channel::Sender<#script_type>, };
                       pat_type_receiver = quote!{ #receiver: async_std::channel::Receiver<#script_type>, };
                       declaration       = quote!{ let ( #sender, #receiver ) = async_std::channel::unbounded(); };                    
                   },
            
                   Lib::Smol      => {
                       pat_type_sender   = quote!{ #sender: async_channel::Sender<#script_type>, };
                       pat_type_receiver = quote!{ #receiver: async_channel::Receiver<#script_type>, };
                       declaration       = quote!{ let ( #sender, #receiver ) =  async_channel::unbounded(); }; 
                   },
               }
            },
            Channel::Buffer(val)  => {
            
               match  lib { 
            
                   Lib::Std      => {
                       pat_type_sender   = quote!{ #sender: std::sync::mpsc::SyncSender<#script_type>, };
                       pat_type_receiver = quote!{ #receiver: std::sync::mpsc::Receiver<#script_type>, };
                       declaration       = quote!{ let ( #sender, #receiver ) = std::sync::mpsc::sync_channel(#val); };
                       sender_call       = quote!{ let _ = self.#sender.send(#msg).expect(#error);};
                   },
                   Lib::Tokio    => {
                       pat_type_sender   = quote!{ #sender: tokio::sync::mpsc::Sender<#script_type>, };
                       pat_type_receiver = quote!{ mut #receiver: tokio::sync::mpsc::Receiver<#script_type>, };
                       declaration       = quote!{ let ( #sender, #receiver ) = tokio::sync::mpsc::channel(#val); };               
                   },
            
                   Lib::AsyncStd  => {
                       pat_type_sender   = quote!{ #sender: async_std::channel::Sender<#script_type>, };
                       pat_type_receiver = quote!{ #receiver: async_std::channel::Receiver<#script_type>, };
                       declaration       = quote!{ let ( #sender, #receiver ) = async_std::channel::bounded(#val); };
                   },
            
                   Lib::Smol      => {
                       pat_type_sender   = quote!{ #sender: async_channel::Sender<#script_type>, };
                       pat_type_receiver = quote!{ #receiver: async_channel::Receiver<#script_type>, };
                       declaration       = quote!{ let ( #sender, #receiver ) = async_channel::bounded(#val); };
                   },
               }
            },
        };

        Self {
            pat_type_sender,
            pat_type_receiver,
            declaration,  
            sender_call, 
        }
    }

}


//-----------------------  ACTOR CHANNEL 

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Channel {

    Unbounded,
    Buffer(syn::LitInt),
}

impl Default for Channel {
    fn default() -> Self {
        Channel::Unbounded
    }
}


impl Channel {

    //OLD
    pub fn get_all(&self, lib: &Lib,
        script_name: &Ident,
          live_name: &Ident,   
           generics: &syn::TypeGenerics<'_> ) -> (
                               TokenStream,
                               TokenStream,
                            //    TokenStream,
                            //    TokenStream,
                            //    Box<dyn Fn(Box<Type>) -> TokenStream>,
                               TokenStream,
                               TokenStream ){
    
    let live_field_sender:   TokenStream;
    let play_input_receiver: TokenStream;
    let new_live_send_recv:  TokenStream;
    let mut live_send_input: TokenStream =
    quote!{let _ = self.sender.send(msg).await;};
                       
    // let type_ident = &name::script(cust_name);
    let (error_live_send,error_live_recv) = error::live_send_recv(live_name);
    
    // let mut live_meth_send_recv = 
    // quote!{ let ( inter_send, inter_recv ) = oneshot::channel(); };
    
    // let mut script_field_output: Box<dyn Fn(Box<Type>) -> TokenStream> =
    // Box::new(|out_type: Box<Type>|quote!{ inter_send : oneshot::Sender<#out_type>, }); 
    
  
    
    
    // let mut live_recv_output: TokenStream = 
    // quote!{ inter_recv.await.expect(#error_live_recv)};
    
    match  &self {
    
    Self::Unbounded    => {
    
       match  lib { 
    
           Lib::Std      => {
               live_field_sender   = quote!{ sender: std::sync::mpsc::Sender<#script_name #generics>, };   
               play_input_receiver = quote!{ receiver: std::sync::mpsc::Receiver<#script_name #generics>, }; 
               new_live_send_recv  = quote!{ let ( sender, receiver ) = std::sync::mpsc::channel(); };
               live_send_input     = quote!{ let _ = self.sender.send(msg).expect(#error_live_send);};
            //    live_recv_output    = quote!{ inter_recv.recv().expect(#error_live_recv)};
           },
    
           Lib::Tokio    => {
               live_field_sender   = quote!{ sender: tokio::sync::mpsc::UnboundedSender<#script_name #generics>, };
               play_input_receiver = quote!{ mut receiver: tokio::sync::mpsc::UnboundedReceiver<#script_name #generics>, }; 
               new_live_send_recv  = quote!{ let ( sender, receiver ) = tokio::sync::mpsc::unbounded_channel(); }; 
            //    live_meth_send_recv = quote!{ let ( inter_send, inter_recv ) = tokio::sync::oneshot::channel(); };
            //    script_field_output = Box::new(|out_type: Box<Type>|quote!{ tokio::sync::oneshot::Sender<#out_type>, });                
               live_send_input     = quote!{ let _ = self.sender.send(msg).expect(#error_live_send);};
           },
    
           Lib::AsyncStd  => {
               live_field_sender   = quote!{ sender: async_std::channel::Sender<#script_name #generics>, };
               play_input_receiver = quote!{ receiver: async_std::channel::Receiver<#script_name #generics>, };
               new_live_send_recv  = quote!{ let ( sender, receiver ) = async_std::channel::unbounded(); };                    
           },
    
           Lib::Smol      => {
               live_field_sender   = quote!{ sender: async_channel::Sender<#script_name #generics>, };
               play_input_receiver = quote!{ receiver: async_channel::Receiver<#script_name #generics>, };
               new_live_send_recv  = quote!{ let ( sender, receiver ) =  async_channel::unbounded(); }; 
           },
       }
    },
    Self::Buffer(val)  => {
    
       match  lib { 
    
           Lib::Std      => {
               live_field_sender   = quote!{ sender: std::sync::mpsc::SyncSender<#script_name #generics>, };
               play_input_receiver = quote!{ receiver: std::sync::mpsc::Receiver<#script_name #generics>, };
               new_live_send_recv  = quote!{ let ( sender, receiver ) = std::sync::mpsc::sync_channel(#val); };
               live_send_input     = quote!{ let _ = self.sender.send(msg).expect(#error_live_send);};
            //    live_recv_output    = quote!{ inter_recv.recv().expect(#error_live_recv)};
           },
           Lib::Tokio    => {
               live_field_sender   = quote!{ sender: tokio::sync::mpsc::Sender<#script_name #generics>, };
               play_input_receiver = quote!{ mut receiver: tokio::sync::mpsc::Receiver<#script_name #generics>, };
               new_live_send_recv  = quote!{ let ( sender, receiver ) = tokio::sync::mpsc::channel(#val); }; 
            //    live_meth_send_recv = quote!{ let ( inter_send, inter_recv ) = tokio::sync::oneshot::channel(); };
            //    script_field_output = Box::new(|out_type: Box<Type>|quote!{ tokio::sync::oneshot::Sender<#out_type>, });                
           },
    
           Lib::AsyncStd  => {
               live_field_sender   = quote!{ sender: async_std::channel::Sender<#script_name #generics>, };
               play_input_receiver = quote!{ receiver: async_std::channel::Receiver<#script_name #generics>, };
               new_live_send_recv  = quote!{ let ( sender, receiver ) = async_std::channel::bounded(#val); };
           },
    
           Lib::Smol      => {
               live_field_sender   = quote!{ sender: async_channel::Sender<#script_name #generics>, };
               play_input_receiver = quote!{ receiver: async_channel::Receiver<#script_name #generics>, };
               new_live_send_recv  = quote!{ let ( sender, receiver ) = async_channel::bounded(#val); };
           },
       }
    },
    }
    
    
    (
    live_field_sender,
    play_input_receiver, 
    new_live_send_recv , 
    // live_meth_send_recv, 
    // script_field_output, 
    live_send_input,
    // live_recv_output,
    )
    }
    
    //OLD
    /*
    
    pub fn get_all(&self, lib: &Lib,
        // channel: &AAChannel,
    //   cust_name: &Ident,
        script_name: &Ident,
          live_name: &Ident,   
        generics: &syn::TypeGenerics<'_> ) -> (
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
                       
    // let type_ident = &name::script(cust_name);
    let (error_live_send,error_live_recv) = error::live_send_recv(live_name);
    
    let mut live_meth_send_recv = 
    quote!{ let ( send, recv ) = oneshot::channel(); };
    
    let mut script_field_output: Box<dyn Fn(Box<Type>) -> TokenStream> =
    Box::new(|out_type: Box<Type>|quote!{ output: oneshot::Sender<#out_type>, }); 
    
    let mut live_send_input: TokenStream =
    quote!{let _ = self.sender.send(msg).await;};
    
    
    let mut live_recv_output: TokenStream = 
    quote!{ recv.await.expect(#error_live_recv)};
    
    match  &self {
    
    Self::Unbounded    => {
    
       match  lib { 
    
           Lib::Std      => {
               live_field_sender   = quote!{ sender: std::sync::mpsc::Sender<#script_name #generics>, };   
               play_input_receiver = quote!{ receiver: std::sync::mpsc::Receiver<#script_name #generics>, }; 
               new_live_send_recv  = quote!{ let ( sender, receiver ) = std::sync::mpsc::channel(); };
               live_send_input     = quote!{ let _ = self.sender.send(msg).expect(#error_live_send);};
               live_recv_output    = quote!{ recv.recv().expect(#error_live_recv)};
           },
    
           Lib::Tokio    => {
               live_field_sender   = quote!{ sender: tokio::sync::mpsc::UnboundedSender<#script_name #generics>, };
               play_input_receiver = quote!{ mut receiver: tokio::sync::mpsc::UnboundedReceiver<#script_name #generics>, }; 
               new_live_send_recv  = quote!{ let ( sender, receiver ) = tokio::sync::mpsc::unbounded_channel(); }; 
               live_meth_send_recv = quote!{ let ( send, recv ) = tokio::sync::oneshot::channel(); };
               script_field_output = Box::new(|out_type: Box<Type>|quote!{ output: tokio::sync::oneshot::Sender<#out_type>, });                
               live_send_input     = quote!{ let _ = self.sender.send(msg).expect(#error_live_send);};
           },
    
           Lib::AsyncStd  => {
               live_field_sender   = quote!{ sender: async_std::channel::Sender<#script_name #generics>, };
               play_input_receiver = quote!{ receiver: async_std::channel::Receiver<#script_name #generics>, };
               new_live_send_recv  = quote!{ let ( sender, receiver ) = async_std::channel::unbounded(); };                    
           },
    
           Lib::Smol      => {
               live_field_sender   = quote!{ sender: async_channel::Sender<#script_name #generics>, };
               play_input_receiver = quote!{ receiver: async_channel::Receiver<#script_name #generics>, };
               new_live_send_recv  = quote!{ let ( sender, receiver ) =  async_channel::unbounded(); }; 
           },
       }
    },
    Self::Buffer(val)  => {
    
       match  lib { 
    
           Lib::Std      => {
               live_field_sender   = quote!{ sender: std::sync::mpsc::SyncSender<#script_name #generics>, };
               play_input_receiver = quote!{ receiver: std::sync::mpsc::Receiver<#script_name #generics>, };
               new_live_send_recv  = quote!{ let ( sender, receiver ) = std::sync::mpsc::sync_channel(#val); };
               live_send_input     = quote!{ let _ = self.sender.send(msg).expect(#error_live_send);};
               live_recv_output    = quote!{ recv.recv().expect(#error_live_recv)};
           },
           Lib::Tokio    => {
               live_field_sender   = quote!{ sender: tokio::sync::mpsc::Sender<#script_name #generics>, };
               play_input_receiver = quote!{ mut receiver: tokio::sync::mpsc::Receiver<#script_name #generics>, };
               new_live_send_recv  = quote!{ let ( sender, receiver ) = tokio::sync::mpsc::channel(#val); }; 
               live_meth_send_recv = quote!{ let ( send, recv ) = tokio::sync::oneshot::channel(); };
               script_field_output = Box::new(|out_type: Box<Type>|quote!{ output: tokio::sync::oneshot::Sender<#out_type>, });                
           },
    
           Lib::AsyncStd  => {
               live_field_sender   = quote!{ sender: async_std::channel::Sender<#script_name #generics>, };
               play_input_receiver = quote!{ receiver: async_std::channel::Receiver<#script_name #generics>, };
               new_live_send_recv  = quote!{ let ( sender, receiver ) = async_std::channel::bounded(#val); };
           },
    
           Lib::Smol      => {
               live_field_sender   = quote!{ sender: async_channel::Sender<#script_name #generics>, };
               play_input_receiver = quote!{ receiver: async_channel::Receiver<#script_name #generics>, };
               new_live_send_recv  = quote!{ let ( sender, receiver ) = async_channel::bounded(#val); };
           },
       }
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

    */

}
