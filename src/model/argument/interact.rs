use crate::error;


use proc_macro2::TokenStream;
use syn::{Type,Ident,Signature,FnArg,Token, punctuated::Punctuated};
use quote::{quote,format_ident};

use proc_macro_error::abort;
use proc_macro::Span;


pub static INTER_SEND: &'static str = "inter_send";
pub static INTER_RECV: &'static str = "inter_recv";


#[derive(Debug,Clone)]
pub enum InterOneshot {

    Recv(Type),
    Send(Type),
}

impl InterOneshot {

    pub fn get_name(&self) -> Ident {

        match &self {
            Self::Recv(_) => format_ident!("{INTER_RECV}"), 
            Self::Send(_) => format_ident!("{INTER_SEND}"), 
        }
    }   
}


#[derive(Debug,Clone)]
pub struct InterGetter { 
    name:     Ident, 
    ty:        Type,
}


impl InterGetter {

    pub fn get_name(&self) -> Ident {
        self.name.clone()
    }
}

#[derive(Debug,Clone)]

pub enum InterVariable {

    Oneshot(InterOneshot),
    Getter(InterGetter),
}


impl InterVariable {

    pub fn get_name(&self) -> Ident {
        match &self {
            Self::Oneshot(var) => var.get_name(),
            Self::Getter(var) => var.get_name(),
        }
    }



}

pub fn some_inter_var( ident: &Ident, ty: &Type, ret: bool ) -> Result<Option<InterVariable>,String> {

    let msg_inter_var = format!("Expected `inter_variable`, found {}.",ident);

    let ident_str = ident.to_string();
    let ident_split_coll = ident_str.split('_').collect::<Vec<_>>();

    if ident_split_coll.len() < 2 {
        let first = ident_split_coll[0];
        if first.eq("inter"){
            let second = ident_split_coll[1..].join("_");
            if second.eq("send"){
                // check if is sender and extract the Type it sends 
                match oneshot_get_type( ty, "Sender"){
                    Ok(new_ty) => { Ok(Some(InterVariable::Oneshot(InterOneshot::Send(new_ty.clone())))) },
                    Err(e) => {Err(e)},
                }
            }
            
            else if second.eq("recv"){

                if ret { return Err("`inter_recv` not available inside methods returning a Type.".to_string()); }

                match oneshot_get_type( ty, "Receiver"){
                    Ok(new_ty) => { Ok(Some(InterVariable::Oneshot(InterOneshot::Recv(new_ty.clone())))) },
                    Err(e) => {Err(e)},
                }

            } else {
                Ok(Some(InterVariable::Getter(InterGetter{ name:ident.clone(), ty:ty.clone()})))
             }

        } else { Ok(None) }

    } else { Ok(None) }

}


pub fn oneshot_get_type( ty: &Type, target: &str ) -> Result<Type,String> {

    let msg_path = format!("Expected a path type, ::{}<Type> .",&target);
    let msg_args = format!("Unexpected arguments for {}<?>.",target);

    if let syn::Type::Path(type_path) = &ty {
        if let Some(seg) = type_path.path.segments.last(){
            if target.eq(target) {
                
                let gen_args = seg.arguments.clone();
                if let syn::PathArguments::AngleBracketed(ang_brck_gen_arg) = &gen_args{
                    if let Some( gen ) = ang_brck_gen_arg.args.first(){
                        if let syn::GenericArgument::Type(new_ty) = gen {
                            Ok(new_ty.clone())
                        } else { Err(msg_args) }

                    } else { Err(msg_args) }

                } else { Err(msg_args) }
                 
            } else { Err(msg_path) }

        }else { Err(msg_path) }

    } else { Err(msg_path) }

}




// impl InterGetter {



//     pub fn getter_decl(&self) -> TokenStream {
        
//         let Self{ name, ty };
//         let inter_get_name = format_ident!("inter_get_{name}");

//         quote!{ 
//             let  #name = self . inter_get_name ();
//         }
//     }

//     pub fn field_decl(&self) -> TokenStream {

//         let Self{ name, ty };

//         quote!{ name : ty }
        
//     }
// }




#[derive(Clone,Debug)]
pub struct InteractVariables {

    pub channel: Option<InterOneshot>,
    pub getters:     Vec<InterGetter>, 
    pub new_sig:            Signature,
    pub new_args:          Vec<FnArg>,
    pub ret:                     bool,
}

impl InteractVariables {

    pub fn from( new_sig: Signature, new_args: Vec<FnArg>, ret: bool ) -> Self {

        Self{
            channel: None,
            getters: Vec::new(),
            new_sig,
            new_args,
            ret,
        }
    }

    pub fn insert(&mut self,  vars: Vec<InterVariable>){

        for var in vars {

            match var{
                InterVariable::Oneshot(v) => {
                    if self.channel.is_none() {
                        self.channel = Some(v);
                    } else {
                        abort!(Span::call_site(),"Multiple use of `oneshot` channel, not allowed.");
                    }
                }
                InterVariable::Getter(v) => self.getters.push(v) ,
            }
        }
    }
}

pub fn get_variables( actor_type: &Type, sig: &Signature, args: &Vec<FnArg> ,ret: bool) 
    -> Option<InteractVariables> {
    
    let mut new_args = Vec::new() ;
    let mut inter_vars = Vec::new() ;

    for arg in args {
        if let FnArg::Typed(pat_ty) = arg { 
            // check if there is no conflictiong

            if let syn::Pat::Ident(pat_ident) = &*pat_ty.pat{

                match some_inter_var(&pat_ident.ident,&*pat_ty.ty,ret){

                    Ok(inter_var) => {
                        if let Some(inter_var) = inter_var {

                            inter_vars.push(inter_var);

                        } else { new_args.push(arg.clone())} 
    
                    },
                    Err(e) => {
                        let msg = format!("{}.{}", error::origin(actor_type, sig),e);
                        abort!(Span::call_site(),msg);
                    }
                }

            } 
        }  
    }
    
    let vars =
    inter_vars.iter().map(|x| x.get_name()).collect::<Vec<_>>();

    // check for colision
    check_send_recv(actor_type, sig,&new_args,Some(vars));

    // change the signature 
    let mut new_sig = sig.clone();
    let inputs = new_args.iter().cloned().collect::<Punctuated::<FnArg,Token![,]>>();
    new_sig.inputs = inputs;

    let mut vars = InteractVariables::from(new_sig,new_args,ret);
    vars.insert(inter_vars);

    Some(vars)
}



pub fn check_send_recv( actor_type: &Type, sig: &Signature, args: &Vec<FnArg>, vars: Option<Vec<Ident>> ){

    let mut words = vec![INTER_SEND.to_string(),INTER_RECV.to_string()];
    if let Some(vars) = vars {
        words.extend(vars.iter().map(|x|x.to_string()));
    } 

    for arg in args  { 

        if let FnArg::Typed(arg) = arg { 

            let pat_str = pat_to_str(&*arg.pat);

            for word in words.iter(){

                if pat_str.contains(&format!(" {word} ")){
                    // let actor     = quote!(#actor_type).to_string();
                    // let sig       = quote!(#sig).to_string();
                    let msg       = format!("{}. Conflicting name case `{}`!",error::origin(actor_type,sig),word);
                    abort!(Span::call_site(),msg);
                }
            }
        }
    }
}

pub fn pat_to_str( pat: &syn::Pat ) -> String {

    let mut s = quote!{ #pat }.to_string();
    let char_set = vec![ '(', ')', '{', '}', '[', ']', ':', '<', '>', ',' ];
    for c in char_set {
        s = s.replace(c,&format!(" {c} "))
    }
    s
}




