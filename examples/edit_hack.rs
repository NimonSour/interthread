
use std::sync::mpsc;
use interthread::actor;
 
pub struct MyActor {
    value: i8,
}

// this is initial macro 
// #[actor(channel=2,file="src/main.rs",edit(script(imp(play))))]
// will change to 
#[actor(channel=2, edit(script(imp(play))))]

impl MyActor {

    pub fn new( value: i8 ) -> Self {
        Self{value}
    }
    pub fn increment(&mut self) -> i8{
        self.value += 1;
        self.value
    }
    // it's safe to hack the macro in this way
    // having `&self` as receiver along  with
    // other things creates a `Script` variant  
    // We'll catch it in `play` function
    pub fn play_get_counter(&self)-> Option<u32>{
        None
    }

}

// we have the code of `play` component
// using `edit` in conjuction with `file`
// Initiated By  : #[actor(channel=2,file="src/main.rs",edit(script(imp(play))))]  
impl MyActorScript {

    pub fn play( 
         receiver: mpsc::Receiver<MyActorScript>,
        mut actor: MyActor) {
        // set a custom variable 
        let mut call_counter = 0;
    
        while let Ok(msg) = receiver.recv() {
    
            // match incoming msgs
            // for `play_get_counter` variant
            match msg {
                // you don't have to remember the 
                // the name of the `Script` variant 
                // your text editor does it for you
                // so just choose the variant
                MyActorScript::PlayGetCounter { output  } =>
                { let _ = output.send(Some(call_counter));},
                
                // else as usual 
                _ => { msg.direct(&mut actor); }
            }
            call_counter += 1;
        }
        eprintln!("the end");
    }
}


fn main() {

    let my_act = MyActorLive::new(0);
    let mut act_a = my_act.clone();
    let mut act_b = my_act.clone();

    let handle_a = std::thread::spawn(move || {
        act_a.increment();
    });
    let handle_b = std::thread::spawn(move || {
        act_b.increment();
    });
    
    let _ = handle_a.join();
    let _ = handle_b.join();


    let handle_c = std::thread::spawn(move || {

        // as usual we invoke a method on `live` instance
        // which has the same name as on the Actor object
        // but 
        if let Some(counter) = my_act.play_get_counter(){

            println!("This call never riched the `Actor`, 
            it returns the value of total calls from the 
            `play` function ,call_counter = {:?}",counter);

            assert_eq!(counter, 2);
        }
    });
    let _ = handle_c.join();

}