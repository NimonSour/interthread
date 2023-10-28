pub mod channel;
pub mod debut;
pub mod edit;
pub mod interact;

pub use channel::*;
pub use debut::*;
pub use edit::*;
pub use interact::*;



use crate::error;

use proc_macro2::{Span,TokenStream};
use proc_macro_error::abort;
use quote::{quote,format_ident};
use syn::{Ident,Meta};

use std::path::PathBuf;



//-----------------------  EXAMPLE EXPAND
#[derive(Debug, Copy,Eq, PartialEq, Clone)]

pub enum SubModel{
    Actor,
    GroupActor,
    Group,
}


//-----------------------  EXAMPLE EXPAND
#[derive(Debug,Copy, Eq, PartialEq, Clone)]
pub enum Model {
    Actor,
    // GroupActor,
    Group,
}
impl Model {

    pub fn get_invers(&self) -> Self {
        match self {
            Self::Actor => Self::Group,
            Self::Group => Self::Actor,
        }
    }
}

// impl Model {

//     pub fn to_str(&self) -> &'static str {

//         match self {
//             Self::Actor => crate::ACTOR,
//             Self::Group => crate::GROUP,
//         }
//     }
// }
impl std::fmt::Display for Model {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Actor => write!(f,"{}",crate::ACTOR),
            // Self::GroupActor => write!(f,"{}_{}",crate::GROUP,crate::ACTOR),
            Self::Group => write!(f,"{}",crate::GROUP),
        }
    }
}

//-----------------------  ACTOR LIB

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Lib {
    Std,
    Smol,
    Tokio,
    AsyncStd,
}

impl Lib {

    pub fn from( s: &str  ) -> Self {

        match s {

            val if val == "std"       =>  Lib::Std,
            val if val == "smol"      =>  Lib::Smol,
            val if val == "tokio"     =>  Lib::Tokio,
            val if val == "async_std" =>  Lib::AsyncStd,
            val => {
                let msg = format!("Unknown option  -  {:?} for 'channel' ", val);
                abort!( s, msg; help=error::AVAIL_LIB );   
            } 
        }
        

    }
    
    pub fn method_new_spawn(&self, play_args: &TokenStream, script_name: &Ident) -> TokenStream {

        match &self {
            Lib::Std      => {
                quote!{ std::thread::spawn(move|| { #script_name :: play(#play_args) } );}
            },
            Lib::Smol     => {
                quote!{ smol::spawn( #script_name :: play(#play_args) ).detach();} 
            },
            Lib::Tokio    => {
                quote!{ tokio::spawn( #script_name :: play(#play_args) );}
            },
            Lib::AsyncStd => {
                quote!{ async_std::task::spawn( #script_name :: play(#play_args) );}
            },
        }
    }


}

impl Default for Lib {
    fn default() -> Self {
        Lib::Std
    }
}

//-----------------------  ACTOR FILE
#[derive(Debug, Eq, PartialEq, Clone)]

pub struct EditAttribute {

    pub path:              PathBuf,
    pub attr:       syn::Attribute,
    pub attrs: Vec<syn::Attribute>,
    pub remove:               bool,
    pub idents: Option<Vec<Ident>>,
}


impl EditAttribute {

    pub fn get_attr_str(&self) -> String {
        let attr = &self.attr;
        let mut attr_str = quote::quote!{ #attr }.to_string();
        attr_str = (&attr_str).replace(crate::LINE_ENDING,"");
        attr_str = (&attr_str).replace(" ","");
        return attr_str;
    }
}

pub enum Edit{
    Actor(EditActor),
    Group(EditGroup),
}

impl Edit {

    pub fn new(model:&Model) -> Self {

        match model {
            Model::Actor      => Self::Actor(EditActor::default()),
            // Model::GroupActor => Self::Actor(EditActor::default()),
            Model::Group      => Self::Group(EditGroup::default()),
        }
    }

    pub fn parse(&mut self, meta: &Meta ) {
        match self {
            Self::Actor(edit_actor) => edit_actor.parse(meta), 
            Self::Group(edit_group) => edit_group.parse(meta), 
        }
    }

    pub fn is_any_active(&self) -> bool {
        match &self {
            Self::Actor(edit_actor) => edit_actor.is_any_active(), 
            Self::Group(edit_group) => edit_group.is_any_active(), 
        }
    }
    pub fn get_remove(&self) -> bool {
        match &self {
            Self::Actor(edit_actor) => edit_actor.remove, 
            Self::Group(edit_group) => edit_group.remove, 
        }
    }

    pub fn get_some_ident_list(&self) -> Option<Vec<Ident>> { 

        match &self {
            Self::Actor(_) => None, 
            Self::Group(edit_group) => {
                edit_group.edits
                .as_ref()
                .map(|x| 
                    x.iter()
                     .map(|i|i.0.clone())
                     .collect::<Vec<_>>())
            }, 
        }
    }

    // pub fn get_attr_str(&self) -> String {

    //     if let Some(attr)  = match &self {
    //         Self::Actor(edit_actor) => edit_actor.attr.as_ref().map(|e| e.attr), 
    //         Self::Group(edit_group) => edit_group.attr.as_ref().map(|e| e.attr), 
    //     }{
    //         // let attr = &edit_attr.attr;
    //         let mut attr_str = quote::quote!{ #attr }.to_string();
    //         attr_str = (&attr_str).replace(crate::LINE_ENDING,"");
    //         attr_str = (&attr_str).replace(" ","");
    //         return attr_str;
    //     }
    //     "".to_string()
    // }


}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_functions(){

        let sig: syn::Signature     = syn::parse_quote!{ fn foo ( send: oneshot::Sender<&'static str>) -> i8 };  
        let mut my_type:  syn::Type = syn::parse_quote!{ oneshot::Receiver };
        let argument  =  sig.inputs.last().unwrap();
        
        let target = quote::format_ident!("Sender");
        
        if let syn::FnArg::Typed(arg) = &argument{
            // ty = arg.ty.clone();

            if let syn::Type::Path(type_path) = &*arg.ty {
                if let Some(seg) = type_path.path.segments.last(){
                    if target.eq(&seg.ident) {
                        
                        let gen_args = seg.arguments.clone();
                        if let syn::PathArguments::AngleBracketed(ang_brck_gen_arg) = &gen_args{
                            if let Some( gen ) = ang_brck_gen_arg.args.first(){
                                if let syn::GenericArgument::Type(ty) = gen {
                                    let ty_str = quote!{#ty}.to_string();
                                    println!("ty - {}",ty_str); 
                                } else { println!("Not generic argument Type "); }
                            } else { println!("Not first arg"); }
                        } else { println!("Not angle bracketed"); }
                         
                        // let mut my_type:  syn::Type = syn::parse_quote!{ oneshot::Receiver #gen_args };

                        // println!(" new Receiver -  {}", quote::quote!{#my_type}.to_string())
                    } else { println!("Not equal to Sender!") }
                }
            };
        };
        






        println!("fn argument -  {}", quote::quote!{#argument}.to_string() );

    }



    
}