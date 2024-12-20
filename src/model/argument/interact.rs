use crate::error;

use proc_macro2::TokenStream;
use crate::model::{OneshotChannel,to_string_wide};
use syn::{punctuated::Punctuated,Token,Pat,Type,Ident,Signature,FnArg};
use quote::{quote,format_ident};

use proc_macro_error::abort;




#[derive(Debug,Clone)]
pub enum InterOneshot {

    Recv(Type),
    Send(Type),
}

impl InterOneshot {
    pub fn get_type(&self) -> &Type {
        match &self {
            Self::Recv(ty) => ty, 
            Self::Send(ty) => ty, 
        }
    }

    pub fn get_invers_name(&self) -> Ident{
        match &self {
            Self::Recv(_) => format_ident!("{}",crate::INTER_SEND),
            Self::Send(_) => format_ident!("{}",crate::INTER_RECV),  
        }
    }

    pub fn get_return_type(&self, one: &OneshotChannel) -> syn::ReturnType {

        let token_type = match &self {
            Self::Recv(_) => one.send_type(self.get_type()),
            Self::Send(_) => one.recv_type(self.get_type()),  
        };
        syn::parse_quote!{ -> #token_type}
    }   
}


#[derive(Debug,Clone)]
pub struct InterGetter { 
    name:        Ident, 
    method_name: Ident,
    _ty:          Type,
}


#[derive(Debug,Clone)]
pub enum InterVar {

    Oneshot(InterOneshot),
    Getter(InterGetter),
}




pub fn some_inter_var( ident_ty:Vec<(&Ident, &Type)>, ret: bool ) -> Option<Vec<InterVar>> {

    let mut inter_vars = Vec::new();
    for (ident,ty) in ident_ty {

        let ident_str = ident.to_string();
    
        if let Some(second) = ident_str.strip_prefix("inter_"){
    
            if second.eq("send"){
                if ret { 
                    abort!(ident,error::NOT_ACCESSIBLE_CHANNEL_END;help=error::INTERACT_VARS_HELP);
                }
                // check if is sender and extract the Type it sends 
                match oneshot_get_type( ty, "Sender"){
                    Ok(new_ty) => { inter_vars.push(InterVar::Oneshot(InterOneshot::Send(new_ty.clone()))) },
                    Err(e)   => { abort!(ty,e;help=error::INTERACT_VARS_HELP); },
                }
            }
    
            else if second.eq("recv"){
    
                if ret { 
                    abort!(ident,error::NOT_ACCESSIBLE_CHANNEL_END;help=error::INTERACT_VARS_HELP);
                }
    
                match oneshot_get_type( ty, "Receiver"){
                    Ok(new_ty) => { inter_vars.push(InterVar::Oneshot(InterOneshot::Recv(new_ty.clone()))) },
                    Err(e)   => { abort!(ty,e;help=error::INTERACT_VARS_HELP); },
                }
            } 
            else { 
                let method_name = format_ident!("inter_get_{second}");
                inter_vars.push(InterVar::Getter(InterGetter{ name:ident.clone(), _ty:ty.clone(), method_name })) 
            }
        } else { 
            let msg = "Unexpected usage of `inter variables` mixed identifiers.";
            abort!(ident, msg; note=error::INTER_VARIABLE_SUPPORTED_PATTERN_NOTE);
        }
    }
    Some(inter_vars)

    
}


pub fn oneshot_get_type( ty: &Type, target: &str ) -> Result<Type,String> {

    let msg_path = format!("Expected a path type, .. ::{}<Type> .", target);
    let msg_args = format!("Unexpected type argument for {}<?>.",target);

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



#[derive(Debug,Clone)]
pub struct InterVars{

    pub channel:  Option<InterOneshot>,
    pub getters:      Vec<InterGetter>, 
    pub new_sig:             Signature,
    pub new_args:           Vec<FnArg>,
    pub one:    Option<OneshotChannel>,
}

impl InterVars {
    pub fn from( new_sig: Signature, 
                new_args: Vec<FnArg>, 
                     one: Option<OneshotChannel> ) -> Self {

        Self{
            channel: None,
            getters: Vec::new(),
            new_sig,
            new_args,
            one,
        }
    }

    pub fn some_ret_name(&self) -> Option<TokenStream> {
        if let Some(inter_one) = self.channel.as_ref(){
            let channel_ident = inter_one.get_invers_name();
            return Some(quote!{#channel_ident});
        }
        None
    }

    pub fn insert(&mut self,  vars: Vec<InterVar>){
        for var in vars {
            match var {
                InterVar::Oneshot(ch) => {
                    if self.channel.is_none() { self.channel = Some(ch); }

                    else {

                        abort!(ch.get_type(),error::CONCURRENT_INTER_SEND_RECV;help=error::INTERACT_VARS_HELP);
                    }
                },
                InterVar::Getter(gt) => { self.getters.push(gt); },
            }
        }
    }

    pub fn check (&self) {

        if let Some((var, pat)) = check_send_recv( &self.new_args, None){
            let msg = format!("Unexpected. `inter variable` - {var}, within function parameter pattern. ");
            abort!(pat,msg;note=error::INTER_VARIABLE_SUPPORTED_PATTERN_NOTE);
        }
    }


    pub fn get_getters_decl(&mut self) -> TokenStream {

        let mut loc = Vec::new();
        for get in &self.getters {

            let InterGetter{ name, method_name,..} = get;
            loc.push(quote!{
                let #name = self. #method_name ();
            });
        }
        
        if let Some(ch_one) = &self.one {
            if let Some(channel) = &self.channel {
                let declare = ch_one.decl(Some(&channel.get_type()));
                loc.push( quote!{ #declare });
                self.new_sig.output =  channel.get_return_type(ch_one);
            }
        }
        quote!{#(#loc)*}
    }

}



pub fn get_variables( sig: &Signature, one: Option<&OneshotChannel> ) -> Option<InterVars> {
    let ret = one.is_none();
    let mut new_args = Vec::new() ;
    let mut inter_vars = Vec::new() ;

    for arg in &sig.inputs {
        if let FnArg::Typed(pat_ty) = arg { 

            if let syn::Pat::Ident(pat_ident) = &*pat_ty.pat {
                let ident = &pat_ident.ident;
                if ident.to_string().contains("inter_") {
                    let ty = &*pat_ty.ty;
                    if let Some(inter_var) = 
                        some_inter_var(vec![(ident,ty)],ret){
                        inter_vars.extend(inter_var.into_iter());
                        continue;
                    } 
                }
            } 
        } 

        new_args.push(arg.clone()); 
    }

    if inter_vars.is_empty(){
        return None;
    }

    // change signature input
    let mut sig = sig.clone();
    sig.inputs = new_args.into_iter().collect::<Punctuated::<FnArg,Token![,]>>();
    let (live_arguments,live_sig) = crate::model::method::get_live_args_and_sig(&sig );
    
    let mut vars = InterVars::from(live_sig,live_arguments,one.cloned());
    
    vars.insert(inter_vars);
    vars.check();
    // just to change the ouput
    // of method if required
    let _ = vars.get_getters_decl();

    Some(vars)
}


pub fn check_send_recv<'a,I>( args: I, vars: Option<Vec<Ident>> ) -> Option<(String, Pat)> 
where I: IntoIterator<Item = &'a FnArg>,
{

    let mut words = vec![crate::INTER_SEND.to_string(), crate::INTER_RECV.to_string()];
    
    if let Some(vars) = &vars {
        words = vars.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    } 

    for arg in args  { 
        if let FnArg::Typed(arg) = arg { 

            let pat_str = to_string_wide(&*arg);
            for word in words.iter(){

                if pat_str.contains(&format!(" {word} ")){
                    return Some((word.to_string(),*arg.pat.clone()));
                }
            }
        }
    }
    None
}




