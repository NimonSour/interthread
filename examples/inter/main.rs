mod maun_actor;

fn main() {
    let act = maun_actor::MaunActorLive::<String, &'static str, char>::new();
    let mut one = act.clone();
    let mut two = act.clone();
    let mut thr = act.clone();
    let one_h = std::thread::spawn(move || {
        one.set_a("I'm a generic".to_string());
    });
    let _ = one_h.join();
    // let two_h = std::thread::spawn(move || {
    //     two.set_b(" actor - ");
    // });
    // let _ = two_h.join();
    let thr_h = std::thread::spawn(move || {
        thr.set_c('😀');
    });
    let _ = thr_h.join();
    assert_eq!(act.sentence(), "I'm a generic actor - 😀".to_string());
}
