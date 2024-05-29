// use crate::name;
use crate::model::{Debut,ActorMethod};

use quote::{quote,ToTokens};
use syn::{Type,Path,Ident,Signature};
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


pub fn met_new_found<T: ToTokens>(sig: &Signature, name: &T, bit: TokenStream, res_opt: Option<bool>) -> (String,String,String){
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

pub fn met_new_not_instance<T: ToTokens>(sig: &Signature, name: &T, bit: TokenStream, res_opt: Option<bool>) -> (String,String,String){
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

pub fn abort_async_no_lib(name: &Ident, met: &ActorMethod){

    let (sig,_ ) = met.get_sig_and_field_name();
    let sig = quote!{#sig}.to_string();
    let msg = format!("Actor {name} has 'async' methods but the runtime (lib) is not specified. \
    Method signature - '{sig}'.");
    abort!( Span::call_site(), msg; help=crate::error::AVAIL_LIB );
}


pub fn unknown_attr_arg( aa: &str, path: &Path ){

    let path_str = quote!{#path}.to_string().replace(" ","");
    let msg = format!("Unknown argument option  -  '{}'  for '{}' ", path_str,aa);

    match aa {

        val if val == "actor"   => abort!(path, msg ;help = AVAIL_ACTOR  ),                   
        val if val == "expand"  => abort!(path, msg ;help = AVAIL_EXPAND ),
        val if val == "example" => abort!(path, msg ;help = AVAIL_EXAMPLE),
        val if val == "edit"    => abort!(path, msg ;help = AVAIL_EDIT   ),
        val if val == "group"   => abort!(path, msg ;help = AVAIL_GROUP  ),

        _ => (),
    }
}

pub fn error_name_type(n: &syn::Path, t: &str ) -> String {

    let path_str = quote::quote!{#n}.to_string().replace(" ","");
    return format!("Expected a  < {} >  value for attribute argument '{}'.", t,path_str  );
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


pub static AVAIL_EDIT_GROUP: &'static str = "

The 'edit' option for `group` accepts a list of `actor` edits, \
prefixed with the field's name, like 'field_name::edit(..)'.\
Special case being `self::edit` referring to itself.


For instance, given a struct `AB`:

struct AB {
    pub a: Type,
    pub b: Type,
}

impl AB {
    pub fn new() -> Self {
        // Implementation
    }
}

To edit the model parts include in the 'edit' list:

edit(
    a::edit(..),
    b::edit(..), 
    self::edit( live(imp(new)) )
)

Special case: `edit(file)` is similar to \
`edit(a::edit(file), b::edit(file), self::edit(file))`, \
but the former entirely writes to the file and excludes the macro,\
while the latter only writes to the file, persisting in the form of \
`edit(a::edit, b::edit, self::edit)`.


";

pub static AVAIL_DEBUT: &'static str = "
\navailable 'debut' options:
    debut
        (
         legend
        )
    
    When using the `legend` option, the model is stored on the heap and \
    saved upon the last instance being dropped.
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
        
       debut(
             legend
            ) 
    interact
)]

*  -  default 
";


pub static AVAIL_GROUP: &'static str = "

#[interthread::group( 
    
AA  channel = 0 * 
              n (usize)

AA      lib = \"std\" *
              \"smol\"
              \"tokio\"
              \"async_std\"

AA     file = \"path/to/current/file.rs\"

AA     debut(
             legend
            )   

(AA)   show(
             self::show,
             ..
            )

(AA) include(
            self::include(
                          method_name,
                          ..
                         ),
            ..
            )

(AA) exclude(
            self::exclude(
                          method_name,
                          ..
                         ),
            ..
            )            

(AA)    edit( 
             self::edit(
                       script(..)
                       live(..)
                       ),
             ..
            ) 

(AA)    name(
             self::name = \"\",
             ..
            )

(AA)    path(
             a::path = \"path/to/type.rs\",
             ..
            )       
    )
]

  *     -  default 
  AA    -  similar to `actor` attribute argument.
 (AA)   -  a list of similar to `actor` attribute arguments.

 `self` - When specifying arguments for `(AA)`, remember \
 to prefix them with the corresponding field name to indicate \
 which member of the `group` they refer to. If the argument 
 pertains to the `group` struct itself, use the conventional `self` notation.


For instance, given a struct 'Group':

struct Group {
    pub a: Type,
    pub b: Type,
}

`edit( b::edit(script) )` - edit the `script` part of the field `b` model.

`path( a::path= \"path/to/type.rs\" )` - provide the path to field `a` Type definition.

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
pub static EDIT_GROUP_FILE_OUTSIDE: &'static str ="
The 'file' option must be used within the context of a 'field_name::edit' argument or 
special case `edit(file)`.
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

pub static ABOUT_SKIP: &'static str  =
"The `group` macro automatically considers any non-private field as a \
member of the group. The `skip` option is used to provide a list of \
non-private fields to be excluded from `group` membership.";

pub static SKIP_PRIVATE_FIELD_ERROR: &'static str  =
"Use of private field. Ensure only non-private fields are included in the `skip` list.";

pub static TUPLE_STRUCT_NOT_ALLOWED: &'static str  =
"The `group` macro cannot be applied to a tuple struct. Please use it with a regular struct instead.";

pub static GROUP_FIELD_TYPE: &'static str =
"The non-private fields in the `group` struct must be paths or identifiers representing potential valid 'actor' types.";

pub static REQ_FILE: &'static str  =
r#"Expected a 'file' argument `file = "path/to/current/file.rs"`."#;

// Mismatched impl block
pub static MISMATCHED_IMPL_BLOCK: &'static str = "
Mismatched impl block detected!

Possible causes:

    1) The 'file' parameter points to a different file with \
a similar or identical struct name.
    2) The macro may not be applied to the first impl block \
of the associated struct.
";

pub static HELP_TYPE_NAMING_CONFLICT: &'static str ="
    The model relies on a specific naming convention critical \
for generating accurate type names. However, it encounters \
issues in some cases:

    `foo_bar`   -> FooBar
    `_foo_bar`  -> FooBar
    `foo_bar_`  -> FooBar
    `_foo_bar_` -> FooBar

    Please ensure that the provided names adhere to the \
Rust camel case convention and differ by at least one character \
to avoid naming conflicts. This will allow the model to function \
correctly and generate accurate type names.
";

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

pub fn expected_path_ident(s: &str ) -> String {
   format!("Expected a path, `field`::{s} .")
}

pub fn type_naming_conflict(a: &Ident, b: &Ident )-> String {
    let ty_name = crate::model::name::script_field(a);
    format!("Naming conflict detected. Conflicting \
type names from the provided field names. Both`{a}` and `{b}` result \
in {ty_name} .")
}

pub fn double_decl(s: &str) -> String {
    format!("Double declaration of `{s}` option.")
}

pub fn end_of_life( name: &syn::Ident, debut: &Debut ) -> TokenStream {
    if debut.active(){
        let msg = if debut.is_legend(){
            format!("{name} [ {{:?}} ] to be continued ...")
        } else { format!("{name} [ {{:?}} ] the end ...")};
        quote!{ eprintln!(#msg,debut); }
    } else { 
        let msg = format!("{name} the end ...");
        quote!{ eprintln!(#msg); }
    }
}

pub fn direct_send(script_name: &syn::Ident, variant: &syn::Ident) -> TokenStream {
    let msg = format!("'{script_name}::{variant}.direct'. Sending on a closed channel.");
    quote!{.unwrap_or_else(|_error| core::panic!( #msg ))}

}

use std::path::PathBuf;
#[derive(Clone,Debug)]
pub struct OriginVars{
    pub path: Option<PathBuf>,
    pub actor_type: Type, 
    pub sig: Signature,
}

impl OriginVars {
    pub fn origin<T: ToString>(&self,e: T ) -> String {
        let e = e.to_string();
        let Self{ path,actor_type,sig} = self;
        let actor_name = quote!{#actor_type}.to_string();
        let path =  
        if let Some(p) = path{
            format!("Path : `{}` \n", p.to_string_lossy())
        } else { "".to_string() };
        let sig       = quote!(#sig).to_string();
        format!("{path}Object : `{actor_name}`\nMethod : `{sig}`\n\n{e} ") 
    }
}

pub static LEGEND_LIMIT_GENERIC: &str = 
"   The 'legend' option is not supported for generic objects.";

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

