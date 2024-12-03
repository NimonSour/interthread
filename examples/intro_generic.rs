
pub struct MaunActor<T> {
    value: T,
}


#[interthread::actor]
impl <T> MaunActor <T>
where
    T: std::ops::AddAssign + Copy,
{
    pub fn new(v: T) -> Self {
        Self { value: v }
    }
    pub fn add_number<I: Into<T>>(&mut self, num: I) {
        self.value += num.into();
    }
    pub fn get_value(&self) -> T {
        self.value
    }
}

//  #[interthread::example(main,path="examples/intro_generic.rs")] 
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

    assert_eq!(actor.get_value(), 2_u128);
    // println!("value - {}",actor.get_value());
}