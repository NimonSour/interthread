use tokio::time::{sleep,Duration};
pub struct MyActor;

#[interthread::actor(channel=2,lib="tokio")] 
impl MyActor {

    pub fn new() -> Self {Self}

    pub async fn do_sleep(&self, n:u8) {
        tokio::spawn(async move{
            sleep(Duration::from_secs(1)).await;
            println!("Task {} awake now!",n);
        });
    }
}

#[tokio::main]
async fn main(){

    let actor = MyActorLive::new();

    for i in 0..60 {

        let act_clone = actor.clone();

        let _ = tokio::spawn(async move {
            act_clone.do_sleep(i).await;
        });
    }
    // see how long it takes to commplete all the tasks
    sleep(Duration::from_secs_f64(1.01)).await;
}

//outputs ( on my machine )
/*
Task 34 awake now!
Task 23 awake now!
Task 25 awake now!
Task 24 awake now!
Task 5 awake now!
Task 32 awake now!
Task 0 awake now!
Task 26 awake now!
Task 33 awake now!
Task 22 awake now!
Task 1 awake now!
Task 27 awake now!
Task 30 awake now!
Task 2 awake now!
Task 39 awake now!
Task 18 awake now!
Task 36 awake now!
Task 35 awake now!
Task 28 awake now!
Task 13 awake now!
Task 29 awake now!
Task 17 awake now!
Task 15 awake now!
Task 16 awake now!
Task 40 awake now!
Task 14 awake now!
Task 41 awake now!
Task 19 awake now!
Task 3 awake now!
Task 7 awake now!
Task 31 awake now!
Task 20 awake now!
Task 38 awake now!
Task 6 awake now!
Task 4 awake now!
Task 12 awake now!
Task 11 awake now!
Task 9 awake now!
Task 8 awake now!
Task 21 awake now!
Task 10 awake now!
Task 43 awake now!
Task 54 awake now!
Task 46 awake now!
Task 47 awake now!
Task 45 awake now!
Task 44 awake now!
Task 50 awake now!
Task 37 awake now!
Task 49 awake now!
Task 51 awake now!
Task 48 awake now!
Task 52 awake now!
Task 56 awake now!
Task 53 awake now!
Task 59 awake now!
Task 42 awake now!
Task 57 awake now!
Task 58 awake now!
Task 55 awake now!
*/



//setting the duration to `1.00`
//outputs ( on my machine )
/*
Task 57 awake now!
Task 0 awake now!
 */