

pub struct MyActor(String);

// opt `interact`
#[interthread::actor(show, debut, interact)] 
impl MyActor {

    pub fn new() -> Self { Self("".to_string()) } 

    // we know there is a getter `inter_get_name`
    // using argument `inter_name` we imply
    // we want the return type of that getter
    pub fn set_value(&mut self, inter_name: String){
        self.0 = inter_name;
    }
    pub fn get_value(&self) -> String {
        self.0.clone()
    }
}

//  #[interthread::example(main,path="examples/intro_interact.rs")] 
fn main () {

    let mut actor = MyActorLive::new();

    // setting name for `live` instance
    actor.inter_set_name("cloud");

    // setting actor's value now
    // note the signature it's not the same  
    actor.set_value();

    assert_eq!("cloud".to_string(), actor.get_value());
}