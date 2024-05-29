mod intro_std;

use intro_std::MyActorLive;
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
    assert_eq!(actor.get_value(), 11)
}
