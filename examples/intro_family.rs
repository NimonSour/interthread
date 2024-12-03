
pub struct Actor {
    value: u16,
}

#[interthread::family( show,
    actor( first_name = "User" ,show, include(increment)),
    actor( first_name = "Admin" ,show, include(get_value)),
)] 

impl Actor {

  pub fn new( v: u16 ) -> Self {
     Self { value: v } 
  }
  pub fn increment(&mut self) {
      std::thread::sleep(std::time::Duration::from_millis(10));
      self.value += 1;
  }
  pub fn get_value(&self) -> u16 {
      self.value
  }

}


fn main() {

    let family = ActorFamily::new(0);

    let ActorFamily { mut user, admin } = family ;

    let _ = std::thread::spawn( move || { 

        for _ in 0..100 { user.increment(); }

    }).join(); 

    // after all the messages are sent
    // we access the actor via `admin`
    println!("processed messages = {}",admin.get_value());
}