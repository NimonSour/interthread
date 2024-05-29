// This is a test file 
pub struct Aa;
impl Aa {
    pub fn method1(&self){}
    pub fn method2(&self){}
    pub fn method3(&self){}
    pub fn method4(&self){}
    pub fn method5(&self){}
    pub fn method6(&self){}
    pub fn method7(&self){}
}

struct Bb;
impl Bb {
    pub fn method1(&self){}
    pub fn method2(&self){}
    pub fn method3(&self){}
    pub fn method4(&self){}
    pub fn method5(&self){}
    pub fn method6(&self){}
    pub fn method7(&self){}
}


struct Cc;
impl Cc {
    pub fn method1(&self){}
}

pub struct AaBb {
    pub a: Aa,
    pub(crate) b: Bb,
    pub(crate) c: Cc,
}

#[interthread::group( file= "examples/group_filter_test.rs", 
skip(c),
include(   
    a::include(method1,method2),
    self::include(new,method6)
),
exclude(b::exclude(
    method1,
    method2,
    method3,
    // method4,
    method5,
    method6,

)))]
impl AaBb {

    pub fn new( ) -> Self {
        let a = Aa;
        let b = Bb;
        let c = Cc;

        Self{a,b, c}
    }

    pub fn method1(&self){
        // just to keep the compiler happy
        self.b.method1();
        self.b.method2();
        self.b.method3();
        self.b.method5();
        self.b.method6();

        self.c.method1();
    } 

    pub fn method2(&self){}
    pub fn method3(&self){}
    pub fn method4(&self){}
    pub fn method5(&self){}
    pub fn method6(&self){}
    pub fn method7(&self){}
}


pub fn main(){
    let group = AaBbGroupLive::new();

    group.a.method1();
    group.a.method2();

    group.method6();

    group.b.method4();
    group.b.method7();

}