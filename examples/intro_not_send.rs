use std::cell::RefCell;
use std::rc::Rc;
// both Rc and RefCell are !Send
pub struct Actor {
    state: Rc<RefCell<u64>>,
}

#[interthread::actor(ty="!Send")]
impl Actor {

    pub fn new<U:Into<u64>>(value: U) -> Self {
        Actor {
            state: Rc::new(RefCell::new(value.into())),
        }
    }

    pub fn update<U>(&self, value: U)
    where U: Into<u64>,
    {
        *self.state.borrow_mut() += value.into();
    }

    pub fn get(&self) -> u64
    {
        self.state.borrow().clone()
    }
}


//  #[interthread::example(main,path="examples/intro_not_send.rs")] 
fn main() {

    let act = ActorLive::new(0u8);
    act.update(u8::MAX);
    // println!("Actor.state = {}",act.get());
    assert_eq!(act.get(), u8::MAX as u64);
}