pub struct MyActor {
    value: i8,
}
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
pub enum MyActorScript {
    Increment {},
    AddNumber { num: i8, inter_send: oneshot::Sender<i8> },
    GetValue { inter_send: oneshot::Sender<i8> },
}
impl MyActorScript {
    pub fn direct(self, actor: &mut MyActor) {
        match self {
            MyActorScript::Increment {} => {
                actor.increment();
            }
            MyActorScript::AddNumber { num, inter_send } => {
                inter_send
                    .send(actor.add_number(num))
                    .unwrap_or_else(|_error| {
                        core::panic!(
                            "'MyActorScript::AddNumber.direct'. Sending on a closed channel."
                        )
                    });
            }
            MyActorScript::GetValue { inter_send } => {
                inter_send
                    .send(actor.get_value())
                    .unwrap_or_else(|_error| {
                        core::panic!(
                            "'MyActorScript::GetValue.direct'. Sending on a closed channel."
                        )
                    });
            }
        }
    }
    pub fn play(receiver: std::sync::mpsc::Receiver<MyActorScript>, mut actor: MyActor) {
        while let std::result::Result::Ok(msg) = receiver.recv() {
            msg.direct(&mut actor);
        }
        eprintln!("MyActor the end ...");
    }
}
impl std::fmt::Debug for MyActorScript {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MyActorScript::Increment { .. } => write!(f, "MyActorScript::Increment"),
            MyActorScript::AddNumber { .. } => write!(f, "MyActorScript::AddNumber"),
            MyActorScript::GetValue { .. } => write!(f, "MyActorScript::GetValue"),
        }
    }
}
#[derive(Clone)]
pub struct MyActorLive {
    sender: std::sync::mpsc::Sender<MyActorScript>,
}
impl MyActorLive {
    pub fn new(v: i8) -> Self {
        let actor = MyActor::new(v);
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || { MyActorScript::play(receiver, actor) });
        Self { sender }
    }
    pub fn increment(&mut self) {
        let msg = MyActorScript::Increment {};
        let _ = self
            .sender
            .send(msg)
            .expect("'MyActorLive::method.send'. Channel is closed!");
    }
    pub fn add_number(&mut self, num: i8) -> i8 {
        let (inter_send, inter_recv) = oneshot::channel();
        let msg = MyActorScript::AddNumber {
            num,
            inter_send,
        };
        let _ = self
            .sender
            .send(msg)
            .expect("'MyActorLive::method.send'. Channel is closed!");
        inter_recv
            .recv()
            .unwrap_or_else(|_error| {
                core::panic!("'MyActor::add_number' from inter_recv. Channel is closed!")
            })
    }
    pub fn get_value(&self) -> i8 {
        let (inter_send, inter_recv) = oneshot::channel();
        let msg = MyActorScript::GetValue {
            inter_send,
        };
        let _ = self
            .sender
            .send(msg)
            .expect("'MyActorLive::method.send'. Channel is closed!");
        inter_recv
            .recv()
            .unwrap_or_else(|_error| {
                core::panic!("'MyActor::get_value' from inter_recv. Channel is closed!")
            })
    }
}

