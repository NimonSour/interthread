pub struct AnyOtherType;
pub struct Aa(pub u8);
pub struct Bb(pub u8);
pub struct Cc(pub u8);
pub struct AaBbCc {
    pub a: Aa,
    pub b: Bb,
    pub c: Cc,
    any: AnyOtherType,
}
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
    AddNumber { input: (i8), output: oneshot::Sender<i8> },
    GetValue { output: oneshot::Sender<i8> },
}
impl MyActorScript {
    pub fn direct(self, actor: &mut MyActor) {
        match self {
            MyActorScript::Increment {} => {
                actor.increment();
            }
            MyActorScript::AddNumber { input: (num), output: send } => {
                send.send(actor.add_number(num))
                    .unwrap_or_else(|_error| {
                        core::panic!(
                            "'MyActorScript::AddNumber.direct'. Sending on a closed channel."
                        )
                    });
            }
            MyActorScript::GetValue { output: send } => {
                send.send(actor.get_value())
                    .unwrap_or_else(|_error| {
                        core::panic!(
                            "'MyActorScript::GetValue.direct'. Sending on a closed channel."
                        )
                    });
            }
        }
    }
    pub fn play(receiver: std::sync::mpsc::Receiver<MyActorScript>, mut actor: MyActor) {
        while let Ok(msg) = receiver.recv() {
            msg.direct(&mut actor);
        }
        eprintln!("MyActor end of life ...");
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
        let actor_live = Self { sender };
        std::thread::spawn(|| { MyActorScript::play(receiver, actor) });
        actor_live
    }
    pub fn add_number(&mut self, num: i8) -> i8 {
        let (send, recv) = oneshot::channel();
        let msg = MyActorScript::AddNumber {
            input: (num),
            output: send,
        };
        let _ = self
            .sender
            .send(msg)
            .expect("'MyActorLive::method.send'. Channel is closed!");
        recv.recv().expect("'MyActorLive::method.recv'. Channel is closed!")
    }
    pub fn get_value(&self) -> i8 {
        let (send, recv) = oneshot::channel();
        let msg = MyActorScript::GetValue {
            output: send,
        };
        let _ = self
            .sender
            .send(msg)
            .expect("'MyActorLive::method.send'. Channel is closed!");
        recv.recv().expect("'MyActorLive::method.recv'. Channel is closed!")
    }
}
pub fn main() {}
