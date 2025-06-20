

// STD

#[test]
fn interact_simple(){
    /*
        Usually we need  "inter_send" and "inter_recv" if we 
        spawn a thread inside the method, but this is not 
        the reason for this test
    */

    pub struct Actor;

    #[interthread::actor(debut,interact)] 
    impl Actor {

        pub fn new() -> Self { Self{} } 
        
        pub fn needs_sender(&self, (a,b,..):(u8,u8,u8), inter_send: oneshot::Sender<String>){
            let _ = inter_send.send(format!("{a}-{b}"));
        }

        pub fn needs_receiver(&self, [ref a @ .., b]: [u8;4],inter_recv: oneshot::Receiver<oneshot::Sender<String>>){
            let sender = inter_recv.recv().unwrap();
            let _ = sender.send(format!("{a:?}-{b}"));
        }

        pub fn needs_name_and_state(&mut self, mut inter_name: String, inter_state: u8 ) -> String {
            inter_name += &format!("-{inter_state}");
            inter_name
        }
    }
    // impl a custom getter
    impl ActorLive {
        fn inter_get_state(&self)-> u8 { 121 }
    }

    let mut act = ActorLive::new();
   

    assert_eq!(
        act.needs_sender((1,2,3u8)).recv().unwrap(), 
        "1-2".to_string()
    );

    // call the method 
    let sender = act.needs_receiver([1,2,3,4u8]);
    let (send,recv) = oneshot::channel::<String>();
    // send the value( channel in this case )
    let _ = sender.send(send);

    assert_eq!(
        recv.recv().unwrap(),
        "[1, 2, 3]-4".to_string()
    );

    act.inter_set_name("Cloud");
    assert_eq!(
        act.needs_name_and_state(),
        "Cloud-121".to_string()
    )
}




