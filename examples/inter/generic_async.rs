pub struct Actor<T>(T);
impl<T: Clone> Actor<T>
where
    T: core::ops::AddAssign + std::fmt::Debug,
{
    pub fn new(v: T) -> Self {
        Self(v)
    }
    pub fn input(&mut self, v: T) {
        self.0 = v;
    }
    pub fn output(&self) -> T {
        self.0.clone()
    }
    pub fn in_out(&self, v: T) -> T {
        v
    }
    pub fn add(&mut self, v: T) -> T {
        self.0 += v;
        self.0.clone()
    }
}
pub enum ActorScript<T>
where
    T: core::ops::AddAssign + std::fmt::Debug + Send + Sync + 'static,
{
    Input { input: (T) },
    Output { output: tokio::sync::oneshot::Sender<T> },
    InOut { input: (T), output: tokio::sync::oneshot::Sender<T> },
    Add { input: (T), output: tokio::sync::oneshot::Sender<T> },
}
impl<T: Clone> ActorScript<T>
where
    T: core::ops::AddAssign + std::fmt::Debug + Send + Sync + 'static,
{
    pub fn debut() -> std::sync::Arc<std::time::SystemTime> {
        static LAST: std::sync::Mutex<std::time::SystemTime> = std::sync::Mutex::new(
            std::time::SystemTime::UNIX_EPOCH,
        );
        let mut last_time = LAST.lock().unwrap();
        let mut next_time = std::time::SystemTime::now();
        while !(*last_time < next_time) {
            if *last_time == next_time {
                next_time += std::time::Duration::new(0, 1);
            } else {
                next_time = std::time::SystemTime::now();
            }
        }
        *last_time = next_time.clone();
        std::sync::Arc::new(next_time)
    }
    pub fn direct(self, actor: &mut Actor<T>) {
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
    pub async fn play(
        mut receiver: tokio::sync::mpsc::Receiver<ActorScript<T>>,
        mut actor: Actor<T>,
    ) {
        while let Some(msg) = receiver.recv().await {
            msg.direct(&mut actor);
        }
        eprintln!("Actor end of life ...");
    }
}
impl<T> std::fmt::Debug for ActorScript<T>
where
    T: core::ops::AddAssign + std::fmt::Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActorScript::Input { .. } => write!(f, "Input"),
            ActorScript::Output { .. } => write!(f, "Output"),
            ActorScript::InOut { .. } => write!(f, "InOut"),
            ActorScript::Add { .. } => write!(f, "Add"),
        }
    }
}
#[derive(Clone)]
pub struct ActorLive<T>
where
    T: core::ops::AddAssign + std::fmt::Debug + Send + Sync + 'static,
{
    sender: tokio::sync::mpsc::Sender<ActorScript<T>>,
    pub debut: std::sync::Arc<std::time::SystemTime>,
    pub name: String,
}
impl<T: Clone> ActorLive<T>
where
    T: core::ops::AddAssign + std::fmt::Debug + Send + Sync + 'static,
{
    pub fn new(v: T) -> Self {
        let actor = Actor::new(v);
        let (sender, receiver) = tokio::sync::mpsc::channel(3);
        let debut = ActorScript::<T>::debut();
        let name = String::from("");
        let actor_live = Self { sender, debut, name };
        tokio::spawn(ActorScript::play(receiver, actor));
        actor_live
    }
    pub async fn input(&mut self, v: T) {
        let msg = ActorScript::Input { input: (v) };
        let _ = self.sender.send(msg).await;
    }
    pub async fn output(&self) -> T {
        let (send, recv) = tokio::sync::oneshot::channel();
        let msg = ActorScript::Output {
            output: send,
        };
        let _ = self.sender.send(msg).await;
        recv.await.expect("'ActorLive::method.recv'. Channel is closed!")
    }
    pub async fn in_out(&self, v: T) -> T {
        let (send, recv) = tokio::sync::oneshot::channel();
        let msg = ActorScript::InOut {
            input: (v),
            output: send,
        };
        let _ = self.sender.send(msg).await;
        recv.await.expect("'ActorLive::method.recv'. Channel is closed!")
    }
    pub async fn add(&mut self, v: T) -> T {
        let (send, recv) = tokio::sync::oneshot::channel();
        let msg = ActorScript::Add {
            input: (v),
            output: send,
        };
        let _ = self.sender.send(msg).await;
        recv.await.expect("'ActorLive::method.recv'. Channel is closed!")
    }
    pub fn inter_get_debut(&self) -> std::time::SystemTime {
        *self.debut
    }
    pub fn inter_get_count(&self) -> usize {
        std::sync::Arc::strong_count(&self.debut)
    }
    pub fn inter_set_name<Name: std::string::ToString>(&mut self, name: Name) {
        self.name = name.to_string();
    }
    pub fn inter_get_name(&self) -> &str {
        &self.name
    }
}
impl<T> PartialEq for ActorLive<T>
where
    T: core::ops::AddAssign + std::fmt::Debug + Send + Sync + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        *self.debut == *other.debut
    }
}
impl<T> Eq for ActorLive<T>
where
    T: core::ops::AddAssign + std::fmt::Debug + Send + Sync + 'static,
{}
impl<T> PartialOrd for ActorLive<T>
where
    T: core::ops::AddAssign + std::fmt::Debug + Send + Sync + 'static,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.debut.partial_cmp(&self.debut)
    }
}
impl<T> Ord for ActorLive<T>
where
    T: core::ops::AddAssign + std::fmt::Debug + Send + Sync + 'static,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.debut.cmp(&self.debut)
    }
}
#[tokio::main]
async fn main() {
    let mut live = ActorLive::new(0);
    live.input(3).await;
    assert_eq!(live.output(). await, 3);
    assert_eq!(live.in_out(4). await, 4);
    assert_eq!(live.add(5). await, 8);
}
