use crate::error::{self, OriginVars};

use proc_macro2::TokenStream;
use crate::model::{OneshotChannel,to_string_wide};
use syn::{Type,Ident,Signature,FnArg,Token, punctuated::Punctuated};
use quote::{quote,format_ident};

use proc_macro_error::abort;
use proc_macro2::Span;



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
    ty:           Type,
}


#[derive(Debug,Clone)]

pub enum InterVar {

    Oneshot(InterOneshot),
    Getter(InterGetter),
}




pub fn some_inter_var( org_err: &error::OriginVars, ident_ty:Vec<(&Ident, &Type)>, ret: bool ) -> Option<Vec<InterVar>> {

    let mut inter_vars = Vec::new();
    for (ident,ty) in ident_ty {

        let ident_str = ident.to_string();
    
        if let Some(second) = ident_str.strip_prefix("inter_"){
    
            if second.eq("send"){
                if ret { 
                    let msg =org_err.origin(error::NOT_ACCESSIBLE_CHANNEL_END);
                    abort!(Span::call_site(),msg;help=error::INTERACT_VARS_HELP);
                }
                // check if is sender and extract the Type it sends 
                match oneshot_get_type( ty, "Sender"){
                    Ok(new_ty) => { inter_vars.push(InterVar::Oneshot(InterOneshot::Send(new_ty.clone()))) },
                    Err(e)   => { abort!(Span::call_site(),org_err.origin(e);help=error::INTERACT_VARS_HELP); },
                }
            }
    
            else if second.eq("recv"){
    
                if ret { 
                    let msg =org_err.origin(error::NOT_ACCESSIBLE_CHANNEL_END);
                    abort!(Span::call_site(),msg;help=error::INTERACT_VARS_HELP);
                }
    
                match oneshot_get_type( ty, "Receiver"){
                    Ok(new_ty) => { inter_vars.push(InterVar::Oneshot(InterOneshot::Recv(new_ty.clone()))) },
                    Err(e)   => { abort!(Span::call_site(),org_err.origin(e);help=error::INTERACT_VARS_HELP); },
                }
            } 
            else { 
                let method_name = format_ident!("inter_get_{second}");
                inter_vars.push(InterVar::Getter(InterGetter{ name:ident.clone(), ty:ty.clone(), method_name })) 
            }
        } else { 
            let msg = org_err.origin("Unexpected usage of `inter variables` mixed identifiers.");
            abort!(Span::call_site(), msg; note=error::INTER_VARIABLE_PATTERN_NOTE);
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



#[derive(Clone)]
pub struct InterVars{

    pub channel:  Option<InterOneshot>,
    pub getters:      Vec<InterGetter>, 
    pub new_sig:             Signature,
    pub new_args:           Vec<FnArg>,
    pub one:    Option<OneshotChannel>,
    pub org_err:     error::OriginVars,
}

impl InterVars {
    pub fn some_ret_name(&self) -> Option<Ident> {
        self.channel.as_ref().map(|x| x.get_invers_name())
    }
    pub fn from( org_err: error::OriginVars, new_sig: Signature, 
                new_args: Vec<FnArg>, one: Option<OneshotChannel> ) -> Self {

        Self{
            channel: None,
            getters: Vec::new(),
            new_sig,
            new_args,
            one,
            org_err,
        }
    }

    pub fn insert(&mut self,  vars: Vec<InterVar>){
        for var in vars {
            match var {
                InterVar::Oneshot(ch) => {
                    if self.channel.is_none() { self.channel = Some(ch); }

                    else {
                        let msg = self.org_err.origin(error::CONCURRENT_INTER_SEND_RECV);
                        abort!(Span::call_site(),msg;help=error::INTERACT_VARS_HELP);
                    }
                },
                InterVar::Getter(gt) => { self.getters.push(gt); },
            }
        }
    }

    pub fn check (&self) {

        if let Some(var) = check_send_recv( &self.new_args, None){
        let msg = self.org_err.origin(format!("Unexpected pattern for `inter variable` - {}.",var));
            abort!(Span::call_site(),msg;note=error::INTER_VARIABLE_PATTERN_NOTE);
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

pub fn get_pat_idents( org_err:&OriginVars, pat_tuple: &syn::PatTuple ) -> Option<Vec<syn::Ident>> {

    if to_string_wide(pat_tuple).contains("inter_"){
        let length = pat_tuple.elems.len();
        let pat_idents = pat_tuple.elems.iter().filter_map(|p|  
            
            if let syn::Pat::Ident(pat_ident) = p {
                Some(pat_ident.ident.clone())
            } else {None} ).collect::<Vec<_>>();

        if length == pat_idents.len(){
            return Some(pat_idents);
        } else {
            let msg = org_err.origin("Unexpected usage of `inter variables` nested patterns.");
            abort!(Span::call_site(), msg; note=error::INTER_VARIABLE_PATTERN_NOTE);
        }
    }
    None
}


pub fn get_variables( org_err: &error::OriginVars, sig: & Signature, 
                         args: & Vec<FnArg>, one: Option<&OneshotChannel> ) 
    -> Option<InterVars> {
    let ret = one.is_none();
    let mut new_args = Vec::new() ;
    let mut inter_vars = Vec::new() ;

    for arg in args {
        if let FnArg::Typed(pat_ty) = arg { 

            if let syn::Pat::Ident(pat_ident) = &*pat_ty.pat {
                let ident = &pat_ident.ident;
                if ident.to_string().contains("inter_") {
                    let ty = &*pat_ty.ty;
                    if let Some(inter_var) = 
                        some_inter_var( &org_err,vec![(ident,ty)],ret){
                        inter_vars.extend(inter_var.into_iter());
                        continue;
                    } 
                }
            } 

            if let syn::Pat::Tuple(pat_tuple) = &*pat_ty.pat { 

                if let Some(idents) = get_pat_idents( org_err, &pat_tuple){

                    if let syn::Type::Tuple(ty_tuple) = &*pat_ty.ty {
                        let tys = ty_tuple.elems.iter();

                        if idents.len() == tys.len() {
                            let ident_ty = idents.iter().zip(tys).collect::<Vec<_>>();
                        
                            if let Some(inter_var) = some_inter_var( &org_err,ident_ty,ret){
                                inter_vars.extend(inter_var.into_iter());
                                continue;
                            }
                        }
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
    let mut new_sig = sig.clone();
    new_sig.inputs = new_args.iter().cloned().collect::<Punctuated::<FnArg,Token![,]>>();

    let mut vars = InterVars::from(org_err.clone(),new_sig,new_args,one.cloned());
    vars.insert(inter_vars);
    vars.check();

    Some(vars)
}


pub fn check_send_recv( args: &Vec<FnArg>, vars: Option<Vec<Ident>> ) -> Option<String> {

    let mut words = vec![crate::INTER_SEND.to_string(), crate::INTER_RECV.to_string()];
    
    if let Some(vars) = &vars {
        words = vars.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    } 

    for arg in args  { 
        if let FnArg::Typed(arg) = arg { 

            let pat_str = to_string_wide(&*arg);
            for word in words.iter(){

                if pat_str.contains(&format!(" {word} ")){
                    return Some(word.to_string());
                    /*
                    // the error can be more precises based on 
                    if let Some(v) = &vars {
                        if v.is_empty(){
                            // msg for Some([]) pattern declaration not allowed
                            let msg = "Some([])";
                            abort!(Span::call_site(),msg;note= error::INTER_SEND_RECV_RESTRICT_NOTE; help=error::INTERACT_VARS_HELP);
                        
                        } else {
                            // msg for Some([one]) only one end of the channel can be used and pattern delcaration not allowed
                            let msg = "Some([one])";
                            abort!(Span::call_site(),msg;note= error::INTER_SEND_RECV_RESTRICT_NOTE; help=error::INTERACT_VARS_HELP);
                        
                        }

                    } else {
                        // msg for None check there are not inter send recv
                        let msg = org_err.origin(format!("Conflicting name case. Please use a different name for the argument `{word}`."));
                        abort!(Span::call_site(),msg;note= error::INTER_SEND_RECV_RESTRICT_NOTE; help=error::INTERACT_VARS_HELP);
                    }
                     */
                }
            }
        }
    }
    None
}




