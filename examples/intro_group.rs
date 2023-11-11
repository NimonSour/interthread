
pub struct Aa(u8);
impl Aa {
    pub fn add(&mut self, v: u8){
        self.0 += v;
    }
}

pub struct Bb(u8);
impl Bb {
    pub fn add(&mut self, v: u8){
        self.0 += v;
    }
}


pub struct AaBb {
    pub a: Aa,
    pub b: Bb,
}

#[interthread::group( file= "examples/intro_group.rs")]
impl AaBb {

    pub fn new( ) -> Self {
        let a = Aa(0);
        let b = Bb(0);

        Self{ a,b}
    }

    pub fn add(&mut self, v:u8){
        self.a.0 += v;
        self.b.0 += v;
    }

    pub fn get_value(&mut self) -> (u8,u8) {
        (self.a.0,self.b.0)
    }
}



// #[interthread::example(main(path = "examples/intro_group.rs"))]
pub fn main(){

    let mut group = AaBbGroupLive::new();

    group.add(1);
    assert_eq!((1,1),group.get_value());

    group.a.add(10);
    assert_eq!((11,1),group.get_value());

    group.b.add(100);
    assert_eq!((11,101),group.get_value());
}