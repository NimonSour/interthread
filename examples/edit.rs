

 
use std::sync::mpsc;
use interthread::actor;
 
pub struct MyActor {
    value: i8,
}

#[actor(channel=2, edit(script(imp(play))))]

impl MyActor {

    pub fn new( value: i8 ) -> Self {
        Self{value}
    }
    pub fn increment(&mut self) -> i8{
        self.value += 1;
        self.value
    }
}

// manually create "play" function 
// use `example` macro to copy paste
// `play`'s body 
impl MyActorScript {

    pub fn play( 
         receiver: mpsc::Receiver<MyActorScript>,
        mut actor: MyActor) {
        // set a custom variable 
        let mut call_counter = 0;
        while let Ok(msg) = receiver.recv() {
            // do something 
            // like 
            println!("Value of call_counter = {}",call_counter);
    
            // `direct` as usual 
            msg.direct(&mut actor);
    
            // increment the counter as well
            call_counter += 1;
        }
        eprintln!(" the end ");
    }
}


fn main() {

    let my_act       = MyActorLive::new(0);
    let mut act_a = my_act.clone();
    

    let handle_a = std::thread::spawn(move || -> i8{
        act_a.increment()
    });

    let value = handle_a.join().unwrap();
    
    assert_eq!(value, 1);

    // and will print the value of 
    // call_counter
}