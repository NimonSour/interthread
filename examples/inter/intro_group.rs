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
pub struct Actor(i8);
impl Actor {
    pub fn new() -> Self {
        Self(0)
    }
    pub fn input(&mut self, v: i8) {
        self.0 = v;
    }
    pub fn output(&self) -> i8 {
        self.0
    }
    pub fn in_out(&self, v: i8) -> i8 {
        v
    }
    pub fn add(&mut self, v: i8) -> i8 {
        self.0 += v;
        self.0
    }
}
pub enum ActorScript {
    Input { input: (i8) },
    Output { output: oneshot::Sender<i8> },
    InOut { input: (i8), output: oneshot::Sender<i8> },
    Add { input: (i8), output: oneshot::Sender<i8> },
}
impl ActorScript {
    pub fn direct(self, actor: &mut Actor) {
        match self {
            ActorScript::Input { input: (v) } => {
                actor.input(v);
            }
            ActorScript::Output { output: send } => {
                send.send(actor.output())
                    .unwrap_or_else(|_error| {
                        core::panic!(
                            "'ActorScript::Output.direct'. Sending on a closed channel."
                        )
                    });
            }
            ActorScript::InOut { input: (v), output: send } => {
                send.send(actor.in_out(v))
                    .unwrap_or_else(|_error| {
                        core::panic!(
                            "'ActorScript::InOut.direct'. Sending on a closed channel."
                        )
                    });
            }
            ActorScript::Add { input: (v), output: send } => {
                send.send(actor.add(v))
                    .unwrap_or_else(|_error| {
                        core::panic!(
                            "'ActorScript::Add.direct'. Sending on a closed channel."
                        )
                    });
            }
        }
    }
    pub fn play(receiver: std::sync::mpsc::Receiver<ActorScript>, mut actor: Actor) {
        while let Ok(msg) = receiver.recv() {
            msg.direct(&mut actor);
        }
        eprintln!("Actor end of life ...");
    }
}
impl std::fmt::Debug for ActorScript {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActorScript::Input { .. } => write!(f, "ActorScript::Input"),
            ActorScript::Output { .. } => write!(f, "ActorScript::Output"),
            ActorScript::InOut { .. } => write!(f, "ActorScript::InOut"),
            ActorScript::Add { .. } => write!(f, "ActorScript::Add"),
        }
    }
}
#[derive(Clone)]
pub struct ActorLive {
    sender: std::sync::mpsc::Sender<ActorScript>,
}
impl ActorLive {
    pub fn new() -> Self {
        let actor = Actor::new();
        let (sender, receiver) = std::sync::mpsc::channel();
        let actor_live = Self { sender };
        std::thread::spawn(|| { ActorScript::play(receiver, actor) });
        actor_live
    }
    pub fn input(&mut self, v: i8) {
        let msg = ActorScript::Input { input: (v) };
        let _ = self
            .sender
            .send(msg)
            .expect("'ActorLive::method.send'. Channel is closed!");
    }
    pub fn output(&self) -> i8 {
        let (send, recv) = oneshot::channel();
        let msg = ActorScript::Output {
            output: send,
        };
        let _ = self
            .sender
            .send(msg)
            .expect("'ActorLive::method.send'. Channel is closed!");
        recv.recv().expect("'ActorLive::method.recv'. Channel is closed!")
    }
    pub fn in_out(&self, v: i8) -> i8 {
        let (send, recv) = oneshot::channel();
        let msg = ActorScript::InOut {
            input: (v),
            output: send,
        };
        let _ = self
            .sender
            .send(msg)
            .expect("'ActorLive::method.send'. Channel is closed!");
        recv.recv().expect("'ActorLive::method.recv'. Channel is closed!")
    }
    pub fn add(&mut self, v: i8) -> i8 {
        let (send, recv) = oneshot::channel();
        let msg = ActorScript::Add {
            input: (v),
            output: send,
        };
        let _ = self
            .sender
            .send(msg)
            .expect("'ActorLive::method.send'. Channel is closed!");
        recv.recv().expect("'ActorLive::method.recv'. Channel is closed!")
    }
}
pub fn main() {
    let mut live = ActorLive::new();
    live.input(3);
    assert_eq!(live.output(), 3);
    assert_eq!(live.in_out(4), 4);
    assert_eq!(live.add(5), 8);
}
