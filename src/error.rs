// use crate::name;
use crate::model::method::ActorMethod;

use quote::{quote,ToTokens};
use proc_macro_error::abort;
use proc_macro2::TokenStream;
use proc_macro::Span;


pub fn met_new_note_help<T: ToTokens>(name: &T) -> (String,String) {
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

pub fn abort_async_no_lib(name: &syn::Ident, met: &ActorMethod){

    let (sig,_ ) = met.get_sig_and_field_name();
    let sig = quote!{#sig}.to_string();
    let msg = format!("Actor {name} has 'async' methods but the runtime (lib) is not specified. \
    Method signature - '{sig}'.");
    abort!( Span::call_site(), msg; help=crate::error::AVAIL_LIB );
}


pub fn unknown_attr_arg( aa: &str, ident: &syn::Ident ){
    
    let msg = format!("Unknown argument option  -  {:?} for '{}' ", ident.to_string(),aa,);

    match aa.to_string() {

        val if val == "actor".to_string()   => abort!(ident, msg ;help = AVAIL_ACTOR  ),                   
        val if val == "expand".to_string()  => abort!(ident, msg ;help = AVAIL_EXPAND ),
        val if val == "example".to_string() => abort!(ident, msg ;help = AVAIL_EXAMPLE),
        val if val == "edit".to_string()    => abort!(ident, msg ;help = AVAIL_EDIT   ),
        _ => (),
    }
}

pub fn error_name_type(n: &syn::Ident, t: &str ) -> String {

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

pub static AVAIL_EDIT: &'static str = "
\navailable 'edit' options:
         
     Struct        Options        
         
    'script'    ( 
                 def        
                 imp(name, ..)
                 trt(name, ..)
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


pub static AVAIL_DEBUT: &'static str = "
\navailable 'debut' options:
    debut
        (
        legend
             (
              path='..'  
             )
        )
    
    When employing the `legend` option without providing a tuple list \
    with a path in it like `debut(legend)` the model will be saved on the heap.
";

pub static AVAIL_ACTOR: &'static str = "
#[interthread::actor( 
    
    channel = 0 * 
              n (usize)

        lib = \"std\" *
              \"smol\"
              \"tokio\"
              \"async_std\"

        edit
            ( 
             script(..)
             live(..)
            ) 

        file = \"path/to/current/file.rs\"
        
        name = \"\" 

        assoc = false *
                 true
        
        debut 
             (
              legend(..)
             )  
            
    )
]

*  -  default 
";



pub static HELP_EDIT_FILE_ACTOR: &'static str = "
The 'file' identifier within the 'edit' argument customizes writing \
behavior. It allows you selectively write portions of the \
model to a file,  enabling edition of other parts while excluding those \
that have already been modified.

Here are two key guidelines to keep in mind when using the 'file' identifier:

1. Options `script` and `live`, along with their suboptions `def`, `imp`, and `trt`, \
as well as their respective arguments (the names of methods/traits), can only \
be declared once within their respective scopes.

2. While multiple 'file' declarations are allowed, nesting \
them is not permitted.

Example 1:
edit( script, live(file(def), imp))
                   ^^^^
   write:   live(def)
   exclude: script, live(imp)

Example 2:
edit( script(imp), file(live(def, imp)))
                   ^^^^
   write:   live(def, imp)
   exclude: script(imp)

Example 3:
edit( live(file(def), imp(try_new, file(try_old))))
           ^^^^                    ^^^^
   write:   live(def,imp(try_old))
   exclude: live(imp(try_new))

Special case: `edit(file)` is equivalent to \
`edit(file(script, live))` and writes the entire model.
";

// pub static ALREADY_ASSIGNED: &'static str = "Option has already been assigned."; 
// pub static ALREADY_DECLARED: &'static str = "Option has already been declared.";
pub static EXPECT_IDENT: &'static str = "Expected an identifier. Please pass only a single identifier without any namespace or path.";
pub static NESTED_FILE: &'static str  = "Nested `file` option!"; 

pub fn double_decl(s: &str) -> String {
    format!("Double declaration of `{s}` option.")
}

// pub fn assigned_already(s:&str) -> String {
//     format!("Option {s} has already been assigned.")
// }
// pub fn declared_already(s:&str) -> String {
//     format!("Option {s} has already been `file` declared.")
// }


pub fn live_send_recv(live_name:&syn::Ident ) -> (TokenStream, TokenStream){

    // let live_name  = &name::live(cust_name);
    let send_msg = format!("'{live_name}::method.send'. Channel is closed!");
    let recv_msg = format!("'{live_name}::method.recv'. Channel is closed!");
    (quote!{#send_msg},quote!{#recv_msg})
}

pub fn end_of_life(name: &syn::Ident) -> TokenStream {
    let msg    = format!("{name} end of life ...");
    quote!{ eprintln!(#msg); }
}

pub fn direct_send(script_name: &syn::Ident, variant: &syn::Ident) -> TokenStream {
    let msg = format!("'{script_name}::{variant}.direct'. Sending on a closed channel.");
    quote!{.unwrap_or_else(|_error| core::panic!( #msg ))}

}

// pub fn trait_new_sig<T: quote::ToTokens>(ty:&T, exists: bool) -> (String,String){
//     let actor_ty = quote!{#ty}.to_string();
//     let note = format!("
//     Using the `actor` macro with a `trait` block is not as flexible \
//     as when it is applied to an `impl` block. \n
//     The `trait` must include a specific signature for the `new` \
//     initiation function: \n \t
//     fn new(s: Self) -> Self
//     This signature, is the only available initiation signature \
//     that the macro will consider for its functionality.
//     \n"); 
//     let msg = 
//     if exists {
//         format!("Expected signature `fn new (s:Self) -> Self` for {} ! \n",actor_ty)
//     } else {
//         format!("Expected signature `fn new (s:Self) -> Self` for {} not found !\n",actor_ty)
//     };
//     (msg,note)
// }


// pub fn item_vis() -> (String,String){
//     //"The macros 'actor' and 'group' require the object itself and its \
//     // - `fn` block: `fn`'s visibility itself
//     let note = format!("The macro 'actor' require the object itself and its \
//     methods to have explicit visibility (public or restricted) if they are intended \
//     to be considered.
    
//     The macros adhere to Rust's principles, where private functions are regarded as internal \
//     helper functions, not intended for external use beyond the object body.
    
//     The visibility level of the newly generated Actor Model types will \
//     be preserved and applied from : \n 
//     - `impl`  block: function `new` \n 
//     - `trait` block: `trait`'s visibility itself ( which is the same as `new` function ) 
     
//     Please ensure that the required visibility specifications are followed to use the 'actor' \
//     macro effectively.\n") ;

//     let help = format!("If a private Actor Model is desired, it is recommended to begin with \
//     public visibility and then manually adjust visibility using the 'edit' option or \
//     the macro 'example' to modify the types created by the macro.");


//     (note,help)
// }

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

// v.2.0.0
pub static OLD_ARG_ID: &'static str = "
    Since v2.0.0 `id` argument is not aplicable. Use `debut` instead!
";
