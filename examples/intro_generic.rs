
// BAD GENERIC ACTOR
/*
pub struct Actor<T> {
    value: T,
}
#[interthread::actor(channel=2)]
impl<T> Actor<T>
where
    T: std::ops::AddAssign + Copy,
{
    pub fn new(v: T) -> Self {
        Self { value: v }
    }
    pub fn add_number<I:Into<T>>(&mut self, num: I) {
        self.value += num.into();
    }
    pub fn get_value(&self) -> T {
        self.value
    }
}
*/



// GOOD GENERIC ACTOR
pub struct Actor<T> {
    value: T,
}
#[interthread::actor(channel=2,
    edit( live( imp( add_number))))]
impl<T> Actor<T>
where
    T: std::ops::AddAssign + Copy,
{
    pub fn new(v: T) -> Self {
        Self { value: v }
    }
    pub fn add_number(&mut self, num: T) {
        self.value += num.into();
    }
    pub fn get_value(&self) -> T {
        self.value
    }
}

//++++++++++++++++++[ Interthread  Write to File ]+++++++++++++++++//
// Object Name   : Actor  
// Initiated By  : #[interthread::actor(channel=2,file="examples/intro_generic.rs",edit(live(imp(add_number))))]  


impl<T> ActorLive<T>
where
    T: std::ops::AddAssign + Copy + Send + Sync + 'static,
{
    pub fn add_number<I: Into<T>>(&mut self, num:I) {
        let msg = ActorScript::AddNumber {
            input: (num.into()),
        };
        let _ = self
            .sender
            .send(msg)
            .expect("'ActorLive::method.send'. Channel is closed!");
    }
}

// *///.............[ Interthread  End of Write  ].................//



// #[interthread::example(main(path="examples/intro_generic.rs"))]

fn main() {

    let actor = ActorLive::new(0u128);

    let mut actor_a = actor.clone();
    let mut actor_b = actor.clone();

    let handle_a = std::thread::spawn( move || { 
    actor_a.add_number(1_u8);
    });

    let handle_b = std::thread::spawn( move || {
    actor_b.add_number(1_u64);
    });

    let _ = handle_a.join();
    let _ = handle_b.join();

    assert_eq!(actor.get_value(), 2_u128)
}

