
mod attribute;
mod use_macro;
mod show;
mod file;
mod actor_gen;
mod name;
mod method;


static INTERTHREAD: &'static str            = "interthread";
static INTER_EXAMPLE_DIR_NAME: &'static str = "INTER_EXAMPLE_DIR_NAME";
static INTER: &'static str                  = "inter";
static GROUP: &'static str                  = "group";
static ACTOR: &'static str                  = "actor";
static EXAMPLE: &'static str                = "example";
static EXAMPLES: &'static str               = "examples";
static MAIN: &'static str                   = "main"; 

/// # Code transparency and exploration
///  
/// The [`example`](./attr.example.html) macro serves as a 
/// convenient tool for code transparency and exploration.
/// By automatically generating an expanded code file,
/// it provides developers with a tangible representation of
/// the code produced by the `interthread` macros. 
/// 
/// Having the expanded code readily available in the `examples/inter`
/// directory offers a few key advantages:
///  
/// - It provides a clear reference point for developers to inspect 
/// and understand the underlying code structure.
/// 
/// - The generated code file serves as a starting point for 
/// customization. Developers can copy and paste the generated code 
/// into their own project files and make custom changes as needed. 
/// This allows for easy customization of the generated actor 
/// implementation to fit specific requirements or to add additional 
/// functionality.
/// 
/// - Helps maintain a clean and focused project structure, 
/// with the `examples` directory serving as a dedicated location for 
/// exploring and experimenting with the generated code.
/// 
/// [`example`](./attr.example.html) macro helps developers to 
/// actively engage with the generated code 
/// and facilitates a smooth transition from the generated code to a 
/// customized implementation. This approach promotes code transparency,
/// customization, and a better understanding of the generated code's 
/// inner workings, ultimately enhancing the development experience 
/// when working with the `interthread` macros.
/// 
/// Consider a macro [`actor`](./attr.actor.html)  inside the project 
/// in `src/my_file.rs`.
/// 
/// Filename: my_file.rs 
/// ```rust
/// 
/// pub struct Number;
/// 
/// // you can have "example" macro in the same file
/// // #[interthread::example(file="src/my_file.rs")]
/// 
/// #[interthread::actor(channel=5)]
/// impl Number {
///     pub fn new(value: u32) -> Self {
///         Self 
///     }
/// }
/// 
/// ```
/// 
/// Filename: main.rs 
/// ```rust
/// 
/// #[interthread::example(file="src/my_file.rs")]
/// fn main(){
/// }
/// 
/// ```
/// 
/// The macro will create and write to `examples/inter/my_file.rs`
/// the content of `src/my_file.rs` with the 
/// [`actor`](./attr.actor.html) macro expanded.
/// 
/// 
/// ```text
/// my_project/
/// ├── src/
/// │  ├── my_file.rs      <---  macro "actor" 
/// |  |
/// │  └── main.rs         <---  macro "example" 
/// |
/// ├── examples/          
///    ├── ...
///    └── inter/      
///       ├── my_file.rs   <--- expanded "src/my_file.rs"  
/// ```
///
/// [`example`](./attr.example.html) macro can be placed on any 
/// item in any file within your `src` directory, providing 
/// flexibility in generating example code for/from different 
/// parts of your project.
///
///  It provides two options for generating example code files: 
///   - [`mod`](##mod) 
///   - [`main`](##main) (default)
///
///   ## mod 
///   The macro generates an example code file within the 
///   `examples/inter` directory. For example:
///
///   ```text
///   #[example(file="my_file.rs")]
///   ```
///
///   This is equivalent to:
///
///   ```text
///   #[example(mod(file="my_file.rs"))]
///   ```
///
///   The generated example code file will be located at 
///   `examples/inter/my_file.rs`.
///
///   This option provides developers with an easy way to 
///   view and analyze the generated code, facilitating code 
///   inspection and potential code reuse.
///
///   ## main 
///
///   This option is used when specifying the `main` argument 
///   in the `example` macro. It generates two files within 
///   the `examples/inter` directory: the expanded code file 
///   and an additional `main.rs` file. For example:
///
///   ```text
///   #[example(main(file="my_file.rs"))]
///   ```
///
///   This option is particularly useful for testing and 
///   experimentation. It allows developers to quickly 
///   run and interact with the generated code by executing:
///
///   ```terminal
///   $ cargo run --example inter
///   ```
///
///   The expanded code file will be located at 
///   `examples/inter/my_file.rs`, while the `main.rs` file 
///   serves as an entry point for running the example.
/// 
/// # Arguments
/// 
/// - [`file`](#file)
/// - [`expand`](#expand) (default)
/// 
/// # file
/// 
/// 
/// The file argument is a required parameter of the example macro.
/// It expects the path to the file that needs to be expanded.
/// 
/// This argument is essential as it specifies the target file 
/// for code expansion.
/// One more time [`example`](./attr.example.html) macro can be 
/// placed on any item in any file within your `src` directory.
/// 
///  
/// # expand
/// 
/// This argument allows the user to specify which 
/// `interthread` macros to expand. 
/// 
/// By default, the value of `expand` includes 
/// the [`actor`](./attr.actor.html) and 
/// [`group`](./attr.group.html) macros.
/// 
/// For example, if you want to expand only the
/// [`actor`](./attr.actor.html) macro in the generated 
/// example code, you can use the following attribute:
/// 
/// ```text
/// #[example(file="my_file.rs",expand(actor))]
/// ```
/// This will generate an example code file that includes 
/// the expanded code of the [`actor`](./attr.actor.html) macro,
/// while excluding other macros like 
/// [`group`](./attr.group.html).
/// 
 

#[proc_macro_error::proc_macro_error]
#[proc_macro_attribute]
pub fn example( attr: proc_macro::TokenStream, _item: proc_macro::TokenStream ) -> proc_macro::TokenStream {

    let mut eaa   = attribute::ExampleAttributeArguments::default();

    let aaa_parser = 
    syn::meta::parser(|meta| eaa.parse(meta));
    syn::parse_macro_input!(attr with aaa_parser);


    let (file, lib)  = file::expand_macros(&eaa.get_file(),&eaa.expand);

    if eaa.main { 
        show::example_show(file, &eaa.get_file(), Some(lib));
    } else {
        show::example_show(file, &eaa.get_file(), None ); 
    }
    let msg = format!("The file has been SUCCESSFULLY created !");
    let note  = "The macro has successfully written to a file. To avoid potential issues and improve maintainability, it is recommended that you comment out the macro after its successful execution. To proceed, please comment out the macro and re-run the compilation.";
    
    proc_macro_error::abort!( proc_macro2::Span::call_site(),msg; note = note);
    
}

 
/// ## Evolves a regular object into an actor
/// 
/// The macro is placed upon an implement block of an object,
/// which has a public method named `new` returning  `Self`.
///
/// In case if the initialization could potentially fail, 
/// the method can be named `try_new` 
/// and return `Option<Self>` or `Result<Self>`.
///  
/// # Arguments
///  
///
/// - [`channel`](#channel)
/// - [`lib`](#lib) 
/// - [`edit`](#edit)
/// - [`name`](#name)
/// - [`assoc`](#assoc)
///
/// 
/// 
/// # channel
///
/// The `channel` argument specifies the type of channel. 
///
/// - `"inter"` (default)  
/// - `"unbounded"` or `0` 
/// - `8` ( [`usize`] buffer size)
/// > **Note:** The default `"inter"` option is experimental and primarily intended for experimentation purposes, specifically with the `lib = "std"` setting. It is recommended to avoid using this option unless you need it.
/// 
/// The two macros
/// ```text
/// #[actor(channel="unbounded")]
/// ```
/// and
/// ```text
/// #[actor(channel=0)]
/// ```
/// are identical and both specify an unbound channel.
/// 
/// When specifying an [`usize`] value for the `channel` argument 
/// in the [`actor`](./attr.actor.html) macro, such as 
/// ```text
/// #[actor(channel=4)]
/// ```
/// the actor will use a bounded channel with a buffer size of 4.
/// This means that the channel can hold up to 4 messages in its 
/// buffer before blocking/suspending the sender.
///
/// Using a bounded channel with a specific buffer size allows 
/// you to control the memory usage and backpressure behavior 
/// of the actor. When the buffer is full, any further attempts 
/// to send messages will block until there is available space. 
/// This provides a natural form of backpressure, allowing the 
/// sender to slow down or pause message production when the 
/// buffer is near capacity
/// 
/// # lib
///
/// The `lib` argument specifies the 'async' library to use.
///
/// - `"std"` (default)
/// - `"smol"`
/// - `"tokio"`
/// - `"async_std"`
///
/// ## Examples
/// ```
/// struct MyActor;
/// 
/// #[actor(channel=10, lib ="tokio")]
/// impl MyActor{
///     pub fn new() -> Self
/// }
/// #[tokio::main]
/// async fn main(){
///     let my_act = MyActorLive::new();
/// }
/// ```
/// 
/// 
/// 
/// # edit
///
/// The `edit` argument specifies the available editing options.
/// When using this argument, the macro expansion will exclude the code related to `edit` options, allowing the user to manually implement and customize those parts according to their specific needs.
/// 
/// - [`script`](index.html#script)
/// - [`direct`](index.html#direct)
/// - [`play`](index.html#play)
/// - [`live`](index.html#live)
/// - `live::new`  
///
/// 
/// ## Examples
///```
///use std::sync::mpsc;
/// 
///pub struct MyActor {
///    value: i8,
///}
//
///#[actor(channel=2, edit(play))]
///impl MyActor {
///
///    pub fn new( value: i8 ) -> Self {
///        Self{value}
///    }
///    pub fn increment(&mut self) -> i8{
///        self.value += 1;
///        self.value
///    }
///}
///
/// // manually create "play" function 
///pub fn my_actor_play( 
///     receiver: mpsc::Receiver<MyActorScript>,
///    mut actor: MyActor) {
///     
///    while let Ok(msg) = receiver.recv() {
///        /* do something */
///        msg.my_actor_direct(&mut actor);
///    }
///    eprintln!("{} the end ", "MyActor");
///}
///
///
///fn main() {
///
///    let my_act = MyActorLive::new(0);
///    let mut my_act_clone = my_act.clone();
///
///    let handle = std::thread::spawn(move || -> i8{
///        my_act_clone.increment()
///    });
///    
///    let value = handle.join().unwrap();
///
///    assert_eq!(value, 1);
///}
///```
///
/// > **Note:** The expanded `actor` can be viewed using [`interthread::example`](./attr.example.html) 
/// 
/// 
/// 
/// # name
/// 
/// The `name` attribute allows developers to provide a custom name for `actor`, overriding the default naming conventions of the crate. This can be useful when there are naming conflicts or when a specific naming scheme is desired.  
/// 
/// - "" (default): No name specified
///
/// ## Examples
///```
///pub struct MyActor;
/// 
///#[actor(name="OtherActor")]
///impl MyActor {
///
///   pub fn new() -> Self {
///       Self{}
///   }
///}
///fn main () {
///   let other_act = OtherActorLive::new();
///}
///```
/// 
/// 
/// 
/// # assoc
/// 
/// The `assoc` option indicates whether associated functions of the actor struct are included in generated code as instance methods, allowing them to be invoked on the generated struct itself. 
/// 
/// - true  (default)
/// - false
/// 
///  ## Examples
///```
///pub struct Aa;
///
///#[actor(name="Bb")]
///impl Aa {
///
///    pub fn new() -> Self { Self{} }
///
///    pub fn is_even( n: u8 ) -> bool {
///        n % 2 == 0
///    }
///}
///
///fn main() {
///    
///    let bb = BbLive::new();
///    assert_eq!(bb.is_even(84), Aa::is_even(84));
///}
///```
///
/// 
/// 






#[proc_macro_error::proc_macro_error]
#[proc_macro_attribute]
pub fn actor( attr: proc_macro::TokenStream, item: proc_macro::TokenStream ) -> proc_macro::TokenStream {
    
    let impl_block                      = syn::parse_macro_input!(item as syn::ItemImpl);
    let mut paaa    = attribute::ParseActorAttributeArguments::default();

    let attr_str = attr.clone().to_string();

    if !attr_str.is_empty(){

        let aaa_parser      = 
        syn::meta::parser(|meta| paaa.parse(meta));
        syn::parse_macro_input!(attr with aaa_parser);
    }
    let aaa = paaa.get_arguments();


    let mut inter_gen_actor = actor_gen::ActorMacroGeneration::new( /*name,*/ aaa, impl_block );
    let code = inter_gen_actor.generate();
    quote::quote!{#code}.into()
   
}

/// ## Currently under development (((
/// 
/// The `group` macro, although not currently included 
/// in the `interthread` crate.It aims to address 
/// several critical challenges encountered when
///  working with the `actor` macro:
/// 
/// - Instead of creating separate threads for each object, 
/// the `group` macro will enable the user to create an actor 
/// that represents a group of objects, consolidating 
/// their processing and execution within a single thread.
/// 
/// 
/// - In scenarios where objects are already created or imported,
/// and the user does not have the authority to implement 
/// additional methods such as  "new" or "try_new",
/// the `group` macro should offer a way to include 
/// these objects as part of the actor system.
///
/// Although the `group` macro is not currently part of the 
/// `interthread` crate, its development aims to offer a 
/// comprehensive solution to these challenges, empowering 
/// users to efficiently manage groups of objects within an 
/// actor system.
/// 
/// Check `interthread` on ['GitHub'](https://github.com/NimonSour/interthread.git)
/// 

#[proc_macro_error::proc_macro_error]
#[proc_macro_attribute]
pub fn group( _attr: proc_macro::TokenStream, _item: proc_macro::TokenStream ) -> proc_macro::TokenStream {
    let msg = "The \"group\" macro is currently under development and is not yet implemented in the `interthread` crate.";
    proc_macro_error::abort!( proc_macro2::Span::call_site(),msg );
}






