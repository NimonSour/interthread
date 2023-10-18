use crate::model::argument::Lib;
use crate::error;



use std::boxed::Box;
use syn::{ Ident,Type };
use quote::quote;
use proc_macro2::TokenStream;



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
    quote!{ let ( inter_send, inter_recv ) = oneshot::channel(); };
    
    let mut script_field_output: Box<dyn Fn(Box<Type>) -> TokenStream> =
    Box::new(|out_type: Box<Type>|quote!{ output: oneshot::Sender<#out_type>, }); 
    
    let mut live_send_input: TokenStream =
    quote!{let _ = self.sender.send(msg).await;};
    
    
    let mut live_recv_output: TokenStream = 
    quote!{ inter_recv.await.expect(#error_live_recv)};
    
    match  &self {
    
    Self::Unbounded    => {
    
       match  lib { 
    
           Lib::Std      => {
               live_field_sender   = quote!{ sender: std::sync::mpsc::Sender<#script_name #generics>, };   
               play_input_receiver = quote!{ receiver: std::sync::mpsc::Receiver<#script_name #generics>, }; 
               new_live_send_recv  = quote!{ let ( sender, receiver ) = std::sync::mpsc::channel(); };
               live_send_input     = quote!{ let _ = self.sender.send(msg).expect(#error_live_send);};
               live_recv_output    = quote!{ inter_recv.recv().expect(#error_live_recv)};
           },
    
           Lib::Tokio    => {
               live_field_sender   = quote!{ sender: tokio::sync::mpsc::UnboundedSender<#script_name #generics>, };
               play_input_receiver = quote!{ mut receiver: tokio::sync::mpsc::UnboundedReceiver<#script_name #generics>, }; 
               new_live_send_recv  = quote!{ let ( sender, receiver ) = tokio::sync::mpsc::unbounded_channel(); }; 
               live_meth_send_recv = quote!{ let ( inter_send, inter_recv ) = tokio::sync::oneshot::channel(); };
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
               live_recv_output    = quote!{ inter_recv.recv().expect(#error_live_recv)};
           },
           Lib::Tokio    => {
               live_field_sender   = quote!{ sender: tokio::sync::mpsc::Sender<#script_name #generics>, };
               play_input_receiver = quote!{ mut receiver: tokio::sync::mpsc::Receiver<#script_name #generics>, };
               new_live_send_recv  = quote!{ let ( sender, receiver ) = tokio::sync::mpsc::channel(#val); }; 
               live_meth_send_recv = quote!{ let ( inter_send, inter_recv ) = tokio::sync::oneshot::channel(); };
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

