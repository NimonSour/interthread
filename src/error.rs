use crate::name;
use quote::quote;
use proc_macro_error::abort;
use proc_macro2::TokenStream;

pub fn met_new_note_help(name: &syn::Ident) -> (String, String)  {
    let name = name.to_string();

    let note = format!(
        "The object {name:?} must implement a public or restricted method named 'new' \
        that returns Self or {name}. If the function may fail to return \
        an instance of {name}, name it 'try_new' \
        and return a 'Result<{name}>' or 'Option<{name}'. 
        It is recommended to follow the standard Rust naming \
        convention and use 'try_new', but it is not mandatory.\n"
    );

    let help = format!("
    The flowing are possible method signatures:
    
    - returning Type
    
        pub fn new (arg, ...) -> Self
        pub(crate) fn new (arg, ...) -> Self
        pub fn new (arg, ...) -> {name} 
    
    - returning Option <Type>  
        
        pub fn try_new(arg, ...) -> Option<Self>
        pub fn try_new(arg, ...) -> Option<{name}>
                                or
                 -> custom::module::Option<{name}>
        
    - returning Result <Type>  
    
        pub fn try_new(arg, ...) -> Result<Self,...>
        pub fn try_new(arg, ...) -> Result<{name},...>
                                or
                    -> custom::module::Result<{name}>
                 -> custom::module::Result<{name}, E>

    ");

    return (note,help);
}


pub fn met_new_found(sig: &syn::Signature, name: &syn::Ident, bit: TokenStream, res_opt: Option<bool>) -> (String,String,String){
    let sig_name     = sig.ident.to_string();
    let act_name     = name.to_string();
    let mut bit_str  = bit.to_string();
    if bit_str == ""{
        bit_str = " ".to_string();
    }
    let msg = if res_opt.is_none() {

        if &sig_name == "new"{
            format!("'{act_name}::{sig_name}' expected to return \
            'Self' or '{act_name}'. \nFound: {bit_str:?} .")
        }
        else {
            format!("'{act_name}::{sig_name}' expected to return \
            'Result<Self>' or 'Result<{act_name}>' or 'Option<Self>' or \
            'Option<{act_name}>'. \nFound: {bit_str:?} .")
        }
    }
    else {
        //result 
        if res_opt.unwrap(){
            format!("'{act_name}::{sig_name}' expected to return \
            'Self' or '{act_name}' wrapped in a 'Result' type. \nFound: {bit_str:?} .")
        }

        //option 
        else {
            format!("'{act_name}::{sig_name}' expected to return \
            'Self' or '{act_name}' wrapped in a 'Option' type. \nFound: {bit_str:?} .")
        }
    };
    let (note,help) = crate::error::met_new_note_help(name);

    (msg,note,help)
}

pub fn met_new_not_instance(sig: &syn::Signature, name: &syn::Ident, bit: TokenStream, res_opt: Option<bool>) -> (String,String,String){
    let sig_name     = sig.ident.to_string();
    let act_name     = name.to_string();
    let bit_str  = bit.to_string();
    
    let msg = {
        //result 
        if res_opt.unwrap(){
            format!("'{act_name}::{sig_name}' expected to return \
            Result<'{act_name}'>. \nFound: {bit_str:?} .")
        }

        //option 
        else {
            format!("'{act_name}::{sig_name}' expected to return \
            Option<'{act_name}'>. \nFound: {bit_str:?} .")
        }
    };
    let (note,help) = crate::error::met_new_note_help(name);
    (msg,note,help)
} 

pub fn unknown_attr_arg( aa: &str, ident: &syn::Ident ){
    
    let msg = format!("Unknown option  -  {:?} for '{}' ", ident.to_string(),aa,);

    match aa.to_string() {

        val if val == "actor".to_string()   => abort!(ident, msg ;help = AVAIL_ACTOR  ),                   
        val if val == "expand".to_string()  => abort!(ident, msg ;help = AVAIL_EXPAND ),
        val if val == "example".to_string() => abort!(ident, msg ;help = AVAIL_EXAMPLE),
        val if val == "edit".to_string()    => abort!(ident, msg ;help = AVAIL_EDIT   ),
        _ => (),
    }
}

pub fn error_name_type(n: syn::Ident, t: String ) -> String {

    return format!("Expected a  < {} >  value for attribute argument '{}'.", t, n.to_string() );
}

pub static AVAIL_EXAMPLE: &'static str = "

#[interthread::example( 
   
    mod *
    main 

    (   
        file = \"path/to/file.rs\" 

        expand(actor,group) *
    )
)]


*  -  default       
";

pub static AVAIL_EXPAND: &'static str = "
Argument 'expand' takes a tuple of ident options.

Available ident options are: 

                        actor 
                        group 

Examples of expected usage:

    expand(actor), 
    expand(group), 
*   expand(actor,group) 


* - default 
";

pub static AVAIL_LIB:&'static str = "
\navailable 'lib' options:

*   \"std\"
    \"smol\"
    \"tokio\"
    \"async_std\"


*  -  default
";

pub static AVAIL_CHANNEL: &'static str ="
\navailable 'channel' options:

   Option             Type

*  \"inter\"          str
   0 | \"unbounded\"  str|int
   8                  int


*  -  default
";

pub static AVAIL_EDIT: &'static str = "
\navailable 'edit' options:
         
     Options        Description 
         
    'script'    'enum ActorScript'            
    'direct'    'impl ActorScript::actor_direct()'  
    'play'      'fn actor_play'               
    'live'      'struct ActorLive' 
    'live::new' 'struct ActorLive::new'
";


pub static AVAIL_ACTOR: &'static str = "
#[interthread::actor( 
    
    channel = \"inter\" *
              \"unbounded\" || 0
               8 

        lib = \"std\" *
              \"smol\"
              \"tokio\"
              \"async_std\"

        edit
        ( 
            script
            direct
            play
            live
            live::new
        ) 

        name = \"\" 

        assoc = false *
                 true
        
        id    = false *
                 true
    )
]


*  -  default 
";



pub fn live_send_recv(cust_name: &syn::Ident, ) -> (TokenStream, TokenStream){

    let live_name  = &name::live(cust_name);
    let send_msg = format!("'{live_name}::method.send'. Channel is closed!");
    let recv_msg = format!("'{live_name}::method.recv'. Channel is closed!");
    (quote!{#send_msg},quote!{#recv_msg})
}

pub fn live_guard(cust_name: &syn::Ident) -> TokenStream {
    let live_name  = &name::live(cust_name);
    let msg        = format!("'{live_name}::method'. Failed to unwrap MutexGuard!");
    quote!{#msg}
}

pub fn play_guard(cust_name: &syn::Ident) -> TokenStream {
    let play_name  = &name::play(cust_name);
    let msg        = format!("'{play_name}::queuing'. Failed to unwrap MutexGuard!");
    quote!{#msg}
}

pub fn end_of_life(name: &syn::Ident) -> TokenStream {
    let msg    = format!("{} end of life ...",&name.to_string());
    quote!{
        eprintln!(#msg);
    }
}

pub fn direct_send(cust_name: &syn::Ident) -> TokenStream {
    let direct_name  = &name::direct(cust_name);
    let msg = format!("'{direct_name}.send'. Channel closed");
    quote!{#msg}
}