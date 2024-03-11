
pub struct Actor {
    str: String,
}

// writes to file when 'edit' is used
// in conjuction with 'file' argument

#[interthread::actor(channel=2,
    edit(live(imp(concat))))
]
impl Actor 

{
    pub fn new() -> Self {
        Actor { 
            str: String::new(), 
        }
    }

    pub fn concat(&mut self, s: String) {
        self.str += &s;
    }

    pub fn get_value(&self) -> String {
        self.str.clone()
    }
}


//++++++++++++++++++[ Interthread  Write to File ]+++++++++++++++++//
// Object Name   : MaunActor  
// Initiated By  : #[interthread::actor(channel=2,file="path/to/this/file.rs",edit(live(imp(file(concat)))))]  


impl ActorLive {
    // pub fn concat(&mut self, s: String) {
    pub fn concat<S:ToString>(&mut self, s: S) {
        let msg = ActorScript::Concat {
            // input: (s)
            input: (s.to_string()),
        };
        let _ = self
            .sender
            .send(msg)
            .expect("'MaunActorLive::method.send'. Channel is closed!");
    }
}

// *///.............[ Interthread  End of Write  ].................//



fn main() {

    let act = ActorLive::new();
    
    let mut one = act.clone();
    let mut two = act.clone();
    let mut thr = act.clone();

    let one_h = std::thread::spawn( move || { 
        one.concat("I can handle any".to_string());
    });
    let _ = one_h.join();

    let two_h = std::thread::spawn( move || {
        two.concat(" 'ToString' - ");
    });
    let _ = two_h.join();

    let thr_h = std::thread::spawn( move || {
        thr.concat('😀');
    });
    let _ = thr_h.join();

    
    assert_eq!(
        act.get_value(), 
        "I can handle any 'ToString' - 😀".to_string()
    );
}