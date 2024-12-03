
pub struct MyActor;

// opt `interact`
#[interthread::actor(show,interact)] 
impl MyActor {

    pub fn new() -> Self { Self{} } 

    // oneshot channel can be accessed 
    // in methods that do not return 
    pub fn heavy_work(&self, inter_send: oneshot::Sender<u8>){

        std::thread::spawn(move||{
            // do some havy computation
            let _ = inter_send.send(5);
        });
        
    }
}

//  #[interthread::example(main,path="examples/intro_interact_spec.rs")] 
fn main () {

    let actor = MyActorLive::new();
    
    // hover over `heavy_work` to see the generated code
    let recv: oneshot::Receiver<u8> = actor.heavy_work(); 
    let int = recv.recv().unwrap();

    assert_eq!(5u8, int);
}