use std::thread::spawn;
pub struct MyActor;
impl MyActor {
    pub fn new() -> Self {
        Self {}
    }
}
pub enum MyActorScript {}
impl MyActorScript {
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
    pub fn direct(self, actor: &mut MyActor) {
        match self {}
    }
    pub fn play(
        receiver: std::sync::mpsc::Receiver<MyActorScript>,
        mut actor: MyActor,
        debut: std::time::SystemTime,
    ) {
        while let Ok(msg) = receiver.recv() {
            msg.direct(&mut actor);
        }
        eprintln!("MyActor [ {debut:?} ] the end ...");
    }
}
impl std::fmt::Debug for MyActorScript {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MyActorScript")
    }
}
#[derive(Clone)]
pub struct MyActorLive {
    sender: std::sync::mpsc::SyncSender<MyActorScript>,
    pub debut: std::sync::Arc<std::time::SystemTime>,
    pub name: String,
}
impl MyActorLive {
    pub fn new() -> Self {
        let actor = MyActor::new();
        let (sender, receiver) = std::sync::mpsc::sync_channel(2);
        let debut = MyActorScript::debut();
        let debut_play = *std::sync::Arc::clone(&debut);
        std::thread::spawn(move || { MyActorScript::play(receiver, actor, debut_play) });
        Self {
            sender,
            debut: std::sync::Arc::clone(&debut),
            name: format!("{:?}", * debut),
        }
    }
    pub fn inter_get_debut(&self) -> std::time::SystemTime {
        *self.debut
    }
    pub fn inter_get_count(&self) -> usize {
        std::sync::Arc::strong_count(&self.debut)
    }
    pub fn inter_set_name<InterName: std::string::ToString>(&mut self, name: InterName) {
        self.name = name.to_string();
    }
    pub fn inter_get_name(&self) -> &str {
        &self.name
    }
}
impl std::cmp::PartialEq for MyActorLive {
    fn eq(&self, other: &Self) -> bool {
        *self.debut == *other.debut
    }
}
impl std::cmp::Eq for MyActorLive {}
impl std::cmp::PartialOrd for MyActorLive {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.debut.partial_cmp(&self.debut)
    }
}
impl std::cmp::Ord for MyActorLive {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.debut.cmp(&self.debut)
    }
}