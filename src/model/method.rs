use crate::error::{self,met_new_found, OriginVars};
use crate::model::{
    ActorAttributeArguments, OneshotChannel, MpscChannel,
    name,Cont,Vars,Lib,ImplVars,InterVars};

use syn::punctuated::Punctuated;
use syn::{Attribute,Path,Visibility,Signature,Ident,FnArg,Type,ReturnType,ImplItem,ItemImpl,Token};
use proc_macro_error::abort;
use proc_macro2::{TokenStream,Span};
use quote::{quote,format_ident};

use super::FilterSet;

#[derive(Debug,Clone)]
pub enum ActorMethod {
    Io  { doc_attrs: Vec<Attribute>, vis: Visibility, sig: Signature, ident: Ident, org_err: OriginVars, stat: bool, arguments: Vec<FnArg>, output: Box<Type> },   
    I   { doc_attrs: Vec<Attribute>, vis: Visibility, sig: Signature, ident: Ident, org_err: OriginVars, stat: bool, arguments: Vec<FnArg>,                   },    
    O   { doc_attrs: Vec<Attribute>, vis: Visibility, sig: Signature, ident: Ident, org_err: OriginVars, stat: bool,                        output: Box<Type> },    
    None{ doc_attrs: Vec<Attribute>, vis: Visibility, sig: Signature, ident: Ident, org_err: OriginVars, stat: bool,                                          }, 
}

impl ActorMethod {

    pub fn reset(self) -> Self {

        let (doc_attrs,vis,org_err,sig,stat) = 
        match self {
            Self::Io  {doc_attrs, vis,org_err,sig,stat,..} => (doc_attrs, vis, org_err, sig, stat),  
            Self::I   {doc_attrs, vis,org_err,sig,stat,..} => (doc_attrs, vis, org_err, sig, stat),  
            Self::O   {doc_attrs, vis,org_err,sig,stat,..} => (doc_attrs, vis, org_err, sig, stat),  
            Self::None{doc_attrs, vis,org_err,sig,stat,..} => (doc_attrs, vis, org_err, sig, stat),  
        };
        sieve(doc_attrs,vis,org_err, sig, stat)
    }

    pub fn get_mut_sig(&mut self) -> &mut Signature {
        match self {
            ActorMethod::Io   { sig,..} => sig,
            ActorMethod::I    { sig,..} => sig, 
            ActorMethod::O    { sig,..} => sig, 
            ActorMethod::None { sig,..} => sig,
        }
    }

    pub fn get_sig_and_field_name(&self) -> (Signature, Ident) {
        let (sig,name) = match self {

            Self::Io   { sig, ident, ..} => (sig.clone(),ident),
            Self::I    { sig, ident, ..} => (sig.clone(),ident),
            Self::O    { sig, ident, ..} => (sig.clone(),ident),
            Self::None { sig, ident, ..} => (sig.clone(),ident),
        };
        (sig, name::script_field(name))
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

    pub fn is_static(&self) -> bool {

        match self {

            Self::Io   { stat,..} => *stat,
            Self::I    { stat,..} => *stat,
            Self::O    { stat,..} => *stat,
            Self::None { stat,..} => *stat,
        }
    }


}
 

 
#[derive(Debug,Clone)]
pub struct ActorMethodNew {
    
    pub doc_attrs:     Vec<Attribute>,
    pub vis:               Visibility,
    pub sig:                Signature,
    pub res_opt:         Option<bool>,
    pub res_opt_path:    Option<Path>,
    pub ident:                  Ident,
    pub arguments:         Vec<FnArg>,
    pub output:             Box<Type>,
          

}

impl ActorMethodNew {

    pub fn get_mut_sig(&mut self) -> &mut Signature {
        &mut self.sig
    }

    pub fn try_new( met: ActorMethod,  res_opt: Option<bool>, res_opt_path: Option<Path> ) -> Option<Self>{
        
        match met {

            ActorMethod::Io   { doc_attrs,vis,sig,ident,arguments,output,.. } =>  {
                let (arguments,sig) = live_args_and_sig( &arguments, &sig );
                return  Some(ActorMethodNew{ doc_attrs,vis,sig,ident,arguments: arguments, output, res_opt,res_opt_path });
            },
            ActorMethod::O    { doc_attrs,vis,sig,ident,output,..} =>  {
                return  Some(ActorMethodNew{ doc_attrs,vis,sig,ident,arguments: vec![], output, res_opt,res_opt_path });
            } 
            _   =>  return  None,
        };
    }

    pub fn  live_ret_statement(&self,  init_live: &TokenStream ) -> TokenStream {
        let ActorMethodNew{res_opt,res_opt_path,..} = &self;
        match res_opt {
            Some(true)  =>  quote!{ #res_opt_path :: Ok ( #init_live )},
            Some(false) =>  quote!{ #res_opt_path :: Some( #init_live )},
            None        =>  quote!{ #init_live },
        }
    }

    pub fn unwrap_sign(&self) -> TokenStream {
        if self.res_opt.is_none(){ quote!{}} else { quote!{?}}
    }
        
}

pub fn to_string_wide<T>(ty: &T) -> String 
where T: quote::ToTokens,
{
    let mut type_str = quote! {#ty}.to_string();
    type_str = type_str.replace( "}"," } ");
    type_str = type_str.replace( "{"," { ");
    type_str = type_str.replace( "]"," ] ");
    type_str = type_str.replace( "["," [ ");
    type_str = type_str.replace( ")"," ) ");
    type_str = type_str.replace( "("," ( ");
    type_str = type_str.replace( ","," , ");
    type_str = format!(" {type_str} ");
    type_str.to_string()
}

pub fn replace<T, O, N>(ty: &T, old: &O, new: &N) -> T
where
    T: syn::parse::Parse + quote::ToTokens,
    O: ToString + ?Sized,
    N: ToString + ?Sized,
{
    let type_str = to_string_wide(&ty);
    let old  = format!(" {} ", old.to_string());
    let new  = format!(" {} ",new.to_string());
    let str_type = type_str.replace(&old, &new);

    if let Ok(ty) = syn::parse_str::<T>(&str_type) {
        return ty;
    }
    let msg = format!("Internal Error. 'method::replace'. Could not parse &str to provided type! str_type - '{}'",str_type);
    abort!(Span::call_site(), msg);
}

pub fn get_new_sig( sig: &Signature, ty: &Type) -> Signature {

    let ty_name = quote!{#ty}.to_string() ;
    let mut signature = replace(sig, "Self",&ty_name);
    signature.output = replace(&sig.output,&ty_name,"Self");
    signature
}

pub fn clean_path( path : &Path ) -> Path {
    let mut path = path.clone();
    if let Some(segment) = path.segments.last_mut(){
        segment.arguments = syn::PathArguments::None;
    }
    path
}

fn check_self_return( sig: &Signature, ty_name: &Type ) -> (Signature,Option<bool>,Option<Path>) {

    let option_ident = format_ident!("Option");
    let result_ident = format_ident!("Result");
    let ty_self: Type       = syn::parse_quote!{ Self };

    match &sig.output {
        ReturnType::Type(_,ty_path) => {
            
            if  ty_self.eq(ty_path) {
                return (get_new_sig(sig,ty_name), None, None);
            } 

            else if ty_name.eq(ty_path) { 
                return (get_new_sig(sig,ty_name), None, None);
            }

            match ty_path.as_ref(){ 
                Type::Path( p ) => {
                    let segment = p.path.segments.last().unwrap();
                    let mut res_opt : Option<bool>      = None;
                    let mut res_opt_path : Option<Path> = None;
                    
                    if  option_ident.eq(&segment.ident) {
                        res_opt = Some(false);
                        res_opt_path = Some(clean_path(&p.path));
                    }

                    else if result_ident.eq(&segment.ident) {
                        res_opt = Some(true);
                        res_opt_path = Some(clean_path(&p.path));
                    }

                    if res_opt.is_some(){

                        match &segment.arguments {

                            syn::PathArguments::AngleBracketed(gen_arg) => {
                                if let Some(arg)  = gen_arg.args.first(){

                                    match arg {   
                                        syn::GenericArgument::Type( ty ) => {
                                            if ty_self.eq(ty){ 
                                                return (get_new_sig(sig,ty_name), res_opt, res_opt_path);
                                            }

                                            else if ty_name.eq(ty){
                                                return (get_new_sig(sig,ty_name), res_opt, res_opt_path);
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
        },
        
        bit => { 
            let (msg,note,help) = met_new_found(sig, ty_name, quote!{#bit},None);
            abort!(bit,msg;note=note;help=help);
        },
    }
}

/// Some(true)  - &self 
/// 
/// Some(false) - static 
/// 
/// None        - self
fn is_self_refer ( signature: &Signature ) -> Option<bool>{
    if let Some(input) = signature.inputs.iter().next() {
        match input {
            FnArg::Receiver(receiver) => {
                let slf: syn::token::SelfValue = Default::default();
                if receiver.self_token == slf {
                    if receiver.reference.is_some() {
                        return Some(true);
                    }
                } 
                return None;
            },
            _ => return Some(false),
        }
    }  
    Some(false)
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


fn get_sigs(item_impl: &syn::ItemImpl) -> Vec<(Vec<syn::Attribute>,Visibility,Signature)>{
    let mut res  = Vec::new();

    for itm in &item_impl.items {
        match itm {
            ImplItem::Fn( met ) => {
                if is_vis(&met.vis) {
                    let doc_attrs = 
                        met.attrs
                            .iter()
                            .filter(|&x|  x.path().is_ident("doc"))
                            .map(|x|x.clone())
                            .collect::<Vec<_>>();
                    res.push((doc_attrs, met.vis.clone(), met.sig.clone()));
                }
            },
            _ => (),
        }
    }
    res
}


pub fn get_methods( actor_type: &syn::Type, item_impl: ItemImpl, aaa: & ActorAttributeArguments, mut act: bool) -> (Vec<ActorMethod>, Option<ActorMethodNew>){

    let mut loc              = vec![];
    let mut method_new = None;
    let ident_new                       = format_ident!("new");
    let ident_try_new                   = format_ident!("try_new");
    let ActorAttributeArguments{ path,filter,..} = aaa;

    let mut filter = if let Some( filter) = filter { filter.clone() } else { FilterSet::Exclude(vec![])};

    for (doc_attrs, vis,sig) in get_sigs(&item_impl) {

        let org_err = OriginVars{ path: path.clone(), actor_type: actor_type.clone(), sig: sig.clone()};

        if let Some(b) = is_self_refer(&sig) {
            // fiter methods 
            if !filter.condition(&sig){ continue; }

            if b {

                loc.push(sieve( doc_attrs,vis,org_err,explicit(&sig,actor_type),false));
            } else {
                if act {
                    // check if there is a function "new" or "try_new"
                    if sig.ident.eq(&ident_new) || sig.ident.eq(&ident_try_new){
                    
                        let(new_sig,res_opt,res_opt_path) = check_self_return(&mut sig.clone(),actor_type);
                        let method = sieve(doc_attrs,vis,org_err,new_sig,true );
                        method_new = ActorMethodNew::try_new( method, res_opt, res_opt_path );
                        act = false;
                        continue; 
                    } 
                } else {
                    loc.push(sieve( doc_attrs,vis,org_err,explicit(&sig,actor_type),true));
                }
            }
        }
    }
    // checkif all names have been found
    filter.check();

    (loc, method_new)
}

pub fn sieve( doc_attrs: Vec<Attribute>, vis: Visibility, org_err: OriginVars, mut sig: Signature, stat: bool ) -> ActorMethod {


    let arg_bool = if_args_sig_clean_pats(&org_err, &mut sig);
    let (ident,arguments,output) = ident_arguments_output(&sig);

    match output.clone() {
        ReturnType::Type(_,output) => { 

            if arg_bool {
                return ActorMethod::Io{ doc_attrs, vis, org_err, sig, stat, ident, arguments, output };
            } else {
                return ActorMethod::O{ doc_attrs, vis, org_err, sig, stat, ident, output };
            }
        },
        ReturnType::Default => {

            if arg_bool {
                return ActorMethod::I{ doc_attrs,vis, org_err, sig, stat, ident, arguments };
            } else {
                return ActorMethod::None{ doc_attrs, vis, org_err, sig, stat, ident };
            }
        },
    }
}

pub fn ident_arguments_output( sig: &Signature  ) -> (Ident,Vec<FnArg>,ReturnType) {

    let ident          = sig.ident.clone();
    let arguments = sig.inputs.clone().into_iter().collect::<Vec<_>>();
    let output    = sig.output.clone();

    (ident, arguments, output)
}
 
pub fn args_to_pat_type(args: &Vec<FnArg>) -> (Vec<Box<syn::Pat>>, Vec<Box<Type>>) {
    
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

pub fn if_args_sig_clean_pats( org_err: &OriginVars, sig: &mut Signature ) -> bool {

    let mut arg_bool = false;

    for arg in sig.inputs.iter_mut(){
        match arg { 
            FnArg::Typed(pat_ty) => { 
                arg_bool = true;
                clear_ref_mut(org_err, &mut *pat_ty.pat);
            },
            _ => (),
        }
    }
    arg_bool
}

pub fn clear_ref_mut(org_err: &OriginVars, pat: &mut syn::Pat ){

    
    match *pat {
        syn::Pat::Ident(ref mut pat_ident) => { 
            pat_ident.by_ref     = None;
            pat_ident.mutability = None;
        },
        syn::Pat::TupleStruct(ref mut pat_tuple_struct) => {
            let _ = pat_tuple_struct.elems.iter_mut().map(|p|{
                clear_ref_mut(org_err,p)
            });
        },
        syn::Pat::Tuple(ref mut pat_tuple) => {
            let _ = pat_tuple.elems.iter_mut().map(|p|{
                clear_ref_mut(org_err,p)
            });
        },
        syn::Pat::Struct(ref mut pat_struct) => {
            let _ = pat_struct.fields.iter_mut().map(|f| clear_ref_mut(org_err,&mut *f.pat));
        },
        syn::Pat::Rest(_) => (),

        syn::Pat::Slice( ref mut pat_slice) => {
            let _ = pat_slice.elems.iter_mut().map(|p|{
                clear_ref_mut(org_err,p)
            });
        },

        _ => {
            let msg = "Unexpected pattern for function argument.";
            abort!(Span::call_site(),org_err.origin(msg);note=error::PARAMETERS_ALLOWED_PATTERN_NOTE); 
        },
    }

}


pub fn pat_vars_flat_into_ident( pat: &syn::Pat ) -> Option<Ident> {

    let mut loc = vec![];
    
    match pat {
        syn::Pat::Ident( pat_ident) =>  { 
            return Some(pat_ident.ident.clone());
        },
        syn::Pat::TupleStruct( pat_tuple_struct) => {
            let vars = pat_tuple_struct.elems.iter().filter_map(|p|{
                pat_vars_flat_into_ident(p)
            });
            loc.extend(vars);
        },
        syn::Pat::Tuple( pat_tuple) =>{
            let vars = pat_tuple.elems.iter().filter_map(|p|{
                pat_vars_flat_into_ident(p)
            });
            loc.extend(vars);
        },
        syn::Pat::Struct( pat_struct) => {
            let vars= pat_struct.fields.iter().filter_map(|f| pat_vars_flat_into_ident(&f.pat));
            loc.extend(vars);
        },
        syn::Pat::Slice( pat_slice) => {
            let vars = pat_slice.elems.iter().filter_map(|p|{
                pat_vars_flat_into_ident(p)
            });
            loc.extend(vars);
        },
        syn::Pat::Rest(_) => return None,
        _ => {
            let msg = "Internal Error.'method::pat_vars_flat_into_ident'. Unexpected pattern for function argument.";
            abort!(Span::call_site(),msg;note=error::PARAMETERS_ALLOWED_PATTERN_NOTE); 
        },
    }

    if loc.len() == 0 {
        // the compiler allows an empty (Rest) pattern like  (..) : Tuple
        // we will use '__' identifier to be able to carry the type within the model
        // if the method contains two or more such parameters
        // it will trigger some naming conflict error  
        loc.push(format_ident!("__"));
    }

    return Some(name::combined_ident(loc));
}


fn flat_pat_fn_arg( pat_ty: &syn::PatType ) -> syn::PatType{

    let mut pat_ty = pat_ty.clone();

    let ident = pat_vars_flat_into_ident(&*pat_ty.pat).unwrap();
    let pat = syn::Pat::Ident(
        syn::PatIdent{ 
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

pub fn live_args_and_sig( args: &Vec<FnArg>, sig: &Signature ) -> ( Vec<FnArg>, Signature) {

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

pub fn to_async( lib: &Lib, sig: &mut Signature ) {
    match lib {
        Lib::Std => (),
        _ => { sig.asyncness = Some(Token![async](Span::call_site()));}
    }
}

// TOKENSTREAM FROM METHODS 

pub fn live_static_method( 
    actor_name: &Ident,
    method:     ActorMethod,
    live_mets:  &mut Vec<(Ident,TokenStream,Vec<Attribute>)>,
    live_generics: syn::TypeGenerics ) 
     
{

    let (ident, doc_attrs, vis, sig, arguments ) = 
        match &method {
            ActorMethod::Io   { ident, doc_attrs, vis, sig, arguments,..} => (ident, doc_attrs, vis, sig, Some(arguments)),
            ActorMethod::I    { ident, doc_attrs, vis, sig, arguments,..} => (ident, doc_attrs, vis, sig, Some(arguments)),
            ActorMethod::O    { ident, doc_attrs, vis, sig,..}            => (ident, doc_attrs, vis, sig, None ),
            ActorMethod::None { ident, doc_attrs, vis, sig,..}            => (ident, doc_attrs, vis, sig, None ),
        };
    
    let mut sig = sig.clone();
    let await_call = sig.asyncness.as_ref().map(|_|quote!{.await});
    let mut args = vec![];

    if let Some(arguments) = arguments{ 
        let (live_arguments,live_sig) = live_args_and_sig( &arguments, &sig );
        let (args_ident,_) = args_to_pat_type(&live_arguments);
        sig  = live_sig;
        args = args_ident;
    }
    let turbofish = live_generics.as_turbofish();
    let stat_met = quote! {
        #vis #sig {
            #actor_name #turbofish :: #ident (#(#args),*) #await_call
        }
    };

    live_mets.push((ident.clone(),stat_met,doc_attrs.clone()));
}





pub fn to_raw_parts (

    vars: &Vars,
    Cont{ live_mets,debug_arms,direct_arms,script_fields,.. }: &mut Cont,
    aaa : &ActorAttributeArguments,
    oneshot: &OneshotChannel,
    MpscChannel{ sender_call,.. }: &MpscChannel,
    live_generics: syn::TypeGenerics, 

){  
    let ActorAttributeArguments{ lib,interact,.. } = &aaa;
    let Vars {actor,cust_name,script_name,impl_vars,inter_send,msg,..} = &vars;
    let ImplVars{ actor_name,actor_methods,.. } = &impl_vars;

    let group_wrap_variant = impl_vars.get_group_script_wrapper();
    let live_meth_send_recv = oneshot.decl(None);
    
    for mut method in actor_methods.clone() {
        
        method = method.reset();

        if method.is_static(){
            live_static_method(&actor_name, method, live_mets,live_generics.clone());
            continue;
        }

        let (mut sig, script_field_name) = method.get_sig_and_field_name();
        let await_call = sig.asyncness.as_ref().map(|_|quote!{.await});
        to_async(lib, &mut sig);

        let error_send = error::direct_send(&script_name,&script_field_name);

        // Debug arm
        let add_arm = | debug_arms: &mut Vec<TokenStream>,ident: &Ident | {

            let str_field_name = format!("{}::{}",script_name.to_string() ,ident.to_string());

            let debug_arm = quote! {
                #script_name :: #script_field_name {..} => write!(f, #str_field_name),
            };
            debug_arms.push(debug_arm);
        };

        let some_inter_vars = 
            |interact: bool,org_err:&OriginVars, sig: &Signature, arguments: &Vec<FnArg>, one: Option<&OneshotChannel>| -> Option<InterVars>
            {
                if interact { 
                    crate::model::get_variables( &org_err, sig, arguments,one )
                }
                else {
                    // check for inter vars conflict
                    if let Some(inter_var) = crate::model::check_send_recv( arguments, None ){
                        let msg = org_err.origin(error::var_name_conflict(&inter_var,"parameter"));
                        abort!(Span::call_site(),msg;note= error::INTER_SEND_RECV_RESTRICT_NOTE);
                    }
                    None
                }
        };

        let inter_met_names = vars.get_inter_live_methods(&aaa);
        let check_met_name  = |ident: &Ident,org_err: &OriginVars| {
            if inter_met_names.contains(&ident){
                let msg = org_err.origin(error::var_name_conflict(&ident.to_string(),"method"));
                abort!(Span::call_site(),msg);
            }
        };

        match &method {

            ActorMethod::Io   { doc_attrs, vis, org_err,  ident, arguments, output,.. } => {
                check_met_name(ident,org_err);
                let mut inter_vars = some_inter_vars(*interact, org_err, &sig, arguments,None);
                let (live_arguments,live_sig) = live_args_and_sig( &arguments, &sig );
                let (args_ident,_) = args_to_pat_type(&live_arguments);


                // Debug Arm push
                add_arm(debug_arms, &script_field_name);

                // Direct Arm
                let arm_match    = quote! { 
                    #script_field_name {  #(#args_ident),* ,  #inter_send }
                };

                let direct_arm = {
                    quote! {
                        #script_name :: #arm_match => {#inter_send .send( #actor.#ident (#(#args_ident),*) #await_call ) #error_send ;}
                    }
                };
                direct_arms.push(direct_arm);
                
                // Live Method
                let recv_output = oneshot.recv_call(cust_name,&ident);
                let msg_variant = (*group_wrap_variant)(quote!{ #script_name :: #arm_match });
                
                let (inter_gets, sig) = 
                if let Some(inter_vars) = &mut inter_vars{
                    ( Some( inter_vars.get_getters_decl()), inter_vars.new_sig.clone() )
                } else {( None,live_sig)};

                let live_met    = quote! {
                    #vis #sig {
                        #live_meth_send_recv
                        #inter_gets
                        // declaring getters here
                        let #msg = #msg_variant ;
                        #sender_call
                        #recv_output
                    }
                };

                live_mets.push((ident.clone(),live_met,doc_attrs.clone()));

                // Script Field Struct
                let send_pat_type = oneshot.pat_type_send(&*output);

                let script_field = quote! {
                    #script_field_name {
                        #(#live_arguments),* ,
                        #send_pat_type,
                    }
                };
                script_fields.push(script_field);
            },



            ActorMethod::I    { doc_attrs,vis,org_err, ident, arguments ,..} => {
                check_met_name(ident,org_err);
                let mut inter_vars = some_inter_vars(*interact, org_err, &sig, arguments, Some(oneshot));


                let (live_arguments,live_sig) = live_args_and_sig( &arguments, &sig );
                let (args_ident,_) = args_to_pat_type(&live_arguments);
    
                // Debug Arm push
                add_arm(debug_arms, &script_field_name);

                // Direct Arm
                let arm_match = quote!{ 
                    #script_field_name{ #(#args_ident),* }
                };
            
                let direct_arm = quote!{
                    #script_name::#arm_match => {#actor.#ident (#(#args_ident),*) #await_call;},
                };

                direct_arms.push(direct_arm);

                // Live Method
                let msg_variant = (*group_wrap_variant)(quote!{ #script_name :: #arm_match });
                
                // get getters decl and change sig
                let (inter_gets, sig, ret_chan_end) = 
                if let Some(inter_vars) = &mut inter_vars{
                    (
                        Some(inter_vars.get_getters_decl()),
                        inter_vars.new_sig.clone(),
                        inter_vars.some_ret_name(),
                    )

                } else {( None,live_sig,None)};

                let live_met = quote!{
                    #vis #sig {

                        #inter_gets
                        let #msg = #msg_variant ;
                        #sender_call
                        #ret_chan_end
                    }
                };
                live_mets.push((ident.clone(),live_met,doc_attrs.clone()));
            
                // Script Field Struct
                let script_field = quote!{
                    #script_field_name {
                        #(#live_arguments),*
                    }
                };
                script_fields.push(script_field);
                
                
            },
            ActorMethod::O { doc_attrs, vis, ident, org_err, output ,..} => {
                
                check_met_name(ident,org_err);

                // Debug Arm push
                add_arm(debug_arms, &script_field_name);

                // Direct Arm
                let arm_match = quote!{ 
                    #script_field_name{ inter_send }
                };
    
                let direct_arm = quote!{
                    #script_name::#arm_match => {inter_send.send(#actor.#ident () #await_call) #error_send ;}
                };
                direct_arms.push(direct_arm);



                // Live Method
                let recv_output = oneshot.recv_call(cust_name,&ident);
                let msg_variant = (*group_wrap_variant)(quote!{ #script_name :: #arm_match });
                let live_met = quote!{
                    #vis #sig {
                        #live_meth_send_recv
                        let #msg = #msg_variant ;
                        #sender_call
                        #recv_output
                    }
                };
                live_mets.push((ident.clone(), live_met,doc_attrs.clone()));
            
                // Script Field Struct
                let send_pat_type = oneshot.pat_type_send(&*output);


                let script_field = quote!{
                    #script_field_name {
                        #send_pat_type,
                    }
                };
                script_fields.push(script_field);
            },

            ActorMethod::None { doc_attrs,vis, ident ,org_err,..} => {

                check_met_name(ident,org_err);
                // Debug Arm push
                add_arm(debug_arms, &script_field_name);

                // Direct Arm
                let arm_match = quote!{ 
                    #script_field_name {} 
                };
    
                let direct_arm = quote!{
                    #script_name::#arm_match => {#actor.#ident () #await_call;},
                };
                direct_arms.push(direct_arm);

                // Live Method
                let msg_variant = (*group_wrap_variant)(quote!{ #script_name :: #arm_match });
                let live_met = quote!{
                    #vis #sig {
                        let #msg = #msg_variant ;
                        #sender_call
                    }
                };
                live_mets.push((ident.clone(),live_met,doc_attrs.clone()));
            
                // Script Field Struct
                let script_field = quote!{
                    
                    #script_field_name {}
                };
                script_fields.push(script_field);
            },
        }
    } 
}







