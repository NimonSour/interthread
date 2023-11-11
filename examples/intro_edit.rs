

 

pub struct MyActor(u8);

#[interthread::actor(
    channel=2,
    file="examples/intro_edit.rs",
    edit(live(imp(increment)))
)]  

impl MyActor {

    pub fn new() -> Self {Self(0)}

    pub fn increment(&mut self){
        self.0 += 1;
    }
}

//++++++++++++++++++[ Interthread  Write to File ]+++++++++++++++++//
// Object Name   : MyActor  
// Initiated By  : #[interthread::actor(channel=2,file="examples/intro_edit.rs",edit(live(imp(file(increment)))))]  


impl MyActorLive {
    pub fn increment(&mut self) {
        let msg = MyActorScript::Increment {};
        let _ = self
            .sender
            .send(msg)
            .expect("'MyActorLive::method.send'. Channel is closed!");
    }
}

// *///.............[ Interthread  End of Write  ].................//


fn main(){}




