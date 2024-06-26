
struct Actor {
    live: ActorLive,
}


// we want method "new" of Live to be excluded from 
// code generation and to be printed on the file 
// so we could modify it 
// #[interthread::actor(file="examples/self_live_actor.rs", edit(live(imp( file(new) ))))]
// right after the desired part of code is added to
// the file our initial macro will change to 
#[interthread::actor(file="examples/self_live_actor.rs", edit(live(imp( new ))))]

impl Actor {

    pub fn new (live: ActorLive ) -> Self { Self{live} }


    // a)
    // this method will triger an endless loop
    
    /*
    
    pub fn call(&mut self, msg: String){
        eprintln!("{}",&msg);
        self.live.call(msg);
    }
    // */


    // b)
    // this method will work like Ping - Pong 

    
    pub fn call(&mut self, msg: String, back: bool ){
        if back {
            eprintln!("Ping[{}]",&msg);
            self.live.call(msg,false);
        } else {
            eprintln!("Pong[{}]",&msg);
        };
    }
    // */


    // c)
    // this method will end up in so called "deadlock"
    // for any method that returns a type 
    
    /* 
    pub fn call(&mut self, msg: String ) -> bool {
        // the thread is blocked here 
        // waiting for return type 
        self.live.call(msg);
        true 
    }
    // */

    // d
    // TODO 
    // add a case when the lock 
    // can be avoided using `interact`
    // Actor needs a field to hold oneshot::Receiver


}





impl ActorLive {
    // the Actor needs an instance of ActorLive
    // we will change the method signature from 
    // pub fn new(live: ActorLive) -> Self {
    // to    
    pub fn new() -> Self {
        // we start a new channel
        let (sender, receiver) = std::sync::mpsc::channel();
        // create an ActorLive instance
        let actor_live  = Self { sender }; 
        // create an Actor instance
        let actor = Actor::new(actor_live.clone());
        // nothing to change here 
        std::thread::spawn(move || { ActorScript::play(receiver, actor) });
        // returning Self instance
        actor_live
    }
}


// original code generated by the `actor`
// as example of the method we've just modified 

//++++++++++++++++++[ Interthread  Write to File ]+++++++++++++++++//
// Object Name   : Actor  
// Initiated By  : #[interthread::actor(file="examples/self_live_actor.rs",edit(live(imp(file(new)))))]  


/* 
impl ActorLive {
    pub fn new(live: ActorLive) -> Self {
        let actor = Actor::new(live);
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || { ActorScript::play(receiver, actor) });
        Self { sender }
    }
}

// *///.............[ Interthread  End of Write  ].................//





fn main() {

    let mut actor = ActorLive::new();

    actor.call("Hi".to_string(),true);
    


    // comment and uncomment other "call" 
    // methods in Actor impl and below
    // to experience "deadlock" and "endless loop"
    
    /*
    actor.call("Hi".to_string());
    // */
    


    // we need to wait for execution of code in threads 
    std::thread::sleep( std::time::Duration::from_secs(1));
}