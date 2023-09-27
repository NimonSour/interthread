


pub struct AnyOtherType;


struct Aa(pub u8);
struct Bb(pub u8);
struct Cc(pub u8);


pub struct AaBbCc {
    pub a: Aa,
    pub b: Bb, 
    pub c: Cc,
    any: AnyOtherType,
}

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


Note: the `file` ident inside the `edit` argument. 

#[interthread::group(
    file="path/to/abc.rs",
    edit 
)]

The above `edit` argument triggers the whole model to be written.

*/




// impl AaBbCc {

//     pub fn new(a:u8, b:u8, c:u8) -> Self {
//         Self { a: Aa(a), b: Bb(b), c: Cc(c) }
//     }

// }



// pub fn main(){

// }