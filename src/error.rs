
use crate::model::{method::ModelMethod, method::ModelOutput};
use quote::{quote,ToTokens};
use syn::{Path,Signature};
use proc_macro_error::abort;
use proc_macro2::TokenStream;


pub fn met_new_note_help<T: ToTokens>(name: &T) -> (String,String) {
    let name = quote!{#name}.to_string().replace(" ","");

    let note = format!(
        "The object {name:?} must implement a public or restricted method named 'new' \
        that returns Self or {name}. If the function may fail to return \
        an instance of {name}, name it 'try_new' \
        and return a 'Result<{name}>' or 'Option<{name}'. 
        It is recommended to follow the standard Rust naming \
        convention and use 'try_new', but it is not mandatory.\n"
    );

    let help = format!("
    the following are possible method signatures:
    
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


pub fn met_new_found<T: ToTokens>(sig: &Signature, name: &T, bit: TokenStream, res_opt: ModelOutput) -> (String,String,String){
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
        if res_opt.is_result(){
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

pub fn met_new_not_instance<T: ToTokens>(sig: &Signature, name: &T, bit: TokenStream, res_opt: ModelOutput) -> (String,String,String){
    let sig_name = sig.ident.to_string();
    let act_name = quote!{#name}.to_string();
    let bit_str  = bit.to_string();
    
    let msg = {
        //result 
        if res_opt.is_result(){
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

pub fn abort_async_no_lib(met: &ModelMethod){
    let msg = format!("'async' methods present but the runtime (lib) is not specified.");
    abort!( &met.get_met().sig.asyncness, msg; help=crate::error::AVAIL_LIB );
}


pub fn unknown_attr_arg( aa: &str, path: &Path ){

    let path_str = quote!{#path}.to_string().replace(" ","");
    let msg = format!("Unknown argument option  -  '{}'  for '{}' ", path_str,aa);

    match aa {

        val if val == "actor"   => abort!(path, msg ;help = AVAIL_ACTOR  ),                   
        val if val == "expand"  => abort!(path, msg ;help = AVAIL_EXPAND ),
        val if val == "example" => abort!(path, msg ;help = AVAIL_EXAMPLE),
        val if val == "edit"    => abort!(path, msg ;help = AVAIL_EDIT   ),
        val if val == "family"  => abort!(path, msg ;help = AVAIL_FAMILY ),

        _ => (),
    }
}

pub fn error_name_type(n: &syn::Path, t: &str ) -> String {

    let path_str = quote::quote!{#n}.to_string().replace(" ","");
    return format!("Expected a  < {} >  value for attribute argument '{}'.", t,path_str  );
}

pub static AVAIL_EXAMPLE: &'static str = "

#[interthread::example( 
   
    (   
        main

        path = \"path/to/file.rs\" 

        expand(actor,family) *
    )
)]


*  -  default       
";

pub static AVAIL_EXPAND: &'static str = "
Argument 'expand' takes a tuple of ident options.

Available ident options are: 

                        actor 
                        family 

Examples of expected usage:

    expand(actor), 
    expand(family), 
*   expand(actor,family) 


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
    `debut`
";

pub static AVAIL_ACTOR: &'static str = "
#[interthread::actor( 
    
     channel = 0 * 
               n (usize)

         lib = \"std\" *
               \"smol\"
               \"tokio\"
               \"async_std\"

        edit( 
             script(..)
             live(..)
            ) 

        file = \"path/to/current/file.rs\"
        
        name = \"\" 

        show

     include|exclude 
        
       debut

    interact
)]

*  -  default 
";


pub static HELP_EDIT_FILE_ACTOR: &'static str = "
The 'file' identifier within the 'edit' argument customizes writing \
behavior. It allows you selectively write portions of the \
model to the file,  enabling edition of other parts while excluding those \
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

Special case: `edit(file)` is similar to `edit(file(script, live))`, \
but the former entirely writes to the file and excludes the macro,\
while the latter only writes to the file, persisting in the form of \
`edit(script, live)`.
";


pub static EXPECT_LIST: &'static str = "Expected a list!";
pub static EXPECT_IDENT: &'static str = "Expected an identifier. Please pass only a single identifier without any namespace or path.";

pub static NESTED_FILE: &'static str  = "Nested `file` option!"; 
pub static NOTE_SPECIAL_FILE_EDIT: &'static str  = 
"`edit(file)` is the only scenario where the file argument \
can be specified as an identifier. In all other cases, it must be \
used in the list form `file(..)`."; 

pub static HELP_SPECIAL_FILE_EDIT: &'static str  =
"`edit(file)` serves as a special writable equivalent to `edit`. \
This notation will directly replace the macro with the actual \
generated code. However, in explicit notation like \
`edit(file(script, live))`, the macro will persist in the 
file as `edit(script, live)`, despite that the whole model \
is written to the file.";


pub static REQ_FILE: &'static str  =
r#"Expected a 'file' argument `file = "path/to/current/file.rs"`."#;



pub static FILTER_CONURENT_USE_OF_OPTIONS: &'static str = "Unexpected. Concurrent use of 'include' and 'exclude' options.";

pub static FILTER_OPTION_USE_HELP: &'static str =
"The 'actor' offers two filtering options: 'include' and 'exclude'. 
When applied to the set {a, b, c, d}, these options function as follows:

    include(a, c) -> {a, c}
    exclude(a, c) -> {b, d}

Only one option can be applied at a time.";

pub static PARAMETERS_ALLOWED_PATTERN_NOTE: &'static str ="
The model will accept the following patterns in method parameters:
    1) Ident ( variable name ) - 'foo : Type'
    2) Tuple - '(a, b, ..) : (Type, Type)' 
    3) Array - '[a,b,..] : [Type; n]'
    4) Struct  - 'Struct { a, b, .. } : MyStruct'
    5) Tuple Struct - 'Struct(a, b, ..) : MyTupleStruct'

";

pub static INTER_VARIABLE_SUPPORTED_PATTERN_NOTE: &'static str ="
The ONLY pattern supported for `inter variable` is 'ident' (a variable name 'foo : Type')!";

pub fn double_decl(s: &str) -> String {
    format!("Double declaration of `{s}` option.")
}

pub fn direct_send(script_name: &syn::Ident, variant: &syn::Ident) -> TokenStream {
    let msg = format!("'{script_name}::{variant}.direct'. Sending on a closed channel.");
    quote!{.unwrap_or_else(|_error| core::panic!( #msg ))}
}

pub static INTERACT_VARS_HELP: &str = "
    The `interact` option is designed to provide the model with \
comprehensive non-blocking functionality, along with convenient \
internal getter calls to access the state of the `live` instance.\
Please consult the documentation for correct usage.
";

pub static INTER_SEND_RECV_RESTRICT_NOTE : &'static str =
"   Using method arguments named `inter_send` or `inter_recv` will \
interfere with the model's internal variables. To proceed with \
these names, explicitly opt in by providing the argument `interact` \
to the macro (see option `interact` in documentation). Otherwise, \
consider renaming the arguments.";

pub static CONCURRENT_INTER_SEND_RECV : &'static str =
"   Concurrent use of `inter_send` and `inter_recv`.\
Please make sure to access only one end of the channel, not both simultaneously.";

pub static NOT_ACCESSIBLE_CHANNEL_END: &'static str =
"   `inter_send` and `inter_recv` variables cannot be accessed in methods that return a type.";

pub static EXPECTED_IDENTIFIER_SHOW: &'static str = "Expected an identifier ( show ).";

pub fn var_name_conflict<V,P>( var: V, part: P ) -> String 
where 
    V: ToString,
    P: ToString,
{
    format!("   Naming conflict: `{}`. Please choose a different \
    {} name.", var.to_string(),part.to_string())
} 


pub static AVAIL_FAMILY: &'static str = "
#[interthread::family( 
    
  ~ channel = 0 * 
              n (usize)

        lib = \"std\" *
              \"tokio\"
              \"async_std\"

        edit( 
             live(..)
            ) 

        Mutex | RwLock *
               
        file = \"path/to/current/file.rs\"
        
        name = \"\" 

        show

        debut

        actor(  
                first_name = \"\" 

                edit( 
                    script(..)
                    live(..)
                    ) 

                include|exclude 

                show

                interact

                channel ~
            )

)]

~  -  override 
*  -  default 
";


pub static NOT_ALLOW_FAMILY_DIRECT_MUT_REF: &'static str = 
"Mutable references (`mut`) are not allowed. 
To use this method with the current signature, rename the receiver from `actor` to another name.";

pub static NOT_ALLOW_FAMILY_IN_SMOL: &'static str = "The 'family' macro is only supported for the following runtimes: 'std' (standard), 'tokio' and 'async_std'.";
