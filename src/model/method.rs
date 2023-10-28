use crate::error::{self,met_new_found};
use crate::model::{name,Cont,Vars,Lib,ImplVars,Model,interact::{self,InteractVariables}};

use syn::{Visibility,Signature,Ident,FnArg,Type,ReturnType,ImplItem,ItemImpl,Receiver,Token};
use proc_macro_error::abort;
use proc_macro2::{TokenStream,Span};
use quote::{quote,format_ident};

#[derive(Debug,Clone)]
pub enum ActorMethod {
    Io  { vis: Visibility, sig: Signature, ident: Ident, stat: bool,  arguments: Vec<FnArg>, output: Box<Type> },   
    I   { vis: Visibility, sig: Signature, ident: Ident,              arguments: Vec<FnArg>,                   },    
    O   { vis: Visibility, sig: Signature, ident: Ident, stat: bool,                         output: Box<Type> },    
    None{ vis: Visibility, sig: Signature, ident: Ident                                                        }, 
}

impl ActorMethod {

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

    pub fn get_mut_sig(&mut self) -> &mut Signature {
        &mut self.new_sig
    }

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

    pub fn  live_ret_statement(&self,  init_live: &TokenStream ) -> TokenStream {
       
        match self.res_opt {
            Some(true)  =>  quote!{ Ok ( #init_live )},
            Some(false) =>  quote!{ Some( #init_live )},
            None        =>  quote!{ #init_live },
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
    let old  = format!(" {} ", old.to_string());
    let new  = format!(" {} ",new.to_string());
    let str_type = type_str.replace(&old, &new);

    if let Ok(ty) = syn::parse_str::<T>(&str_type) {
        return ty;
    }

    let msg = format!("Internal Error. 'method::replace'. Could not parse &str to provided type!");
    abort!(Span::call_site(), msg);
}

pub fn get_new_sig( sig: &Signature, ty: &Type) -> Signature {

    let ty_name = quote!{#ty}.to_string() ;
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

fn get_sigs(item_impl: &syn::ItemImpl) -> Vec<(Visibility,Signature)>{
    let mut res :Vec<(Visibility,Signature)> = Vec::new();

    for itm in &item_impl.items {
        match itm {
            ImplItem::Fn( met ) => {
                if is_vis(&met.vis) {
                    res.push((met.vis.clone(),met.sig.clone()));
                }
            },
            _ => (),
        }
    }
    res
}


pub fn get_methods( actor_type: &syn::Type, item_impl: ItemImpl, stat: bool, mut act: bool) -> (Vec<ActorMethod>, Option<ActorMethodNew>){

    let mut loc              = vec![];
    let mut method_new = None;
    let ident_new                       = format_ident!("new");
    let ident_try_new                   = format_ident!("try_new");
    // let mut actor                        = Model::Actor.eq(mac);

    // use item_vis for `group` 
    let sigs = get_sigs(&item_impl);

    for (vis,sig) in sigs {

        if is_self_refer(&sig){
            loc.push(sieve( vis,explicit(&sig,actor_type),Some(false)));
        } else {
            
            if act {
                // check if there is a function "new" or "try_new"
                if sig.ident.eq(&ident_new) || sig.ident.eq(&ident_try_new){
                
                    let(new_sig,res_opt) = check_self_return(&mut sig.clone(),actor_type);
                    let method = sieve( vis,sig.clone(),Some(true));
                    method_new = ActorMethodNew::try_new( method, new_sig, res_opt );
                    act = false;
                    continue; 
                } 
            }
    
            if stat {
                if is_return(&sig){
                    loc.push(sieve( vis,explicit(&sig,actor_type),Some(true)));
                }
            }
        }
    }
    (loc, method_new)
}

pub fn sieve( vis: Visibility, sig: Signature, stat: Option<bool>) -> ActorMethod {

    let stat = if stat.is_some(){ stat.unwrap() } else { is_self_refer(&sig) };
    let (ident,arguments,output) = ident_arguments_output(&sig);

    let arg_bool = { arguments.iter()
        .any( |a| match a { FnArg::Typed(_) => true, _ => false}) };
    


    match output.clone() {
        ReturnType::Type(_,output) => { 

            if arg_bool {
                // let vars = 
                    // if interact {
                    //     // Get variables
                    //     interact::get_variables(actor_type, &sig, &arguments,true)
                    // } else {
                    //     // here checking for inter_variables
                    //     interact::check_send_recv(actor_type, &sig, &arguments,None);
                    //     None
                    // };
                return ActorMethod::Io{ vis, sig, stat, ident, arguments, output };
            } else {
                return ActorMethod::O{ vis, sig, stat, ident, output };
            }
        },
        ReturnType::Default => {

            if arg_bool {
                // let vars = 
                    // if interact {
                    //     // Get variables
                    //     interact::get_variables(actor_type, &sig, &arguments,false)
                    // } else {
                    //     // here checking for inter_variables
                    //     interact::check_send_recv(actor_type, &sig, &arguments,None);
                    //     None
                    // };
                return ActorMethod::I{ vis, sig, ident, arguments };
            } else {
                return ActorMethod::None{ vis, sig, ident };
            }
        },
    }

    // old 
    /*
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
     */


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

// NEW

pub fn args_to_pat_type(args: &Vec<FnArg>) -> (Vec<Box<syn::Pat>>, Vec<Box<Type>>) {

    let mut pats = Vec::new();
    let mut types  = Vec::new();

    for i in args  { 
        match i { 
            FnArg::Typed(arg) => { 
                // check 
                // let words = vec!["inter_send","inter_recv"];
                // if let Some(pos) = 
                // contains_inter_words(&arg.pat,&words).iter().position(|&x| x == true){
                //     let msg = format!("Conflicting name case `{}`.",words[pos]);
                //     abort!(Span::call_site(),msg);
                // }
                pats.push(arg.pat.clone());
                types.push(arg.ty.clone());
            },
            _ => (),
        }
    }
    (pats,types)    
}


// pub fn contains_inter_words( pat: &syn::Pat, names: &Vec<&str> ) -> Vec<bool> {

//     let mut s = quote!{ #pat }.to_string();
//     let char_set = vec![ '(', ')', '{', '}', '[', ']', ':', '<', '>', ',' ];
//     for c in char_set {
//         s = s.replace(c,&format!(" {c} "))
//     }

//     let mut loc = Vec::new();

//     for &name in names {

//         if s.contains(&format!(" {name} ")){ loc.push(true); } 
        
//         else { loc.push(false) }
//     }
//     return loc
// }

/*


Interact Arguments Limitations ( per method ): 

    1) One of `inter_send` or `inter_recv`.
    2) Unlimeted user arguments.
    3) Unlimeted `inter` getters.

implementing `interact`

if 
   off) 
    just check the `inter` words cases
    need a better error messaging 
    explaining the where this error occurs 


   on ) 
   a) check if the `pat` is an `Ident`
            false) check the `inter` words cases

            true) check if the ident is prefixed with ` inter_`
                 false)  push and continue

                 true) extract the second part of the ident
                       `inter_ident`  `ident`

                       check if the arg type is 
                       a) Sender
                               ident     type            return 
                               send  =>  Sender<T>  
                                                        true)  whaits inside the function returns the type 
                                                        false) returns the channel receiver     

                               any   =>  Sender<T>     
                                                        true) whaits inside the function on
                                                              a) Receiver<any> and mutates through inter_set_any
                                                              b) Receiver<return type> and  returns the type 

                                                        false) waits on  Receiver<any> and mutates through inter_set_any
                                                               and returns ()  


                       b) Receiver
                                 ident     type            return 
                                recv => Receiver<T>      
                                                        true) Raise error unspecified behaviour 
                                                        false) return the Sender<type>

                                any => Receiver<T>  
                                                        true) Raise error
                                                        false) Raise error  use recv for this sort of actions    



                       c) Type  
                                ident     type            return 

                                any       AnyType         create a variable any 
                                                          let any = inter_get_any()




The check will happen for inter_recv and inter_send in all 
cases as they are the only 

"Conflicting name case"





I has to check for iter_recv and inter_send in all pats

    false) check for othe inter_values

    true) check for 


// THE NOTE MESSAGE 
"In situations where user-defined variables, types, or other \
 elements could potentially overlap with model internals (such \
 as method names, arguments, and generic types), the model \
 employs a naming convention. It prefixes its variables and \
 types with `inter` in any style, ensuring a clear distinction \
 and preventing any inadvertent collisions."




Interthread naming convention.


 1. For function arguments never use words that are prepended with `inter_` or `Inter`.
    Specifically every method block will start with declaration of 'oneshot' channel with 
    sequent `inter_send` and `inter_recv` variables. Also method `inter_set_name` hase 
    a generic type `InterName` so the only way to make sure user variables and types do not 
    collide with internal model types the rule of thumb is not using any type names that start with 
    'inter' in any form 'inter_', 'INTER' or 'Inter'. 

*/

//OLD
/*
pub fn args_to_ident_type(args: &Vec<FnArg>) -> (Vec<Ident>, Vec<Box<Type>>){
    let mut tuple = 0usize;
    let mut new_tuple_ident = || {tuple +=1; format_ident!("tuple_{}",&tuple)};

    let mut tuple_struct = 0usize;
    let mut new_tuple_struct_ident = || {tuple_struct +=1; format_ident!("tuple_struct_{}",&tuple_struct)};

    let mut strct = 0usize; 
    let mut new_struct_ident = || {strct +=1; format_ident!("struct_{}",&strct)};


    let mut idents = Vec::new();
    let mut types  = Vec::new();

    for i in args  { 
        match i { 
            FnArg::Typed(arg) => { 
                if let Some(id) = match *arg.pat.clone() {
                    syn::Pat::Ident(pat_id) => Some(pat_id.ident.clone()),
                    syn::Pat::Tuple(_pat_tuple) => Some(new_tuple_ident()),
                    syn::Pat::TupleStruct(_pat_struct) => Some(new_tuple_struct_ident()),
                    syn::Pat::Struct(_pat_tuple_struct) => Some(new_struct_ident()),
                    _ => abort!(Span::call_site(),error::invalid_fn_arg_pattern(i)),
                }{
                    idents.push(id);
                    types.push(arg.ty.clone());
                }
            },
            // this needs an error so bad
            _ => (),
        }
    }
    (idents,types)    
}

 */


pub fn arguments_pat_type( args: &Vec<FnArg> ) -> (TokenStream, TokenStream) { 

    let (idents,types) = args_to_pat_type(args); 
    let args_ident =  quote!{ (#(#idents),*)};
    let args_type  =  quote!{ (#(#types),*) };
    ( args_ident, args_type )
}

pub fn to_async( lib: &Lib, sig: &mut Signature ) {
    
    match lib {
        Lib::Std => (),
        _ => {
            sig.asyncness = Some(Token![async](Span::call_site()));
        }
    }
}


// TOKENSTREAM FROM METHODS 

pub fn live_static_method( 
    actor_name: &Ident,
         ident: Ident, 
           vis: Visibility,
       mut sig: Signature,
          args: TokenStream,
     live_mets: &mut Vec<(Ident,TokenStream)> ) {

    change_signature_refer(&mut sig);
    let await_call = sig.asyncness.as_ref().map(|_|quote!{.await});
    let stat_met = quote! {
        #vis #sig {
            #actor_name::#ident #args #await_call
        }
    };
    live_mets.push((ident,stat_met));
}




use super::{ActorAttributeArguments, OneshotChannel, MpscChannel};
pub fn to_raw_parts (
    
    Vars {
        actor,
        cust_name,
        actor_name,
        script_name,
        impl_vars,..
    }: &Vars,
    Cont{
        live_mets,
        debug_arms,
        direct_arms,
        script_fields,..
    }: &mut Cont,
    ActorAttributeArguments{
        lib,..
    } : &ActorAttributeArguments,
    oneshot: &OneshotChannel,
    MpscChannel{
        sender_call,..
    }: &MpscChannel,
){  
    let ImplVars{ actor_methods,.. } = &impl_vars;

    // let variant = group_script_type.as_ref().map( |x| 
    //     {   
    //         let variant_name = crate::model::name::script_field(&field.clone().unwrap());
    //         quote!{  #x :: #variant_name } 
    //     }
    // );

    // let group_variant = | script_struct: TokenStream | -> TokenStream  {
    //     if let Some(var) = variant.clone() {
    //         quote!{ #var( #script_struct )}
    //     } else { script_struct }
    // };
    
    let group_wrap_variant = impl_vars.get_group_script_wrapper();
    let live_meth_send_recv = oneshot.decl(None);
    
    for method in actor_methods.clone() {
        
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




        match method {

            ActorMethod::Io   { vis, ident, stat,  arguments, output,.. } => {
                let (args_ident,args_type) = arguments_pat_type(&arguments);
                
                if stat {
                    live_static_method(&actor_name,ident, vis, sig, args_ident,live_mets)
                }
                else {
                    // Debug Arm push
                    add_arm(debug_arms, &script_field_name);

                    // Direct Arm
                    let arm_match        = quote! { 
                        #script_field_name { input: #args_ident,  inter_send }
                    };
                    let direct_arm       = quote! {
                        #script_name :: #arm_match => {inter_send.send( #actor.#ident #args_ident #await_call ) #error_send ;}
                    };
                    direct_arms.push(direct_arm);
                    
                    // Live Method
                    let recv_output = oneshot.recv_call(cust_name,&ident);
                    // let script_var = quote!{ #script_name :: #arm_match };
                    // let if_group_wrap_variant = group_variant(script_var);
                    let msg_variant = (*group_wrap_variant)(quote!{ #script_name :: #arm_match });
                    let live_met    = quote! {

                        #vis #sig {
                            #live_meth_send_recv
                            // declaring getters here
                            let msg = #msg_variant ;
                            #sender_call
                            #recv_output
                        }
                    };

                    live_mets.push((ident,live_met));

                    // Script Field Struct
                    let send_pat_type = oneshot.pat_type_send(&*output);
                    let script_field = quote! {
                        #script_field_name {
                            input: #args_type,
                            #send_pat_type,
                        }
                    };

                    script_fields.push(script_field);
                }
            },



            ActorMethod::I    { vis, ident, arguments ,..} => {
                
                let (args_ident,args_type) = arguments_pat_type(&arguments);
                
                // Debug Arm push
                add_arm(debug_arms, &script_field_name);

                // Direct Arm
                let arm_match = quote!{ 
                    #script_field_name{ input: #args_ident }
                };
    
                let direct_arm = quote!{
                    #script_name::#arm_match => {#actor.#ident #args_ident #await_call;},
                };
                direct_arms.push(direct_arm);

                // Live Method
                // let script_var = quote!{ #script_name :: #arm_match };
                // let if_group_wrap_variant = group_variant(script_var);
                let msg_variant = (*group_wrap_variant)(quote!{ #script_name :: #arm_match });

                let live_met = quote!{
    
                    #vis #sig {
                        let msg = #msg_variant ;
                        #sender_call
                        // here may be a return  statement other contr-part of the channel (change the output )
                    }
                };
                live_mets.push((ident,live_met));
            


                // Script Field Struct
                let script_field = quote!{
                    #script_field_name {
                        input: #args_type,
                        // here can be i_send or i_recv  ( remember to change the return Type of method )
                    }
                };
                script_fields.push(script_field);

            },
            ActorMethod::O    { vis, ident, stat, output ,..} => {
                let (args_ident,_) = arguments_pat_type(&vec![]);

                if stat {
                    live_static_method(&actor_name,ident, vis, sig, args_ident,live_mets)
                }
                else {
                    
                    // Debug Arm push
                    add_arm(debug_arms, &script_field_name);

                    // Direct Arm
                    let arm_match = quote!{ 
                        #script_field_name{ inter_send }
                    };
        
                    let direct_arm = quote!{
                        #script_name::#arm_match => {inter_send.send(#actor.#ident #args_ident #await_call) #error_send ;}
                    };
                    direct_arms.push(direct_arm);



                    // Live Method
                    // let script_var = quote!{ #script_name :: #arm_match };
                    // let if_group_wrap_variant = group_variant(script_var);
                    let recv_output = oneshot.recv_call(cust_name,&ident);
                    let msg_variant = (*group_wrap_variant)(quote!{ #script_name :: #arm_match });
                    let live_met = quote!{
                    
                        #vis #sig {
                            #live_meth_send_recv
                            let msg = #msg_variant ;
                            #sender_call
                            #recv_output
                        }
                    };
                    live_mets.push((ident, live_met));
                
                    // Script Field Struct
                    // let output_type  = (&*script_field_output)(output);
                    let send_pat_type = oneshot.pat_type_send(&*output);


                    let script_field = quote!{
                        #script_field_name {
                            // inter_send: #output_type
                            #send_pat_type,
                        }
                    };
                    script_fields.push(script_field);
                }
            },



            ActorMethod::None { vis, ident ,..} => {

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
                // let script_var = quote!{ #script_name :: #arm_match };
                // let if_group_wrap_variant = group_variant(script_var);
                let msg_variant = (*group_wrap_variant)(quote!{ #script_name :: #arm_match });
                let live_met = quote!{
                
                    #vis #sig {
                        let msg = #msg_variant ;
                        #sender_call
                    }
                };
                live_mets.push((ident,live_met));
            
                // Script Field Struct
                let script_field = quote!{
                    
                    #script_field_name {}
                };
                script_fields.push(script_field);
            },
        }
    } 
}




