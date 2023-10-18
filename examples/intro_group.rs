



/*
For a struct

pub struct AaBbCc {
    pub a: Aa,
    pub b: Bb, 
    c: Cc,
}

available `edit` arguments are:

#[interthread::group(
    file="path/to/abc.rs",
    edit(
        file   <-  !!!

        script( def, imp(..), trt(..) ),
        live(   def, imp(..), trt(..) ),

        a( script( def, imp(..), trt(..) ),
           live(   def, imp(..), trt(..) ), 
         ),

        b( script( def, imp(..), trt(..) ),
           live(   def, imp(..), trt(..) ), 
         )
    )
)]

#[interthread::group(
    file="path/to/abc.rs",
    edit(
        file   

        script( def, imp(..), trt(..) ),
        live(   def, imp(..), trt(..) ),

        Self::a( script( def, imp(..), trt(..) ),
           live(   def, imp(..), trt(..) ), 
         ),

        Self::b( script( def, imp(..), trt(..) ),
           live(   def, imp(..), trt(..) ), 
         )
    )
)]


Note: the `file` ident inside the `edit` argument. 

#[interthread::group(
    file="path/to/abc.rs",
    edit 
)]

The above `edit` argument triggers the whole model to be written.


#[interthread::group(
    file = "path/to/abc.rs",
    path = ( a("path/to/a.rs"), b("path/to/b.rs") ),

)]

#[interthread::actor(
    file="path/to/abc.rs",
    edit( file(
            script( def, imp(..), trt(..) ),
            live(   def, imp(..), trt(..) ),
        )
    )
)]



//////////////

1) About 'file' argument.
    Actor ) 'file' argument migrates into 'edit' argument.
           When the argument is defined it works as it should 
           editing (writing)to the file. If an additional 
           file-active argument is defined anywere in the scope of 
           'edit' list, than to the file will be written just 
           arguments defined in 'file-active' scope.

    Group ) File argument is defined outside of 'edit' argument.
            To include it inside the 'edit' argument to enforce rules of 
            'file-active' just include a 'file-active' (`file`) where normally
            will use a file key value inside the  'Actor' 'edit' argument.


2) Examples of usage : 
    
    a) write all 

        actor)

        group) 

Examples:


#[interthread::group(
    file="path/to/abc.rs",
    edit(
        file ,  

        script( def, imp, trt ),
        file(live(   def, imp, trt)),

        Self::a( script( def, imp(file(bla)), trt ),
           live(   file(def), imp, trt ), 
         ),

        Self::b( file(script( def, imp)),
           live(   def, imp, trt ), 
         )
    )
)]


#[interthread::group(
    file="path/to/abc.rs",
    edit(
        script( def, imp, trt ),
        live(   def, imp, trt),

        Self::a( script( def, imp(bla), trt ),
                   live( def, imp, trt ), 
         ),

        Self::b( script( def, imp),
                   live( def, imp, trt ), 
         )
    )
)]

#[interthread::group(
    file="path/to/abc.rs",
    edit(

        self::edit(live)
        a::edit( script( def, imp(bla), trt ),
                   live( def, imp, trt ), 
        ),

        b::edit( script( def, imp),
                   live( def, imp, trt ), 
        ),
    )
)]

edit (file ) == edit( self::edit(file),a::edit(file),b::edit(file))


1) Check if all inputs are path ending in 'edit'.
Error message is : Unlike the `actor`s edit 
group edit takes a list of 'edit' lists.

a) catch the first case (`edit(file)`) 

2) Somwhere check if all identifiers are valid.

3) After it will need a check if all are ..::edit(file),
if so delete the macro else ( if there is a field `c`) the
macro edit will look like so  `edit( self::edit, a::edit, b::edit)


*/




// impl AaBbCc {

//     pub fn new(a:u8, b:u8, c:u8) -> Self {
//         Self { a: Aa(a), b: Bb(b), c: Cc(c) }
//     }

// }

pub struct MyActor {
    value: i8,
}

// #[interthread::actor(edit(live(imp(get_value))))]  V

// #[interthread::actor(edit(live(imp(increment))))]   
// #[interthread::actor(file = "examples/intro_group.rs", edit(live(def, imp),script(def(file))))]  
// #[interthread ::
// actor(file = "examples/intro_group.rs", edit(live(def, imp), script(def)))] 
// #[interthread :: actor(file = "examples/intro_group.rs", edit(file))]//edit(script(file(imp))))]
// #[interthread::actor( edit(file(script), 
// live(file(imp(add_number))))
// )] 
/////////////////
// / #[interthread::actor(file="examples/intro_group.rs",edit(file))] 
// #[interthread::actor(file="examples/intro_group.rs",edit(file(script),live))] 

// --------------------------
// #[interthread::actor(file="examples/intro_group.rs",edit)] // default edit
// #[interthread::actor(file="examples/intro_group.rs",edit(script))] // One argument not 'file'
// #[interthread::actor(file="examples/intro_group.rs",edit(file))] // One argument 'file'
// #[interthread::actor(file="examples/intro_group.rs",edit(file(script)))] // One argument 'file(..)'
// #[interthread::actor(file="examples/intro_group.rs",edit(live,file(script)))] // Many arguments not 'file'
// #[interthread::actor(file="examples/intro_group.rs",edit(file(script),live))] // Many arguments 'file'
// #[interthread::actor(file="examples/intro_group.rs",edit(file(script,live)))] // One argument 'file(..)'
// #[interthread::actor(file="examples/intro_group.rs",edit(file,live))] //Expected a list!

// --------------------------

// #[interthread::actor(file="examples/intro_group.rs",edit(file(script,live)))] //just script
// #[interthread::actor(file="examples/intro_group.rs",edit(file(live,script)))] // just live
// #[interthread::actor(file="examples/intro_group.rs",edit(live,script))] // just live
// #[interthread::actor(file="examples/intro_group.rs",edit(script,live))] // just script
// #[interthread::actor(file="examples/intro_group.rs",edit(file(live),script))] // just live
// #[interthread::actor(file="examples/intro_group.rs",edit(file(live(imp)),script))] // outside for loop 'sol(..)'
// #[interthread::actor(file="examples/intro_group.rs",edit(live(file(imp)),script))] // inside for loop 'file(..)'
// #[interthread::actor(file="examples/intro_group.rs",edit(live(file(imp)),script))] // inside for loop 'file(..)'
// #[interthread::actor(file="examples/intro_group.rs",edit(live(file,imp),script))] // Expected a list!

// --------------------------
// #[interthread::actor(file="examples/intro_group.rs",edit(file(live),live(file(imp))))] // <-- trable
// need a new message explaining that 
// while multiple `file` options are allowed 
// they can not be nested
// #[interthread::actor(file="examples/intro_group.rs",edit(file(live,file(script))))] // Option has already been declared.

// #[interthread::actor(file="examples/intro_group.rs",edit(live,live(imp)))] // <-- trable

// #[interthread::actor(file="examples/intro_group.rs",edit(live,file(live(file(imp)))))] // <-- trable
// #[interthread::actor(file="examples/intro_group.rs",edit(file(live(file(imp))),live))] // <-- trable
// #[interthread::actor(file="examples/intro_group.rs",edit( live(file(imp(file(increment)))),live))] // <-- trable
// #[interthread::actor(file="examples/intro_group.rs",edit( live(imp(file))))] // <-- trable

 
// #[interthread::group( edit(file(script(imp), a::edit(live,script(def)))))]
// #[interthread::group( edit(a::edit(live,script(def))))]
// #[interthread::group( edit(
//     file(

//         a::edit(live,script(def)),
//     ),

//     self::edit(file(live(imp))),
//     c::edit(file)

// ))]
// #[interthread::actor(edit(live),assoc,name = "si")]

impl MyActor {
    pub fn new(v: i8) -> Self {
        Self { value: v }
    }
    pub fn increment(&mut self) {
        self.value += 1;
    }
    pub fn add_number(&mut self, num: i8) -> i8 {
        self.value += num;
        self.value
    }
    pub fn get_value(&self) -> i8 {
        self.value
    }
}





//_____________________________________________________________
pub struct AnyOtherType;

pub struct Aa(pub u8);

impl Aa {
    pub fn add(&mut self, v: u8){
        self.0 += v;
    }
}
pub struct Bb(pub u8);

impl Bb {
    pub fn add(&mut self, v: u8){
        self.0 += v;
    }
}
pub struct Cc(pub u8);
impl Cc {
    pub fn add(&mut self, v: u8){
        self.0 += v;
    }
}

pub struct AaBbCc {
    pub a: Aa,
    pub b: Bb, 
    pub c: Cc,
    any: AnyOtherType,
}


/*

AaScriptGroup {
    Add{input: u8}
}

BbScriptGroup {
    Add{input: u8}
}

CcScriptGroup {
    Add{input: u8}
}


AaBbCcGroupScript {

    AddToA{ input:u8, send:Sender },
    A(AaScriptGroup),
    B(BbScriptGroup),
    C(CcScriptGroup),

}

direct ( )
*/

// #[interthread::group( file= "examples/intro_group.rs" )]
impl AaBbCc {

    pub fn new( ) -> Self {
        let a = Aa(0);
        let b = Bb(0);
        let c = Cc(0);
        let any = AnyOtherType;

        Self{ a,b,c,any }
    }
    pub fn add_to_a(&mut self, v:u8){
        self.a.0 += v;
    }
}



// #[interthread::example(path = "examples/intro_group.rs")]

pub fn main(){


}