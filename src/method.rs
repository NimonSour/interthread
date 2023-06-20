

#[derive(Debug,Clone)]
pub enum ActorMethod {
    Io  { sig: syn::Signature, ident: syn::Ident, stat: bool,  arguments: Vec<syn::FnArg>, output: Box<syn::Type> },   
    I   { sig: syn::Signature, ident: syn::Ident,              arguments: Vec<syn::FnArg>                         },    
    O   { sig: syn::Signature, ident: syn::Ident, stat: bool,                              output: Box<syn::Type> },    
    None{ sig: syn::Signature, ident: syn::Ident                                                                  }, 
}

impl ActorMethod {

    pub fn get_sig_and_field_name(&self) -> (syn::Signature, syn::Ident) {
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

    pub sig:                syn::Signature,
    pub new_sig:            syn::Signature,
    pub res_opt:              Option<bool>,
    pub ident:                  syn::Ident,
    pub arguments: Option<Vec<syn::FnArg>>,
    pub output:             Box<syn::Type>,      

}

impl ActorMethodNew {

    pub fn try_new( met: ActorMethod, new_sig: syn::Signature,  res_opt: Option<bool> ) -> Option<Self>{
        
        match met {

            ActorMethod::Io   { sig,ident,arguments,output,.. } =>  {
                return  Some(ActorMethodNew{ sig,ident,arguments: Some(arguments), output, new_sig, res_opt });
            },
            ActorMethod::O    { sig,ident,output,..} =>  {
                return  Some(ActorMethodNew{ sig,ident,arguments: None, output, new_sig, res_opt });
            } 
            _   =>  return  None,
        };
    }

    pub fn get_arguments(&self)-> Vec<syn::FnArg> {
        if let Some(arguments) = &self.arguments{
            return arguments.clone();
        }
        vec![]
    }

    pub fn  live_ret_statement(&self,  live_var: &syn::Ident ) -> proc_macro2::TokenStream {
       
        match self.res_opt {
            Some(true)  =>  quote::quote!{ Ok ( #live_var )},
            Some(false) =>  quote::quote!{ Some( #live_var )},
            None        =>  quote::quote!{ #live_var },
        }
    }

    pub fn unwrap_sign(&self) -> proc_macro2::TokenStream {
        if self.res_opt.is_none(){ quote::quote!{}} else { quote::quote!{?}}
    }
        
}

pub fn replace<T, O, N>(ty: &T, old: &O, new: &N) -> T
where
    T: syn::parse::Parse + quote::ToTokens,
    O: ToString + ?Sized,
    N: ToString + ?Sized,
{
    let str_ret_type = quote::quote! {#ty}.to_string();
    let new_str_ret_type = str_ret_type.replace(&old.to_string(), &new.to_string());

    if let Ok(ty) = syn::parse_str::<T>(&new_str_ret_type) {
        return ty;
    }

    let msg = format!("Internal Error. 'method::replace'. Could not parse &str to provided type!");
    proc_macro_error::abort!(proc_macro2::Span::call_site(), msg);
}


pub fn new_sig( sig: &syn::Signature, name: &syn::Ident) -> syn::Signature {
    let mut signature = replace(sig, "Self",name);
    signature.output  = replace(&sig.output,name,"Self");
    signature
}


fn check_self_return(name: &syn::Ident, sig: &syn::Signature) -> (syn::Signature,Option<bool>) {

    let option_ident = quote::format_ident!("Option");
    let result_ident = quote::format_ident!("Result");
    
    match &sig.output {
        syn::ReturnType::Type(_,ty_path) => {
            match ty_path.as_ref(){ 
                syn::Type::Path( p ) => {

                    if  p.path.is_ident("Self") { 
                        return (new_sig(sig,name), None);
                    } 

                    else if p.path.is_ident(name) {
                        return (new_sig(sig,name), None);
                    }

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
                                            match ty {
                                                syn::Type::Path( typ ) => {

                                                    if typ.path.is_ident("Self"){
                                                        return (new_sig(sig,name), res_opt);
                                                    }

                                                    else if typ.path.is_ident(name){
                                                        return (new_sig(sig,name), res_opt);
                                                    }

                                                    let (msg,note,help) = crate::error::met_new_not_instance(sig, name, quote::quote!{#typ},res_opt);
                                                    proc_macro_error::abort!(typ,msg;note=note;help=help); 
                                                },
                                                bit => {
                                                    let (msg,note,help) = crate::error::met_new_found(sig, name, quote::quote!{#segment},res_opt);
                                                    proc_macro_error::abort!(bit,msg;note=note;help=help); 
                                                },
                                            }
                                        },
                                        bit => {
                                            let (msg,note,help) = crate::error::met_new_found(sig, name, quote::quote!{#segment},res_opt);
                                            proc_macro_error::abort!(bit,msg;note=note;help=help); 
                                        },
                                    }
                                }
                                let (msg,note,help) = crate::error::met_new_found(sig, name, quote::quote!{#segment},res_opt);
                                proc_macro_error::abort!(segment.arguments,msg;note=note;help=help); 
                            },
                            bit => {
                                let (msg,note,help) = crate::error::met_new_found(sig, name, quote::quote!{#segment},res_opt);
                                proc_macro_error::abort!(bit,msg;note=note;help=help);
                            },
                        }
                    }
                    let (msg,note,help) = crate::error::met_new_found(sig, name, quote::quote!{#p},None);
                    proc_macro_error::abort!(p,msg;note=note;help=help);
                },
                bit => {
                    let (msg,note,help) = crate::error::met_new_found(sig, name, quote::quote!{#bit},None);
                    proc_macro_error::abort!(bit,msg;note=note;help=help);
                },
            }
        },
        
        bit => { 
            let (msg,note,help) = crate::error::met_new_found(sig, name, quote::quote!{#bit},None);
            proc_macro_error::abort!(bit,msg;note=note;help=help);
        },
    }
}

fn is_return( sig: &syn::Signature ) -> bool {
    match sig.output {
        syn::ReturnType::Default => return false,
        syn::ReturnType::Type(_, _) => return true,
    }
}

fn is_self_refer (signature: &syn::Signature ) -> bool{
    if let Some(input) = signature.inputs.iter().next() {
        match input {
            syn::FnArg::Receiver(receiver) => {
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

pub fn explicit( sig: &syn::Signature, name: &syn::Ident ) -> syn::Signature{
    replace( sig, "Self", name )
}

// needs an argument for static methods
pub fn get_methods( name: &syn::Ident, item_impl: syn::ItemImpl, stat:bool ) -> (Vec<ActorMethod>, Option<ActorMethodNew>){

    let mut loc                   = vec![];
    let mut method_new      = None;
    let ident_new                            = quote::format_ident!("new");
    let ident_try_new                        = quote::format_ident!("try_new");

    for i in item_impl.items {
        match i {
            syn::ImplItem::Fn( m ) => {
                match m.vis {
                    // check visibility "pub"
                    syn::Visibility::Public(_) => {

                        if is_self_refer(&m.sig){
                            loc.push(sieve(explicit(&m.sig,name),Some(false)));

                        } else {

                            // check if there is a function "new" or "try_new"
                            if m.sig.ident.eq(&ident_new) || m.sig.ident.eq(&ident_try_new){

                                // if let Some(new_sig_ret) = is_self_return(name,&mut m.sig.clone()){
                                let(new_sig,res_opt) = check_self_return(name,&mut m.sig.clone());
                                let method = sieve(m.sig.clone(),Some(true));
                                method_new = ActorMethodNew::try_new( method, new_sig, res_opt ); 
                            } 

                            else {
                                if stat {
                                    if is_return(&m.sig){
                                        loc.push(sieve(explicit(&m.sig,name),Some(true)));
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
    (loc, method_new)
}

pub fn sieve( sig: syn::Signature, stat: Option<bool> ) -> ActorMethod {

    let stat = if stat.is_some(){ stat.unwrap() } else { is_self_refer(&sig) };
    let (ident,arguments,output) = ident_arguments_output(&sig);

    let arg_bool = { arguments.iter()
        .any( |a| match a { syn::FnArg::Typed(_) => true, _ => false}) };


    match output.clone() {

        syn::ReturnType::Type(_,output) => { 

            if arg_bool {
                return ActorMethod::Io{ sig, stat, ident, arguments, output };
            } else {
                return ActorMethod::O{ sig, stat, ident, output };
            }
        },
        syn::ReturnType::Default => {

            if arg_bool {
                return ActorMethod::I{ sig, ident, arguments };
            } else {
                return ActorMethod::None{ sig, ident };
            }
        },
    }
}

pub fn ident_arguments_output( sig: &syn::Signature  ) -> (syn::Ident,Vec<syn::FnArg>,syn::ReturnType) {
    let punct_to_vec = 
    |p: syn::punctuated::Punctuated<syn::FnArg,syn::token::Comma>| -> Vec<syn::FnArg> { p.into_iter().collect::<Vec<_>>() };

    let ident          = sig.ident.clone();
    let arguments = punct_to_vec( sig.inputs.clone());
    let output    = sig.output.clone();

    (ident, arguments, output)
}
 
pub fn change_signature_refer( signature: &mut syn::Signature ) {
    let recv: syn::Receiver = syn::parse2(quote::quote!{ &self }).unwrap();
    let slf = syn::FnArg::Receiver(recv);
    signature.inputs.insert(0,slf);
}

pub fn args_to_ident_type(args: &Vec<syn::FnArg>) -> (Vec<syn::Ident>, Vec<Box<syn::Type>>){

    let mut idents = Vec::new();
    let mut types  = Vec::new();

    for i in args  { 
        match i { 
            syn::FnArg::Typed(arg) => { 
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

pub fn arguments_ident_type( args: &Vec<syn::FnArg> ) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) { 

    let (idents,types) = args_to_ident_type(args); 
    let args_ident =  quote::quote!{ (#(#idents),*)};
    let args_type  =  quote::quote!{ (#(#types),*) };
    ( args_ident, args_type )
}


