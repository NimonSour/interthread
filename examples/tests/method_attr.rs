use interthread::actor;


pub struct ActorA;
#[actor]
impl ActorA{
    #[doc="Comment on ActorA::new"]
    pub fn new() -> Self{ Self{} }
}

pub struct ActorB;
#[actor(name="ActorBB")]
#[actor(file="examples/tests/method_attr.rs", edit(script,live(imp(new))) )]
impl ActorB{
    #[doc="Comment on ActorB::new"]
    pub fn new() -> Self{ Self{} }
}

//++++++++++++++++++[ Interthread  Write to File ]+++++++++++++++++//
// Object Name   : ActorB  
// Initiated By  : #[actor(file="examples/tests/method_attr.rs",edit(file(script,live(imp(new)))))]  



pub enum ActorBScript {}
impl ActorBScript {
    pub fn direct(self, actor: &mut ActorB) {
        match self {}
    }
    pub fn play(receiver: std::sync::mpsc::Receiver<ActorBScript>, mut actor: ActorB) {
        while let std::result::Result::Ok(msg) = receiver.recv() {
            msg.direct(&mut actor);
        }
        eprintln!("ActorB the end ...");
    }
}
impl std::fmt::Debug for ActorBScript {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ActorBScript")
    }
}
impl ActorBLive {
    ///Comment on ActorB::new
    pub fn new() -> Self {
        let actor = ActorB::new();
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || { ActorBScript::play(receiver, actor) });
        Self { sender }
    }
}

// *///.............[ Interthread  End of Write  ].................//




