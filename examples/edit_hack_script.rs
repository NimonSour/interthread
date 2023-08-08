
use std::sync::mpsc;
use interthread::actor;
 
pub struct MyActor {
    value: i8,
}
//  #[actor(channel=2, edit(play))]
#[actor(channel=2, edit(script(imp(play))))]
impl MyActor {

    pub fn new( value: i8 ) -> Self {
        Self{value}
    }
    pub fn increment(&mut self) -> i8{
        self.value += 1;
        self.value
    }
    pub fn play_get_counter(&self)-> Option<u32>{
        None
    }

}


// incapsulate the matching block
// inside `Script` impl block
// where the `direct`ing is happening
// to keep our `play` function nice
// and tidy 
impl MyActorScript {
    pub fn custom_direct(self,
           actor: &mut MyActor, 
           counter: &u32 ){

        // the same mathing block 
        // as in above example    
        match self {
            MyActorScript::PlayGetCounter { output  } =>
            { let _ = output.send(Some(counter.clone()));},
            
            // else as usual 
            msg => { msg.direct(actor); }
        }
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
        
        // nice and tidy while loop ready
        // for more wild things to happen
        while let Ok(msg) = receiver.recv() {
            
            // this is the invocation
            // of MyActorScript.custom_direct()
            msg.custom_direct(&mut actor, &call_counter);
    
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

        if let Some(counter) = my_act.play_get_counter(){

            println!("This call never riched the `Actor`, 
            it returns the value of total calls from the 
            `play` function ,call_counter = {:?}",counter);

            assert_eq!(counter, 2);
        }
    });
    let _ = handle_c.join();

}