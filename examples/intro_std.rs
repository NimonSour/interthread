
pub struct MyActor {
    value: i8,
}

#[interthread::actor(channel=2)] // <-  this is it 
impl MyActor {

    pub fn new( v: i8 ) -> Self {
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
    pub fn send_value(&self,(s,m):(i8,u8)) -> i8 {
        self.value
    }
}


//  uncomment to see the generated code
//  #[interthread::example(path="src/main.rs")] 
//   in examples 
//  #[interthread::example(main(path="examples/intro_std.rs"))]  
fn main() {

    let actor = MyActorLive::new(5);

    let mut actor_a = actor.clone();
    let mut actor_b = actor.clone();

    let handle_a = std::thread::spawn( move || { 
    actor_a.increment();
    });

    let handle_b = std::thread::spawn( move || {
    actor_b.add_number(5);
    });

    let _ = handle_a.join();
    let _ = handle_b.join();

    assert_eq!(actor.get_value(), 11)
}