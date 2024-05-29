
use tokio::sync::oneshot::Sender;
use std::sync::{Arc,Mutex};
pub struct MyActor(Arc<Mutex<u32>>);


// we use argument `debut`
#[interthread::actor(channel=1,lib="tokio",debut)]
impl MyActor {

    pub fn new() -> Self {Self(Arc::new(Mutex::new(0)))}

    pub async fn init_actor_increment(&self,_val:usize, sender: Sender<MyActorLive>){
        
        // clone the value of Actor 
        let value = Arc::clone(&self.0);
        tokio::spawn(async move {
            // initiate new actor 
            let actor = MyActorLive::new();

            // send actor
            let _ = sender.send(actor);

            // increment the value
            let mut guard = value.lock().unwrap();
            *guard += 1;
        });
    }
    pub fn get_value(&self) -> u32 {
        self.0.lock().unwrap().clone()
    }
}

// #[interthread::example(main(path="examples/send_actor_tokio.rs"))]
#[tokio::main]
async fn main() {
    
    let mut handles = Vec::new();
    let actor = MyActorLive::new();
    for i in 0..1000 {
        let act_clone = actor.clone();
        let handle = tokio::spawn(async move {
            let (send, recv) = tokio::sync::oneshot::channel::<MyActorLive>();
            act_clone.init_actor_increment(i, send).await;
            recv.await
        });
        handles.push(handle);
    }

    let mut actors = Vec::new();
    for handle in handles {
        let act = handle.await.expect("Task Fails").expect("Receiver Fails");
        actors.push(act);
    }


    println!("Total tasks - {}", actor.get_value(). await);
    println!("actors.len() -> {}", actors.len());
    actors.sort();
    assert_eq!(actors[0] < actors[1], true);
    assert_eq!(actors[121] < actors[122], true);
    assert_eq!(actors[998] < actors[999], true);

    for i in (0..actors.len()).rev() {

        let target = actors.remove(i);
        if actors.iter().any(move |x| *x == target) {
            panic!("Actor Model Id's are not unique !")
        }
    }
    eprintln!(" * end of program * ");
}