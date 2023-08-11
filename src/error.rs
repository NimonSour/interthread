use crate::name;
use quote::{quote,ToTokens};
use proc_macro_error::abort;
use proc_macro2::TokenStream;

pub fn met_new_note_help<T: ToTokens>(name: &T) -> (String, String)  {
    let name = quote!{#name}.to_string();

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


pub fn met_new_found<T: ToTokens>(sig: &syn::Signature, name: &T, bit: TokenStream, res_opt: Option<bool>) -> (String,String,String){
    let sig_name      = sig.ident.to_string();
    let act_name      = quote!{ #name }.to_string();
    let mut bit_str   = bit.to_string();
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

pub fn met_new_not_instance<T: ToTokens>(sig: &syn::Signature, name: &T, bit: TokenStream, res_opt: Option<bool>) -> (String,String,String){
    let sig_name = sig.ident.to_string();
    let act_name = quote!{#name}.to_string();
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
        path = \"path/to/file.rs\" 

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
         
     Struct        Options        
         
    'script'    ( 
                 def        
                 imp(name, ..)
                )  

    'live'      ( 
                  def
                  imp(name, ..)
                  trt(name, ..)
                ) 

def  - Struct definition 
imp  - Struct methods 
trt  - Struct traits
name - method/trait name

    When employing the `imp` or `trt` option without providing a tuple list, \
the macro interprets it as a request to include all method/trait names.
    Similarly, for `script` or `live` a statement `edit(live)` implies `edit(live(def, imp, trt))`, \
a statement just `edit` implies `edit(live,script)` !
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
             script(..)
             live(..)
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
    let script_name = &name::script(cust_name);
    let msg        = format!("'{script_name}::play::queuing'. Failed to unwrap MutexGuard!");
    quote!{#msg}
}

pub fn end_of_life(name: &syn::Ident) -> TokenStream {
    let msg    = format!("{} end of life ...",&name.to_string());
    quote!{
        eprintln!(#msg);
    }
}

pub fn direct_send(cust_name: &syn::Ident) -> TokenStream {
    let script_name = &name::script(cust_name);
    let msg = format!("'{script_name}::direct.send'. Channel closed");
    quote!{#msg}
}

pub static SCRIPT_NO_TRT: &'static str = "
    The `actor`'s `Script struct` does not implement any traits. 
    Consequently, the use of the `trt` argument for `script` is not applicable. If your intention is \
to modify derived traits, consider using the `def` option instead.";


pub fn trait_new_sig<T: quote::ToTokens>(ty:&T, exists: bool) -> (String,String){
    let actor_ty = quote!{#ty}.to_string();
    let note = format!("
    Using the `actor` macro with a `trait` block is not as flexible \
    as when it is applied to an `impl` block. \n
    The `trait` must include a specific signature for the `new` \
    initiation function: \n \t
    fn new(s: Self) -> Self
    This signature, is the only available initiation signature \
    that the macro will consider for its functionality.
    \n"); 
    let msg = 
    if exists {
        format!("Expected signature `fn new (s:Self) -> Self` for {} ! \n",actor_ty)
    } else {
        format!("Expected signature `fn new (s:Self) -> Self` for {} not found !\n",actor_ty)
    };
    (msg,note)
}


pub fn item_vis() -> (String,String){
    //"The macros 'actor' and 'group' require the object itself and its \
    // - `fn` block: `fn`'s visibility itself
    let note = format!("The macro 'actor' require the object itself and its \
    methods to have explicit visibility (public or restricted) if they are intended \
    to be considered.
    
    The macros adhere to Rust's principles, where private functions are regarded as internal \
    helper functions, not intended for external use beyond the object body.
    
    The visibility level of the newly generated Actor Model types will \
    be preserved and applied from : \n 
    - `impl`  block: function `new` \n 
    - `trait` block: `trait`'s visibility itself ( which is the same as `new` function ) 
     
    Please ensure that the required visibility specifications are followed to use the 'actor' \
    macro effectively.\n") ;

    let help = format!("If a private Actor Model is desired, it is recommended to begin with \
    public visibility and then manually adjust visibility using the 'edit' option or \
    the macro 'example' to modify the types created by the macro.");


    (note,help)
}

// temp error new args 
pub static OLD_DIRECT_ARG: &'static str = "
    Since v1.0.0 `direct` argument is not aplicable. Use `edit(script(imp(direct)))` instead!
";

pub static OLD_PLAY_ARG: &'static str = "
    Since v1.0.0 `play` argument is not aplicable. Use `edit(script(imp(play)))` instead!
";

pub fn old_file_arg( path: String ) -> String {
    format!( "Since v1.0.0 `file` argument is not aplicable. Use `path= \"{}\"` instead!", &path )
}
