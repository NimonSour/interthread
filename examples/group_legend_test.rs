// This is a test file 

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


#[interthread::group( file= "examples/group_legend_test.rs",debut(legend))]
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

pub fn main() {

    let h = std::thread::spawn( || {
        let mut group = AaBbGroupLive::new();
        group.inter_set_name("Zombie"); 
        group.add(1);
        group.a.add(10);
        group.b.add(100);
    });
    
    let _ = h.join();
    let mut old_group = AaBbGroupLive::try_old("Zombie").unwrap();

    assert_eq!("Zombie".to_string(), old_group.inter_get_name());
    assert_eq!((11, 101), old_group.get_value());

}