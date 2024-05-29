pub struct MyActor(u8);


#[interthread::actor(show,debut(legend))] 
impl MyActor {

    pub fn new() -> Self { Self(0) }

    pub fn set(&mut self, v: u8){
        self.0 = v;
    } 

    pub fn get_value(&self) -> u8 {
        self.0
    }
}


// #[interthread::example(main(path="examples/intro_legend.rs"))]
fn main() {

    let h = std::thread::spawn( || {
        let mut act = MyActorLive::new();
        act.inter_set_name("Zombie"); 
        act.set(121);
    });
    
    let _ = h.join();

    let old_act = MyActorLive::try_old("Zombie").unwrap();

    assert_eq!("Zombie".to_string(), old_act.inter_get_name());
    assert_eq!(121u8, old_act.get_value());
}
