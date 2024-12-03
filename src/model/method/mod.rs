mod actor_method;
mod vars;
mod cont;

pub use vars::{ConstVars,ImplVars};
pub use actor_method::*;
pub use cont::Cont;

use crate::{error::{self,met_new_found},model::{name,ShowComment}};
use syn::{parse_quote,Pat,PatType,PatIdent,Path,punctuated::Punctuated,GenericArgument,
          GenericParam,Signature,Ident,FnArg,Type,TypePath,ReturnType,Token};

use proc_macro_error::{abort, abort_call_site};
use proc_macro2::TokenStream;
use quote::{quote,format_ident};
use std::collections::HashMap;


#[derive(Clone)]
pub struct ModelPhantomData {

    pub phantom_fields: TokenStream,
    pub phantom_invoks: TokenStream,
    pub phantom_init_vars: TokenStream, 

}

impl ModelPhantomData {

    pub fn from( gen_set: &HashMap<GenericArgument,GenericParam> ) -> Self {

        let mut slf = Self::default();
        if !gen_set.is_empty(){ 

            let params =                 
                gen_set
                    .values()
                    .map(|p| crate::model::gen_params::as_arg(p))
                    .collect::<Vec<_>>();
    
            let mut pats = vec![];
            let mut invoks = vec![];
            let mut vars = vec![];
            
            for (index,arg) in params.iter().enumerate() {
                let ident = format_ident!("_{index}");
                pats.push( quote!{ #ident : ::std::marker::PhantomData<#arg> } );
                invoks.push( quote!{ let #ident = ::std::marker::PhantomData } );
                vars.push( ident );
            }
    
            slf.phantom_fields = quote!{ #(#pats),*,};
            slf.phantom_invoks = quote!{ #(#invoks);*;};
            slf.phantom_init_vars = quote!{ #(#vars),*,}; 
        }
        slf
    }
}

impl Default for ModelPhantomData {
    fn default() -> Self {
        Self{
            phantom_fields: quote!{},
            phantom_invoks: quote!{},
            phantom_init_vars: quote!{},
        }
    }
}




pub fn get_new_sig( sig: &Signature, actor_ty: &Type) -> Signature {
    let actor_turbo_ty = super::turbofish::from_type(actor_ty);
    super::replace(sig, &quote!{Self::},&actor_turbo_ty);

    let ty_self: Type   = parse_quote!{ Self };
    let mut signature = super::replace(sig, &ty_self,actor_ty);
    signature.output = super::replace(&sig.output,actor_ty,&ty_self);
    signature
}

pub fn clean_path( path : &Path ) -> Path {
    let mut path = path.clone();
    if let Some(segment) = path.segments.last_mut(){
        segment.arguments = syn::PathArguments::None;
    }
    path
}


fn check_self_return( sig: &Signature, actor_ty: &TypePath ) -> (Signature,ModelOutput) {

    let option_ident = format_ident!("Option");
    let result_ident = format_ident!("Result");
    let ty_self: Type       = parse_quote!{ Self };
    let actor_ty      = &Type::Path(actor_ty.clone());
    let mut model_output    = ModelOutput::None;
    match &sig.output {
        ReturnType::Type(_,ty_path) => {
            
            if  ty_self.eq(ty_path)     { return (get_new_sig(sig,actor_ty), model_output);} 
            else if actor_ty.eq(ty_path) { return (get_new_sig(sig,actor_ty), model_output);}

            match ty_path.as_ref(){ 
                Type::Path( p ) => {
                    let segment = p.path.segments.last().unwrap();

                    if  option_ident.eq(&segment.ident) {
                        model_output = ModelOutput::Option(clean_path(&p.path));
                    }

                    else if result_ident.eq(&segment.ident) {
                        model_output = ModelOutput::Result(clean_path(&p.path),None);
                    }

                    if model_output.is_some(){

                        match &segment.arguments {

                            syn::PathArguments::AngleBracketed(gen_arg) => {
                                if let Some(arg)  = gen_arg.args.first(){

                                    match arg {   
                                        syn::GenericArgument::Type( ty ) => {
                                            if ty_self.eq(ty){ 
                                                return (get_new_sig(sig,actor_ty), model_output);
                                            }

                                            else if actor_ty.eq(ty){
                                                return (get_new_sig(sig,actor_ty), model_output);
                                            }
                                            else {
                                                let (msg,note,help) = error::met_new_not_instance(sig, actor_ty, quote!{#ty},model_output);
                                                abort!(ty,msg;note=note;help=help); 
                                            }
                                        },
                                        bit => {
                                            let (msg,note,help) = met_new_found(sig, actor_ty, quote!{#segment},model_output);
                                            abort!(bit,msg;note=note;help=help); 
                                        },
                                    }
                                }
                                let (msg,note,help) = met_new_found(sig, actor_ty, quote!{#segment},model_output);
                                abort!(segment.arguments,msg;note=note;help=help); 
                            },
                            bit => {
                                let (msg,note,help) = met_new_found(sig, actor_ty, quote!{#segment},model_output);
                                abort!(bit,msg;note=note;help=help);
                            },
                        }
                    }
                    let (msg,note,help) = met_new_found(sig, actor_ty, quote!{#p},model_output);
                    abort!(p,msg;note=note;help=help);
                },
                bit => {
                    let (msg,note,help) = met_new_found(sig, actor_ty, quote!{#bit},model_output);
                    abort!(bit,msg;note=note;help=help);
                },
            }
        },
        
        bit => { 
            let (msg,note,help) = met_new_found(sig, actor_ty, quote!{#bit},model_output);
            abort!(bit,msg;note=note;help=help);
        },
    }
}

pub fn ident_arguments_output( sig: &Signature  ) -> (Ident,Vec<FnArg>,ReturnType) {

    let ident          = sig.ident.clone();
    let arguments = sig.inputs.clone().into_iter().collect::<Vec<_>>();
    let output    = sig.output.clone();

    (ident, arguments, output)
}
 
pub fn args_to_pat_type(args: &Vec<FnArg>) -> (Vec<Box<syn::Pat>>, Vec<Box<Type>>) 
{
    
    let mut pats = Vec::new();
    let mut types  = Vec::new();

    for arg in args  { 

        match arg { 
            FnArg::Typed(pat_ty) => { 
                pats.push(pat_ty.pat.clone());
                types.push(pat_ty.ty.clone());
            },
            _ => (),
        }
    }
    (pats,types)    
}
/// clear args from 'ref' and 'mut'
/// 
/// true if contains arguments
pub fn if_args_and_clean_pats( sig: &mut Signature ) -> bool {

    let mut arg_flag = false;

    for arg in sig.inputs.iter_mut(){
        match arg { 
            FnArg::Typed(pat_ty) => { 
                arg_flag = true;
                clear_ref_mut(&mut *pat_ty.pat);
            },
            _ => (),
        }
    }
    arg_flag
}


pub fn clear_ref_mut(pat: &mut Pat ){

    match pat {
        Pat::Ident(pat_ident) => { 
            pat_ident.by_ref     = None;
            pat_ident.mutability = None;
        },
        Pat::TupleStruct(pat_tuple_struct) => {
            let _ = pat_tuple_struct.elems.iter_mut().map(|p|{
                clear_ref_mut(p)
            });
        },
        Pat::Tuple(pat_tuple) => {
            let _ = pat_tuple.elems.iter_mut().map(|p|{
                clear_ref_mut(p)
            });
        },
        Pat::Struct(pat_struct) => {
            let _ = pat_struct.fields.iter_mut().map(|f| clear_ref_mut(&mut *f.pat));
        },
        Pat::Rest(_) => (),

        Pat::Slice(pat_slice) => {
            let _ = pat_slice.elems.iter_mut().map(|p|{
                clear_ref_mut(p)
            });
        },

        _ => {
            let msg = "Unexpected pattern for function argument.";
            abort!(pat,msg;note=error::PARAMETERS_ALLOWED_PATTERN_NOTE); 
        },
    }

}


pub fn pat_vars_flat_into_ident( pat: &Pat ) -> Option<Ident> {

    let mut loc = vec![];
    
    match pat {
        Pat::Ident( pat_ident) =>  { 
            return Some(pat_ident.ident.clone());
        },
        Pat::TupleStruct( pat_tuple_struct) => {
            let vars = pat_tuple_struct.elems.iter().filter_map(|p|{
                pat_vars_flat_into_ident(p)
            });
            loc.extend(vars);
        },
        Pat::Tuple( pat_tuple) =>{
            let vars = pat_tuple.elems.iter().filter_map(|p|{
                pat_vars_flat_into_ident(p)
            });
            loc.extend(vars);
        },
        Pat::Struct( pat_struct) => {
            let vars= pat_struct.fields.iter().filter_map(|f| pat_vars_flat_into_ident(&f.pat));
            loc.extend(vars);
        },
        Pat::Slice( pat_slice) => {
            let vars = pat_slice.elems.iter().filter_map(|p|{
                pat_vars_flat_into_ident(p)
            });
            loc.extend(vars);
        },
        Pat::Rest(_) => return None,
        _ => {
            let msg = "Internal Error.'method::pat_vars_flat_into_ident'. Unexpected pattern for function argument.";
            abort_call_site!(msg;note=error::PARAMETERS_ALLOWED_PATTERN_NOTE); 
        },
    }

    if loc.len() == 0 {
        // the compiler allows an empty (Rest) pattern like  (..) : Tuple
        // we will use '__' identifier to be able to carry the type within the model
        // if the method contains two or more such parameters
        // it will trigger a naming conflict error  
        loc.push(format_ident!("__"));
    }

    return Some(name::combined_ident(loc));
}


fn flat_pat_fn_arg( pat_ty: &PatType ) -> PatType{

    let mut pat_ty = pat_ty.clone();

    let ident = pat_vars_flat_into_ident(&*pat_ty.pat).unwrap();
    let pat = Pat::Ident(
        PatIdent{ 
            attrs: vec![],
            by_ref: None,
            mutability: None,
            ident,
            subpat: None,
        });
    *pat_ty.pat = pat;

    pat_ty
}

fn flat_arguments( args: &Vec<FnArg> ) -> Vec<FnArg> {
    let mut loc = vec![];

    for arg in args {
        match arg { 
            FnArg::Typed(pat_ty) => { 
                let new_arg = flat_pat_fn_arg(pat_ty);
                loc.push(FnArg::Typed(new_arg));
            },
            _ => { loc.push( arg.clone());},
        }
    }
    loc
}


pub fn get_live_args_and_sig( sig: &Signature ) -> (Vec<FnArg>, Signature) {
    let( _,args,_) = &ident_arguments_output(sig);
    let new_args = flat_arguments(args);
    let mut sig = sig.clone();

    sig.inputs = new_args.iter().cloned().collect::<Punctuated::<FnArg,Token![,]>>();

    let args = 
    new_args.into_iter().filter(|x|  
        match x {
            FnArg::Typed(_)    => true,
            FnArg::Receiver(_) => false, 
        }).collect();
    (args, sig)
}





