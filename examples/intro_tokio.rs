
pub struct MyActor {
    value: i8,
}

#[interthread::actor(channel=2,lib="tokio",debut)] 
impl MyActor {

    pub fn new( v: i8 ) -> Self {
       Self { value: v } 
    }
    // if the "lib" is defined
    // object methods can be "async" 
    pub async fn increment(&mut self) {
        self.value += 1;
    }
    pub fn add_number(&mut self, num: i8) -> i8 {
        self.value += num;
        self.value
    }
    pub fn get_value(&self) -> i8 {
        self.value
    }

}

//  #[interthread::example(main(path="examples/intro_tokio.rs"))]  

#[tokio::main]
async fn main() {

    let actor = MyActorLive::new(5);

    let mut actor_a = actor.clone();
    let mut actor_b = actor.clone();

    let handle_a = tokio::spawn( async move { 
    actor_a.increment().await;
    });

    let handle_b = tokio::spawn( async move {
    actor_b.add_number(5).await
    });

    let _  = handle_a.await;
    let hb = handle_b.await.unwrap();

    // hb = 10 or 11
    assert!(hb >= 10);

    assert_eq!(actor.get_value().await, 11);
}