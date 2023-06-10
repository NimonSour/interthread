

#[derive(Debug,Clone)]
pub enum ActorMethod {
    Io  { method: syn::ImplItemFn, stat: bool, ident: syn::Ident, arguments: Vec<syn::FnArg>, output: Box<syn::Type> },   
    I   { method: syn::ImplItemFn, ident: syn::Ident, arguments: Vec<syn::FnArg> },    
    O   { method: syn::ImplItemFn, stat: bool, ident: syn::Ident, output: Box<syn::Type> },    
    None{ method: syn::ImplItemFn, ident: syn::Ident }, 
}

impl ActorMethod {

    pub fn get_sig_and_field_name(&self) -> (syn::Signature, syn::Ident) {
        let (sig,name) = match self {

            Self::Io   { method, ident, ..} => (method.sig.clone(),ident) ,
            Self::I    { method, ident, ..} => (method.sig.clone(),ident) ,
            Self::O    { method, ident, ..} => (method.sig.clone(),ident) ,
            Self::None { method, ident, ..} => (method.sig.clone(),ident) ,
        };
        (sig, crate::name::script_field(name))
    }

    pub fn is_async(&self) -> bool {
        if let Some(_)= match self {

            Self::Io   { method,..} => method.sig.asyncness ,
            Self::I    { method,..} => method.sig.asyncness ,
            Self::O    { method,..} => method.sig.asyncness ,
            Self::None { method,..} => method.sig.asyncness ,
        }{
            return true;
        };
        false
    }
}
 

 
#[derive(Debug,Clone)]
pub struct ActorMethodNew {
    pub method:            syn::ImplItemFn,
    pub new_sig:            syn::Signature,
    pub res_opt:              Option<bool>,
    pub ident:                  syn::Ident,
    pub arguments: Option<Vec<syn::FnArg>>,
    pub output:             Box<syn::Type>,      

}

impl ActorMethodNew {

    pub fn try_new( met: ActorMethod, new_sig: syn::Signature,  res_opt: Option<bool> ) -> Option<Self>{
        
        match met {

            ActorMethod::Io   { method,ident,arguments,output,.. } =>  {
                return  Some(ActorMethodNew{method,ident,arguments: Some(arguments), output, new_sig, res_opt });
            },
            ActorMethod::O    { method,ident,output,..} =>  {
                return  Some(ActorMethodNew{method,ident,arguments: None, output, new_sig, res_opt });
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
}

 
 
fn str_to_return_type( s: String) -> Option<syn::ReturnType> {

    if let Ok(return_type)  = syn::parse_str::<syn::Type>(&s){
        let return_type_quote = quote::quote! {
            -> #return_type
        };
        if let Ok(re_ty) = syn::parse2::<syn::ReturnType>(return_type_quote.into()){
            return Some(re_ty);
        };
        return None;
    };
    None
}

fn is_self_return(name: &syn::Ident, sig: &syn::Signature) -> Option<(syn::Signature,Option<bool>)> {

    let self_ident   = quote::format_ident!("{}","Self");
    let option_ident = quote::format_ident!("{}","Option");
    let result_ident = quote::format_ident!("{}","Result");
    
    let mut signature = sig.clone();
    match &sig.output {
        syn::ReturnType::Type(_,ty_path) => {
            match ty_path.as_ref(){ 
                syn::Type::Path( p ) => {

                    let return_ident = p.path.segments.first().unwrap().ident.clone();
                    let mut name_self  = false;
                    let mut res_opt :Option<bool> = None;

                    if  self_ident.eq(&return_ident) { 
                        return Some((signature, res_opt)) ;
                    } 

                    else if (*name).eq(&return_ident) {
                        let output = str_to_return_type(self_ident.to_string())?;
                        signature.output       = output;
                        return Some((signature, res_opt)) ;
                    }

                    else if  option_ident.eq(&return_ident) {
                        res_opt = Some(false);
                    }

                    else if result_ident.eq(&return_ident) {
                        res_opt = Some(true);
                    }
                    match p.path.segments.first().unwrap().arguments.clone(){

                        syn::PathArguments::AngleBracketed(mut gen_arg) =>{

                            for arg in gen_arg.args.iter_mut(){

                                match arg {
                                    syn::GenericArgument::Type( t ) =>{
                                        match t {
                                            syn::Type::Path( pp ) => {

                                                for ret_path_seg in pp.path.segments.iter_mut(){

                                                    let path_seg_ident =  ret_path_seg.ident.clone();

                                                    if self_ident.eq(&path_seg_ident) {
                                                        name_self = true;
                                                    }
                                                    else if (*name).eq(&path_seg_ident) {
                                                        ret_path_seg.ident = self_ident.clone();
                                                        name_self = true;
                                                    }
                                                }
                                            },
                                            _ => return None,
                                        }
                                    },
                                    _ => return None,
                                }
                            }
                            if res_opt.is_some() {

                                if name_self {

                                    let s = quote::quote!{ #return_ident #gen_arg }.to_string();
                                    let output = str_to_return_type(s)?;
                                    signature.output = output;

                                    return Some( (signature, res_opt) );
                                }
                                return None;
                            }
                            return None;
                        },
                        _ => return None,
                    }
                },
                _ => return None,
            }
        },
        _ => return None,
    }
}

fn is_return( met: &syn::ImplItemFn ) -> bool {
    match met.sig.output {
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

// needs an argument for static methods
pub fn get_methods( name: &syn::Ident, item_impl: syn::ItemImpl, stat:bool ) -> (Vec<ActorMethod>, Option<ActorMethodNew>){

    let mut loc                   = vec![];
    let mut method_new      = None;
    let ident_new                            = quote::format_ident!("{}","new");
    let ident_try_new                        = quote::format_ident!("{}","try_new");

    for i in item_impl.items {
        match i {
            syn::ImplItem::Fn( m ) => {
                match m.vis {
                    // check visibility "pub"
                    syn::Visibility::Public(_) => {

                        if is_self_refer(&m.sig){
                            loc.push(sieve(m.clone(),Some(false)));

                        } else {
                            // check if there is a function "new" or "try_new"
                            if m.sig.ident.eq(&ident_new) || m.sig.ident.eq(&ident_try_new){
                                if let Some(new_sig_ret) = is_self_return(name,&m.sig){
                                    let method = sieve(m.clone(),Some(true));
                                    method_new = ActorMethodNew::try_new( method, new_sig_ret.0, new_sig_ret.1 );
                                }
                            } 
                            else {
                                if stat {
                                    if is_return(&m){
                                        loc.push(sieve(m.clone(),Some(true)));
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

pub fn sieve( method: syn::ImplItemFn, stat: Option<bool> ) -> ActorMethod {

    let stat = if stat.is_some(){ stat.unwrap() } else { is_self_refer(&method.sig) };
    let (ident,arguments,output) = ident_arguments_output(&method);

    let arg_bool = { arguments.iter()
        .any( |a| match a { syn::FnArg::Typed(_) => true, _ => false}) };


    match output.clone() {

        syn::ReturnType::Type(_,output) => { 

            if arg_bool {
                return ActorMethod::Io{ method, stat, ident, arguments, output };
            } else {
                return ActorMethod::O{ method, stat, ident, output };
            }
        },
        syn::ReturnType::Default => {

            if arg_bool {
                return ActorMethod::I{ method, ident, arguments };
            } else {
                return ActorMethod::None{ method, ident };
            }
        },
    }
}
 
pub fn ident_arguments_output( method: &syn::ImplItemFn ) -> (syn::Ident,Vec<syn::FnArg>,syn::ReturnType) {
    let punct_to_vec = 
    |p: syn::punctuated::Punctuated<syn::FnArg,syn::token::Comma>| -> Vec<syn::FnArg> { p.into_iter().collect::<Vec<_>>() };

    let ident          = method.sig.ident.clone();
    let arguments = punct_to_vec( method.sig.inputs.clone());
    let output    = method.sig.output.clone();

    (ident, arguments, output)
}
 
pub fn change_signature_refer( signature: &mut syn::Signature ) {
    let recv: syn::Receiver = syn::parse2(quote::quote!{ &self }).unwrap();
    let slf = syn::FnArg::Receiver(recv);
    signature.inputs.insert(0,slf);
}


pub fn args_ident_type( args: &Vec<syn::FnArg> ) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) { 

    let argument_idents = |arguments: &Vec<syn::FnArg>|{
        arguments
        .into_iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Typed(arg) => { match *arg.pat.clone() {
                syn::Pat::Ident(pat_id) => Some(pat_id.ident),
                                                _ => None,
                }
            },
            _ => None,
        })

        .collect::<Vec<_>>()
    };


    let argument_types = |arguments: &Vec<syn::FnArg>|{
        arguments
        .into_iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Typed(arg) => Some(arg.ty.clone()),
                                           _ => None,
        }).collect::<Vec<_>>()
    };

    let idents      = argument_idents(args);
    let types   = argument_types(args);
    let args_ident =  quote::quote!{ (#(#idents),*)};
    let args_type  =  quote::quote!{ (#(#types),*) };
    ( args_ident, args_type )
}


