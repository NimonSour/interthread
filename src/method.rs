use crate::error::{self,met_new_found};
use crate::attribute::AALib;

use syn::{Visibility,Signature,Ident,FnArg,Type,ReturnType,ImplItem,ItemImpl,Receiver,Item,Token};
use proc_macro_error::abort;
use proc_macro2::{TokenStream,Span};
use quote::{quote,format_ident};

#[derive(Debug,Clone)]
pub enum ActorMethod {
    Io  { vis: Visibility, sig: Signature, ident: Ident, stat: bool,  arguments: Vec<FnArg>, output: Box<Type> },   
    I   { vis: Visibility, sig: Signature, ident: Ident,              arguments: Vec<FnArg>                    },    
    O   { vis: Visibility, sig: Signature, ident: Ident, stat: bool,                         output: Box<Type> },    
    None{ vis: Visibility, sig: Signature, ident: Ident                                                        }, 
}

impl ActorMethod {

    pub fn get_sig_and_field_name(&self) -> (Signature, Ident) {
        let (sig,name) = match self {

            Self::Io   { sig, ident, ..} => (sig.clone(),ident),
            Self::I    { sig, ident, ..} => (sig.clone(),ident),
            Self::O    { sig, ident, ..} => (sig.clone(),ident),
            Self::None { sig, ident, ..} => (sig.clone(),ident),
        };
        (sig, crate::name::script_field(name))
    }

    pub fn is_async(&self) -> bool {

        if let Some(_)= match self {

            Self::Io   { sig,..} => sig.asyncness,
            Self::I    { sig,..} => sig.asyncness,
            Self::O    { sig,..} => sig.asyncness,
            Self::None { sig,..} => sig.asyncness,
        }{
            return true;
        };
        false
    }
}
 

 
#[derive(Debug,Clone)]
pub struct ActorMethodNew {
    
    pub vis:               Visibility,
    pub sig:                Signature,
    pub new_sig:            Signature,
    pub res_opt:         Option<bool>,
    pub ident:                  Ident,
    pub arguments: Option<Vec<FnArg>>,
    pub output:             Box<Type>,      

}

impl ActorMethodNew {

    pub fn try_new( met: ActorMethod, new_sig: Signature,  res_opt: Option<bool> ) -> Option<Self>{
        
        match met {

            ActorMethod::Io   { vis,sig,ident,arguments,output,.. } =>  {
                return  Some(ActorMethodNew{ vis,sig,ident,arguments: Some(arguments), output, new_sig, res_opt });
            },
            ActorMethod::O    { vis,sig,ident,output,..} =>  {
                return  Some(ActorMethodNew{ vis,sig,ident,arguments: None, output, new_sig, res_opt });
            } 
            _   =>  return  None,
        };
    }

    pub fn get_arguments(&self)-> Vec<FnArg> {
        if let Some(arguments) = &self.arguments{
            return arguments.clone();
        }
        vec![]
    }

    pub fn  live_ret_statement(&self,  live_var: &Ident ) -> TokenStream {
       
        match self.res_opt {
            Some(true)  =>  quote!{ Ok ( #live_var )},
            Some(false) =>  quote!{ Some( #live_var )},
            None        =>  quote!{ #live_var },
        }
    }

    pub fn unwrap_sign(&self) -> TokenStream {
        if self.res_opt.is_none(){ quote!{}} else { quote!{?}}
    }
        
}

pub fn replace<T, O, N>(ty: &T, old: &O, new: &N) -> T
where
    T: syn::parse::Parse + quote::ToTokens,
    O: ToString + ?Sized,
    N: ToString + ?Sized,
{
    let mut type_str = quote! {#ty}.to_string();
    type_str = type_str.replace( ")"," )");
    type_str = type_str.replace( "(","( ");
    type_str = format!(" {type_str} ");
    let old = format!(" {} ", old.to_string());
    let str_type = type_str.replace(&old, &new.to_string());

    if let Ok(ty) = syn::parse_str::<T>(&str_type) {
        return ty;
    }

    let msg = format!("Internal Error. 'method::replace'. Could not parse &str to provided type!");
    abort!(Span::call_site(), msg);
}

pub fn is_trait(ty: &Type) -> bool {
    match ty {
        syn::Type::ImplTrait(_) => true,
        _ => false,
    }
}
pub fn get_new_sig( sig: &Signature, ty: &Type) -> Signature {

    let ty_name = if is_trait(ty)  { 
        quote!{#ty + Send + 'static}.to_string() 
    } else { quote!{#ty}.to_string() };
    let mut signature = replace(sig, "Self",&ty_name);
    signature.output = replace(&sig.output,&ty_name,"Self");
    signature
}


fn check_self_return( sig: &Signature, ty_name: &Type ) -> (Signature,Option<bool>) {

    let option_ident = format_ident!("Option");
    let result_ident = format_ident!("Result");
    let ty_self: Type       = syn::parse_quote!{ Self };

    match &sig.output {
        ReturnType::Type(_,ty_path) => {
            
            if  ty_self.eq(ty_path) {
                return (get_new_sig(sig,ty_name), None);
            } 

            else if ty_name.eq(ty_path) { 
                return (get_new_sig(sig,ty_name), None);
            }
            if !is_trait(ty_name){

                match ty_path.as_ref(){ 
                    Type::Path( p ) => {
                        let segment = &mut p.path.segments.last().unwrap();
                        let mut res_opt : Option<bool> = None;
                        
                        if  option_ident.eq(&segment.ident) {
                            res_opt = Some(false);
                        }
    
                        else if result_ident.eq(&segment.ident) {
                            res_opt = Some(true);
                        }
    
                        if res_opt.is_some(){
    
                            match &segment.arguments {
    
                                syn::PathArguments::AngleBracketed(gen_arg) => {
                                    if let Some(arg)  = gen_arg.args.first(){
    
                                        match arg {   
                                            syn::GenericArgument::Type( ty ) => {
                                                if ty_self.eq(ty){ 
                                                    return (get_new_sig(sig,ty_name), res_opt);
                                                }
    
                                                else if ty_name.eq(ty){
                                                    return (get_new_sig(sig,ty_name), res_opt);
                                                }
                                                else {
                                                    let (msg,note,help) = error::met_new_not_instance(sig, ty_name, quote!{#ty},res_opt);
                                                    abort!(ty,msg;note=note;help=help); 
                                                }
                                            },
                                            bit => {
                                                let (msg,note,help) = met_new_found(sig, ty_name, quote!{#segment},res_opt);
                                                abort!(bit,msg;note=note;help=help); 
                                            },
                                        }
                                    }
                                    let (msg,note,help) = met_new_found(sig, ty_name, quote!{#segment},res_opt);
                                    abort!(segment.arguments,msg;note=note;help=help); 
                                },
                                bit => {
                                    let (msg,note,help) = met_new_found(sig, ty_name, quote!{#segment},res_opt);
                                    abort!(bit,msg;note=note;help=help);
                                },
                            }
                        }
                        let (msg,note,help) = met_new_found(sig, ty_name, quote!{#p},None);
                        abort!(p,msg;note=note;help=help);
                    },
                    bit => {
                        let (msg,note,help) = met_new_found(sig, ty_name, quote!{#bit},None);
                        abort!(bit,msg;note=note;help=help);
                    },
                }
            } else {
                let ( msg, note ) = error::trait_new_sig(ty_name,true);
                abort!(sig,msg;note=note);
            }
        },
        
        bit => { 
            let (msg,note,help) = met_new_found(sig, ty_name, quote!{#bit},None);
            abort!(bit,msg;note=note;help=help);
        },
    }
}

fn is_return( sig: &Signature ) -> bool {
    match sig.output {
        ReturnType::Default => return false,
        ReturnType::Type(_, _) => return true,
    }
}

fn is_self_refer (signature: &Signature ) -> bool{
    if let Some(input) = signature.inputs.iter().next() {
        match input {
            FnArg::Receiver(receiver) => {
                let slf: syn::token::SelfValue = Default::default();
                if receiver.reference.is_some()  && (receiver.self_token == slf) {
                    return true;
                }
                return false
            },
            _ => return false,
        }
    }  
    false
}

pub fn explicit( sig: &Signature, ty_name: &syn::Type ) -> Signature{
    let ty = quote!{#ty_name};
    replace( sig, "Self", &ty )
}

fn is_vis( v: &Visibility ) -> bool {
    match v {
        Visibility::Public(_)|
        Visibility::Restricted(_) 
          => true,
        _ => false,
    }
}

fn get_sigs(item: &syn::Item) -> (Option<Visibility>, Vec<(Visibility,Signature)>){
    let mut res :(Option<Visibility>, Vec<(Visibility,Signature)>) = (None,Vec::new());

    match item {
        syn::Item::Fn(item_fn) => {
            if is_vis(&item_fn.vis){
                res.0 = Some(item_fn.vis.clone());
                for stmt in &item_fn.block.stmts {
                    match stmt {
                        syn::Stmt::Item(itm) => {
                            match itm {
                                syn::Item::Fn(itm_fn) => {
                                    // match visibility
                                    if is_vis(&itm_fn.vis){
                                        res.1.push((itm_fn.vis.clone(),itm_fn.sig.clone()));
                                    }
                                },
                                _ => (),
                            }
                        },
                        _ => (),
                    }
                }
            } else {
                let msg = "Expected explicit visibility.";
                let (note,help) = error::item_vis();
                abort!(item,msg;note=note;help=help)
            }
        },
        syn::Item::Trait(item_trait) => {
            if is_vis(&item_trait.vis){
                res.0 = Some(item_trait.vis.clone());
                for itm in &item_trait.items {
                    match itm {
                        syn::TraitItem::Fn(trait_item_fn) => {
                            res.1.push((item_trait.vis.clone(),trait_item_fn.sig.clone()));
                        },
                        _ => (),
                    }
                }
            } else {
                let msg = "Expected explicit visibility.";
                let (note,help) = error::item_vis();
                abort!(item,msg;note=note;help=help) 
            }
        },
        syn::Item::Impl(item_impl) => {
            for itm in &item_impl.items {
                match itm {
                    ImplItem::Fn( met ) => {
                        if is_vis(&met.vis) {
                            res.1.push((met.vis.clone(),met.sig.clone()));
                        }
                    },
                    _ => (),
                }
            }
        },
        v => {
            let msg = "Internal Error. 'methods::get_sigs' Expected Item  `Fn`, `Trait` or `Impl`.";
            abort!(v,msg);
        },
    }
    res
}

// needs an argument for static methods
pub fn get_methods( actor_type: &syn::Type, item: Item, stat:bool ) -> (Vec<ActorMethod>, Option<ActorMethodNew>){

    let mut loc              = vec![];
    let mut method_new = None;
    let ident_new                       = format_ident!("new");
    let ident_try_new                   = format_ident!("try_new");

    let (item_vis,sigs) = get_sigs(&item);
    // proc_macro_error::abort!(item, "After get_sigs");

    for (vis,sig) in sigs {

        if is_self_refer(&sig){
            loc.push(sieve(vis,explicit(&sig,actor_type),Some(false)));
        } else {
    
            // check if there is a function "new" or "try_new"
            if sig.ident.eq(&ident_new) || sig.ident.eq(&ident_try_new){
    
                let(new_sig,res_opt) = check_self_return(&mut sig.clone(),actor_type);
                let method = sieve(vis,sig.clone(),Some(true));
                method_new = ActorMethodNew::try_new( method, new_sig, res_opt ); 
            } 
    
            else {
                if stat {
                    if is_return(&sig){
                        loc.push(sieve(vis,explicit(&sig,actor_type),Some(true)));
                    }
                }
            }
        }
    }


    /*
    for i in item_impl.items {
        match i {
            ImplItem::Fn( m ) => {
                match m.vis {
                    // check visibility "pub"
                    Visibility::Public(_)|
                    Visibility::Restricted(_) 
                    => 
                    {

                        if is_self_refer(&m.sig){
                            loc.push(sieve(m.vis,explicit(&m.sig,actor_type),Some(false)));
                        } else {

                            // check if there is a function "new" or "try_new"
                            if m.sig.ident.eq(&ident_new) || m.sig.ident.eq(&ident_try_new){

                                let(new_sig,res_opt) = check_self_return(actor_type,&mut m.sig.clone());
                                let method = sieve(m.vis,m.sig.clone(),Some(true));
                                method_new = ActorMethodNew::try_new( method, new_sig, res_opt ); 
                            } 

                            else {
                                if stat {
                                    if is_return(&m.sig){
                                        loc.push(sieve(m.vis,explicit(&m.sig,actor_type),Some(true)));
                                    }
                                }
                            }
                        }
                    },
                    _ => (),
                } 
            },
            _ => (),
        }
    }
    */

    (loc, method_new)
}

pub fn sieve( vis: Visibility, sig: Signature, stat: Option<bool> ) -> ActorMethod {

    let stat = if stat.is_some(){ stat.unwrap() } else { is_self_refer(&sig) };
    let (ident,arguments,output) = ident_arguments_output(&sig);

    let arg_bool = { arguments.iter()
        .any( |a| match a { FnArg::Typed(_) => true, _ => false}) };


    match output.clone() {

        ReturnType::Type(_,output) => { 

            if arg_bool {
                return ActorMethod::Io{ vis, sig, stat, ident, arguments, output };
            } else {
                return ActorMethod::O{ vis, sig, stat, ident, output };
            }
        },
        ReturnType::Default => {

            if arg_bool {
                return ActorMethod::I{ vis, sig, ident, arguments };
            } else {
                return ActorMethod::None{ vis, sig, ident };
            }
        },
    }
}

pub fn ident_arguments_output( sig: &Signature  ) -> (Ident,Vec<FnArg>,ReturnType) {
    let punct_to_vec = 
    |p: syn::punctuated::Punctuated<FnArg,syn::token::Comma>| -> Vec<FnArg> { p.into_iter().collect::<Vec<_>>() };

    let ident          = sig.ident.clone();
    let arguments = punct_to_vec( sig.inputs.clone());
    let output    = sig.output.clone();

    (ident, arguments, output)
}
 
pub fn change_signature_refer( signature: &mut Signature ) {
    let recv: Receiver = syn::parse2(quote!{ &self }).unwrap();
    let slf = FnArg::Receiver(recv);
    signature.inputs.insert(0,slf);
}

pub fn args_to_ident_type(args: &Vec<FnArg>) -> (Vec<Ident>, Vec<Box<Type>>){

    let mut idents = Vec::new();
    let mut types  = Vec::new();

    for i in args  { 
        match i { 
            FnArg::Typed(arg) => { 
                if let Some(id) = match *arg.pat.clone() {
                    syn::Pat::Ident(pat_id) => Some(pat_id.ident.clone()),
                    _ => None,
                }{
                    idents.push(id);
                    types.push(arg.ty.clone());
                }
            },
            _ => (),
        }
    }
    (idents,types)    
}

pub fn arguments_ident_type( args: &Vec<FnArg> ) -> (TokenStream, TokenStream) { 

    let (idents,types) = args_to_ident_type(args); 
    let args_ident =  quote!{ (#(#idents),*)};
    let args_type  =  quote!{ (#(#types),*) };
    ( args_ident, args_type )
}

pub fn to_async( lib: &AALib, sig: &mut Signature ) {
    
    match lib {
        AALib::Std => (),
        _ => {
            sig.asyncness = Some(Token![async](Span::call_site()));
        }
    }
}


