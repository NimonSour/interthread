pub struct MaunActor<A, B, C> {
    value_a: Option<A>,
    value_b: Option<B>,
    value_c: Option<C>,
}
impl<A, B, C> MaunActor<A, B, C>
where
    A: ToString,
    B: ToString,
    C: ToString,
{
    pub fn new() -> Self {
        MaunActor {
            value_a: None,
            value_b: None,
            value_c: None,
        }
    }
    pub fn set_a(&mut self, value: A) {
        self.value_a = Some(value);
    }
    pub fn set_b(&mut self, value: B) {
        self.value_b = Some(value);
    }
    pub fn set_c(&mut self, value: C) {
        self.value_c = Some(value);
    }
    pub fn sentence(&self) -> String {
        let mut s = String::new();
        if let Some(v) = self.value_a.as_ref() {
            s += &v.to_string();
        }
        if let Some(v) = self.value_b.as_ref() {
            s += &v.to_string();
        }
        if let Some(v) = self.value_c.as_ref() {
            s += &v.to_string();
        }
        s
    }
}
pub enum MaunActorScript<B, A, C>
where
    B: ToString + Send + Sync + 'static,
    A: ToString + Send + Sync + 'static,
    C: ToString + Send + Sync + 'static,
{
    SetA { input: (A) },
    SetB { input: (B) },
    SetC { input: (C) },
    Sentence { inter_send: oneshot::Sender<String> },
}
impl<B, A, C> MaunActorScript<B, A, C>
where
    B: ToString + Send + Sync + 'static,
    A: ToString + Send + Sync + 'static,
    C: ToString + Send + Sync + 'static,
{
    pub fn direct(self, actor: &mut MaunActor<A, B, C>) {
        match self {
            MaunActorScript::SetA { input: (value) } => {
                actor.set_a(value);
            }
            MaunActorScript::SetB { input: (value) } => {
                actor.set_b(value);
            }
            MaunActorScript::SetC { input: (value) } => {
                actor.set_c(value);
            }
            MaunActorScript::Sentence { inter_send } => {
                inter_send
                    .send(actor.sentence())
                    .unwrap_or_else(|_error| {
                        core::panic!(
                            "'MaunActorScript::Sentence.direct'. Sending on a closed channel."
                        )
                    });
            }
        }
    }
    pub fn play(
        receiver: std::sync::mpsc::Receiver<MaunActorScript<B, A, C>>,
        mut actor: MaunActor<A, B, C>,
    ) {
        while let Ok(msg) = receiver.recv() {
            msg.direct(&mut actor);
        }
        eprintln!("MaunActor the end ...");
    }
}
impl<B, A, C> std::fmt::Debug for MaunActorScript<B, A, C>
where
    B: ToString + Send + Sync + 'static,
    A: ToString + Send + Sync + 'static,
    C: ToString + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaunActorScript::SetA { .. } => write!(f, "MaunActorScript::SetA"),
            MaunActorScript::SetB { .. } => write!(f, "MaunActorScript::SetB"),
            MaunActorScript::SetC { .. } => write!(f, "MaunActorScript::SetC"),
            MaunActorScript::Sentence { .. } => write!(f, "MaunActorScript::Sentence"),
        }
    }
}
#[derive(Clone)]
pub struct MaunActorLive<B, A, C>
where
    B: ToString + Send + Sync + 'static,
    A: ToString + Send + Sync + 'static,
    C: ToString + Send + Sync + 'static,
{
    sender: std::sync::mpsc::SyncSender<MaunActorScript<B, A, C>>,
}
impl<B, A, C> MaunActorLive<B, A, C>
where
    B: ToString + Send + Sync + 'static,
    A: ToString + Send + Sync + 'static,
    C: ToString + Send + Sync + 'static,
{
    pub fn new() -> Self {
        let actor = MaunActor::new();
        let (sender, receiver) = std::sync::mpsc::sync_channel(2);
        std::thread::spawn(move || { MaunActorScript::play(receiver, actor) });
        Self { sender }
    }
    pub fn set_a(&mut self, value: A) {
        let msg = MaunActorScript::SetA {
            input: (value),
        };
        let _ = self
            .sender
            .send(msg)
            .expect("'MaunActorLive::method.send'. Channel is closed!");
    }
    pub fn set_b(&mut self, value: B) {
        let msg = MaunActorScript::SetB {
            input: (value),
        };
        let _ = self
            .sender
            .send(msg)
            .expect("'MaunActorLive::method.send'. Channel is closed!");
    }
    pub fn set_c(&mut self, value: C) {
        let msg = MaunActorScript::SetC {
            input: (value),
        };
        let _ = self
            .sender
            .send(msg)
            .expect("'MaunActorLive::method.send'. Channel is closed!");
    }
    pub fn sentence(&self) -> String {
        let (inter_send, inter_recv) = oneshot::channel();
        let msg = MaunActorScript::Sentence {
            inter_send,
        };
        let _ = self
            .sender
            .send(msg)
            .expect("'MaunActorLive::method.send'. Channel is closed!");
        inter_recv
            .recv()
            .unwrap_or_else(|_error| {
                core::panic!("'MaunActor::sentence' from inter_recv. Channel is closed!")
            })
    }
}

