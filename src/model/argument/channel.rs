use crate::model::{ Lib,ConstVars,ActorAttributeArguments};

use syn::{ parse2, FnArg,Ident,Type };
use quote::quote;
use proc_macro2::TokenStream;


#[derive(Debug,Clone)]
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
    pub fn send_type(&self,ty: &Type) -> TokenStream {
        Self::get_send_type(&self.lib,ty)
    } 
    pub fn recv_type(&self,ty: &Type) -> TokenStream {
        Self::get_recv_type(&self.lib,ty)
    } 
    pub fn pat_type_send(&self, ty: &Type) -> TokenStream {
        let  Self{send,lib,..} = self;
        let ty = Self::get_send_type(lib,ty); 
        quote!{ #send : #ty }
    }

    // pub fn pat_type_recv(&self, ty: &Type) -> TokenStream {
    //     let  Self{recv,lib,..} = self;
    //     let ty = Self::get_recv_type(lib,ty);
    //     quote!{ #recv : #ty } 
    // }

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
#[derive(Clone)]
pub struct MpscChannel {
    pub type_sender:       TokenStream,    
    pub type_receiver:     TokenStream,     
    pub pat_type_sender:   FnArg,    
    pub pat_type_receiver: FnArg,   
    pub declaration:       TokenStream,    
    pub sender_call:       TokenStream,    
}

impl MpscChannel {

    pub fn new(
            ConstVars{ sender,receiver,msg,..} : &ConstVars,
            ActorAttributeArguments{ channel, lib,..} : &ActorAttributeArguments,
            script_type: &Type,
            live_name: &Ident ) -> Self {

        let error = format!("'{live_name}::method.send'. Channel is closed!");
        let type_sender:       TokenStream;    
        let type_receiver:     TokenStream;
        let pat_type_sender:   FnArg;    
        let pat_type_receiver: FnArg;
        let declaration_call:  TokenStream;    
        let declaration:       TokenStream;    
        let mut sender_call = quote!{ let _ = self.#sender.send(msg).await; };
        let exp_error_msg = "argument::channel::MpscChannel::new Failed to pars FnArg!";
        match  channel {
    
            Channel::Unbounded    => {
            
               match  lib { 
            
                   Lib::Std      => {
                        type_sender       = quote!{ std::sync::mpsc::Sender<#script_type>};    
                        type_receiver     = quote!{ std::sync::mpsc::Receiver<#script_type>};
                        pat_type_sender   = parse2(quote!{ #sender: #type_sender }).expect(exp_error_msg);   
                        pat_type_receiver = parse2(quote!{ #receiver: #type_receiver }).expect(exp_error_msg); 
                        declaration_call  = quote!{ std::sync::mpsc::channel() };
                        declaration       = quote!{ let ( #sender, #receiver ) = #declaration_call ;};
                        sender_call       = quote!{ let _ = self.#sender.send(#msg).expect(#error);};
                   },
            
                   Lib::Tokio    => {
                        type_sender       = quote!{ tokio::sync::mpsc::UnboundedSender<#script_type>};    
                        type_receiver     = quote!{ tokio::sync::mpsc::UnboundedReceiver<#script_type>};
                        pat_type_sender   = parse2(quote!{ #sender: #type_sender }).expect(exp_error_msg);
                        pat_type_receiver = parse2(quote!{ mut #receiver: #type_receiver }).expect(exp_error_msg); 
                        declaration_call  = quote!{ tokio::sync::mpsc::unbounded_channel() };
                        declaration       = quote!{ let ( #sender, #receiver ) = #declaration_call ;};                
                        sender_call       = quote!{ let _ = self.#sender.send(#msg).expect(#error);};
                   },
            
                   Lib::AsyncStd  => {
                        type_sender       = quote!{ async_std::channel::Sender<#script_type>};    
                        type_receiver     = quote!{ async_std::channel::Receiver<#script_type>};
                        pat_type_sender   = parse2(quote!{ #sender: #type_sender }).expect(exp_error_msg);
                        pat_type_receiver = parse2(quote!{ #receiver: #type_receiver }).expect(exp_error_msg);
                        declaration_call  = quote!{ async_std::channel::unbounded() };
                        declaration       = quote!{ let ( #sender, #receiver ) = #declaration_call ;};                    
                   },
            
                   Lib::Smol      => {
                        type_sender       = quote!{ async_channel::Sender<#script_type>};    
                        type_receiver     = quote!{ async_channel::Receiver<#script_type>};
                        pat_type_sender   = parse2(quote!{ #sender: #type_sender }).expect(exp_error_msg);
                        pat_type_receiver = parse2(quote!{ #receiver: #type_receiver }).expect(exp_error_msg);
                        declaration_call  = quote!{ async_channel::unbounded() };
                        declaration       = quote!{ let ( #sender, #receiver ) = #declaration_call ;}; 
                   },
               }
            },
            Channel::Buffer(val)  => {
            
               match  lib { 
            
                   Lib::Std      => {
                        type_sender       = quote!{ std::sync::mpsc::SyncSender<#script_type> };    
                        type_receiver     = quote!{ std::sync::mpsc::Receiver<#script_type> };
                        pat_type_sender   = parse2(quote!{ #sender: #type_sender }).expect(exp_error_msg);
                        pat_type_receiver = parse2(quote!{ #receiver: #type_receiver}).expect(exp_error_msg);
                        declaration_call  = quote!{ std::sync::mpsc::sync_channel(#val) };
                        declaration       = quote!{ let ( #sender, #receiver ) = #declaration_call ;};
                        sender_call       = quote!{ let _ = self.#sender.send(#msg).expect(#error);};
                   },
                   Lib::Tokio    => {
                        type_sender       = quote!{ tokio::sync::mpsc::Sender<#script_type> };    
                        type_receiver     = quote!{ tokio::sync::mpsc::Receiver<#script_type> };
                        pat_type_sender   = parse2(quote!{ #sender: #type_sender }).expect(exp_error_msg);
                        pat_type_receiver = parse2(quote!{ mut #receiver: #type_receiver }).expect(exp_error_msg);
                        declaration_call  = quote!{ tokio::sync::mpsc::channel(#val) };
                        declaration       = quote!{ let ( #sender, #receiver ) = #declaration_call ;};               
                   },
            
                   Lib::AsyncStd  => {
                        type_sender       = quote!{ async_std::channel::Sender<#script_type> };    
                        type_receiver     = quote!{ async_std::channel::Receiver<#script_type> };
                        pat_type_sender   = parse2(quote!{ #sender: #type_sender }).expect(exp_error_msg);
                        pat_type_receiver = parse2(quote!{ #receiver: #type_receiver }).expect(exp_error_msg);
                        declaration_call  = quote!{ async_std::channel::bounded(#val) };
                        declaration       = quote!{ let ( #sender, #receiver ) = #declaration_call ;};
                   },
            
                   Lib::Smol      => {
                        type_sender       = quote!{ async_channel::Sender<#script_type> };    
                        type_receiver     = quote!{ async_channel::Receiver<#script_type> };
                        pat_type_sender   = parse2(quote!{ #sender: #type_sender }).expect(exp_error_msg);
                        pat_type_receiver = parse2(quote!{ #receiver: #type_receiver }).expect(exp_error_msg);
                        declaration_call  = quote!{ async_channel::bounded(#val) };
                        declaration       = quote!{ let ( #sender, #receiver ) = #declaration_call ;};
                   },
               }
            },
        };
        
        Self {
            type_sender,
            type_receiver,
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

