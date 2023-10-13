pub struct MyActor {
    value: i8,
}
//  #[interthread::example(path="examples/visibility.rs")]  
#[interthread::actor(channel=2,assoc)] // <-  this is it 
impl MyActor {

    pub(crate) fn new( v: i8 ) -> Self {
       Self { value: v } 
    }
    pub fn increment(&mut self) {
        self.value += 1;
    }
    pub(self) fn add_number(&mut self, num: i8) -> i8 {
        self.value += num;
        self.value
    }
    pub fn get_value(&self) -> i8 {
        self.value
    }
    pub fn is_even(v:u8) -> bool {
        v % 2 == 0
    }
}

fn main(){

}