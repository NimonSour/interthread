use tokio::time::{sleep,Duration};
use std::sync::{Arc,Mutex};
pub struct MyActor(Arc<Mutex<u8>>);

#[interthread::actor(channel=2,lib="tokio")] 
impl MyActor {

    pub fn new() -> Self {Self(Arc::new(Mutex::new(0)))}

    pub async fn sleep_increment(&self) {
        // clone the value of Actor 
        let value = Arc::clone(&self.0);
        tokio::spawn(async move{
            // sleep for one second 
            sleep(Duration::from_secs(1)).await;
            // increment the value
            let mut guard = value.lock().unwrap();
            *guard += 1;
        });
    }
    pub fn get_value(&self) -> u8 {
        self.0.lock().unwrap().clone()
    }
}

#[tokio::main]
async fn main(){

    let actor = MyActorLive::new();

    for _ in 0..60 {
        let act_clone = actor.clone();

        let _ = tokio::spawn(async move {
            act_clone.sleep_increment().await;
        });
    }

    sleep(Duration::from_secs_f64(1.01)).await;
    println!("Total tasks - {}", actor.get_value().await);
}