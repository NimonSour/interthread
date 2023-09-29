


pub struct AnyOtherType;


pub struct Aa(pub u8);
pub struct Bb(pub u8);
pub struct Cc(pub u8);


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

pub struct MyActor {
    value: i8,
}
// #[interthread::actor(edit(live(imp(get_value))))]  V

// #[interthread::actor(edit(live(imp(increment))))]   
// #[interthread::actor(file = "examples/intro_group.rs", edit(live(def, imp),script(def(file))))]  
#[interthread::actor(file = "examples/intro_group.rs", edit(live(def, imp),script(def(file))))]  


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



// #[interthread::example(path = "examples/intro_group.rs")]
pub fn main(){

}