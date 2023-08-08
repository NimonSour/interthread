pub struct MyActor {
    value: i8,
}
#[interthread::actor(lib = "smol", channel = 2)]
impl MyActor {
    pub fn new(v: i8) -> Self {
        Self { value: v }
    }
    pub fn increment(&mut self) {
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
#[derive(Debug)]
pub enum MyActorScript {
    Increment {},
    AddNumber {
        input: (i8),
        output: oneshot::Sender<i8>,
    },
    GetValue {
        output: oneshot::Sender<i8>,
    },
}
impl MyActorScript {
    pub fn direct(self, actor: &mut MyActor) {
        match self {
            MyActorScript::Increment {} => {
                actor.increment();
            }
            MyActorScript::AddNumber {
                input: (num),
                output: send,
            } => {
                send.send(actor.add_number(num))
                    .expect("'my_actor_direct.send'. Channel closed");
            }
            MyActorScript::GetValue { output: send } => {
                send.send(actor.get_value())
                    .expect("'my_actor_direct.send'. Channel closed");
            }
        }
    }
    pub async fn play(receiver: async_channel::Receiver<MyActorScript>, mut actor: MyActor) {
        while let Ok(msg) = receiver.recv().await {
            msg.direct(&mut actor);
        }
        eprintln!("MyActor end of life ...");
    }
}
#[derive(Clone, Debug)]
pub struct MyActorLive {
    sender: async_channel::Sender<MyActorScript>,
}
impl MyActorLive {
    pub async fn increment(&mut self) {
        let msg = MyActorScript::Increment {};
        let _ = self.sender.send(msg).await;
    }
    pub async fn add_number(&mut self, num: i8) -> i8 {
        let (send, recv) = oneshot::channel();
        let msg = MyActorScript::AddNumber {
            input: (num),
            output: send,
        };
        let _ = self.sender.send(msg).await;
        recv.await
            .expect("'MyActorLive::method.recv'. Channel is closed!")
    }
    pub async fn get_value(&self) -> i8 {
        let (send, recv) = oneshot::channel();
        let msg = MyActorScript::GetValue { output: send };
        let _ = self.sender.send(msg).await;
        recv.await
            .expect("'MyActorLive::method.recv'. Channel is closed!")
    }
    pub fn new(v: i8) -> Self {
        let (sender, receiver) = async_channel::bounded(2);
        let actor = MyActor::new(v);
        let actor_live = Self { sender };
        smol::spawn(MyActorScript::play(receiver, actor)).detach();
        actor_live
    }
}
fn main() {
    let actor = MyActorLive::new(5);
    let mut actor_a = actor.clone();
    let mut actor_b = actor.clone();
    let handle_a = std::thread::spawn(move || {
        actor_a.increment();
    });
    let handle_b = std::thread::spawn(move || {
        actor_b.add_number(5);
    });
    let _ = handle_a.join();
    let _ = handle_b.join();
}
