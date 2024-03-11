
// BAD GENERIC ACTOR
/*
pub struct MaunActor<T> {
    value: T,
}
#[interthread::actor(channel=2)]
impl<T> MaunActor<T>
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
pub struct MaunActor<T> {
    value: T,
}
#[interthread::actor(channel=2,
    edit( live( imp( add_number))))]
impl<T> MaunActor<T>
where
    T: std::ops::AddAssign + Copy,
{
    pub fn new(v: T) -> Self {
        Self { value: v }
    }
    pub fn add_number(&mut self, num: T) {
        self.value += num;
    }
    pub fn get_value(&self) -> T {
        self.value
    }
}


//++++++++++++++++++[ Interthread  Write to File ]+++++++++++++++++//
// Object Name   : MaunActor  
// Initiated By  : #[interthread::actor(channel=2,file="path/to/this/file.rs",edit(live(imp(file(add_number)))))]  


impl<T> MaunActorLive<T>
where
    T: std::ops::AddAssign + Copy + Send + Sync + 'static,
{
    // pub fn add_number(&mut self, num: T) {
    pub fn add_number<I: Into<T>>(&mut self, num:I) {
        let msg = MaunActorScript::AddNumber {
            // input: (num),
            input: (num.into()),
        };
        let _ = self
            .sender
            .send(msg)
            .expect("'ActorLive::method.send'. Channel is closed!");
    }
}

// *///.............[ Interthread  End of Write  ].................//


fn main() {

    let actor = MaunActorLive::new(0u128);

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
