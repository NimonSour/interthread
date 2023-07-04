
use tokio::time::{sleep,Duration};
use tokio::sync::oneshot::{self,Sender};
use std::sync::{Arc,Mutex};
pub struct MyActor(Arc<Mutex<u32>>);
// we use argument `id`
#[interthread::actor(channel=2,lib="tokio",id=true)] 
impl MyActor {

    pub fn new() -> Self {Self(Arc::new(Mutex::new(0)))}

    pub async fn init_actor_increment(&self,val:usize, sender: Sender<MyActorLive>){
        
        // clone the value of Actor 
        let value = Arc::clone(&self.0);
        tokio::spawn(async move {
            // I prefer to initialize the actors 
            // like this, since they are competing 
            // with each other to obtain the unique ID.

            // commentout the "sleep" statement
            // it will work anyway
            sleep(Duration::from_millis(val as u64)).await;

            //create actor
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

#[tokio::main]
async fn main(){
    let mut handles = Vec::new();
    let actor = MyActorLive::new();
    
  
    for i in 0..1000 {
        let act_clone = actor.clone();

        let handle = tokio::spawn(async move {

            let (send,recv) = oneshot::channel();
            
            // we want to receive an instance of 
            // new actor 
            // we send channel's "sender"    
            act_clone.init_actor_increment(i, send).await;

            // awaiting on reciver for new actor 
            recv.await
        });
        handles.push(handle);
    }
    
    let mut actors = Vec::new(); 
    // receiving 
    for handle in handles {
        let act = 
        handle.await
              .expect("Task Fails")
              .expect("Receiver Fails");
        
        actors.push(act);
    }

    println!("Total tasks - {}", actor.get_value().await);
    println!("actors.len() -> {}", actors.len());
    

    // actors can be sorted by
    // `time of initiation' 
    // allowing for ordering 
    actors.sort();

    assert_eq!(actors[0] < actors[1],true); 
    assert_eq!(actors[121] < actors[122],true); 
    assert_eq!(actors[998] < actors[999],true); 


    // check if they have unic Ids 
    for i in (actors.len() - 1) ..0{
        let target = actors.remove(i);
        if actors.iter().any(move |x| *x == target){
            println!("ActorModel Id's are not identical")
        }
    }
    eprintln!(" * end of program * ");

}