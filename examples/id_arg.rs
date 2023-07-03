
use std::thread::spawn;
pub struct MyActor ;

#[interthread::actor(channel=2, id=true)] 
impl MyActor {
    pub fn new() -> Self { Self{} } 
}
fn main() {

    let actor_1 = MyActorLive::new();

    let handle_2 = spawn( move || { 
        MyActorLive::new()
    });
    let actor_2 = handle_2.join().unwrap();

    let handle_3 = spawn( move || {
        MyActorLive::new()
    });
    let actor_3 = handle_3.join().unwrap();
    
    // they are the same type objects
    // but serving differrent threads
    // different actors !   
    assert!(actor_1 != actor_2);
    assert!(actor_2 != actor_3);
    assert!(actor_3 != actor_1);

    // sice we know the order of invocation
    // we correctly presume
    assert_eq!(actor_1 > actor_2, true );
    assert_eq!(actor_2 > actor_3, true );
    assert_eq!(actor_3 < actor_1, true );

    // but if we check the order by `debute` value
    assert_eq!(actor_1.debut < actor_2.debut, true );
    assert_eq!(actor_2.debut < actor_3.debut, true );
    assert_eq!(actor_3.debut > actor_1.debut, true );
    
    // This is because the 'debut' 
    // is a time record of initiation
    // Charles S Chaplin (1889)
    // Keanu Reeves      (1964)


    // we can count `live` instances for 
    // every model
    use std::sync::Arc;
    let mut a11 = actor_1.clone();
    let mut a12 = actor_1.clone();

    let mut a31 = actor_3.clone();

    assert_eq!(Arc::strong_count(&actor_1.debut), 3 );
    assert_eq!(Arc::strong_count(&actor_2.debut), 1 );
    assert_eq!(Arc::strong_count(&actor_3.debut), 2 );
            

    // the name field is not taken 
    // into account when comparison is
    // perfomed       
    assert!( a11 == a12);
    assert!( a11 != a31);

    a11.name = String::from("Alice");
    a12.name = String::from("Bob");

    a31.name = String::from("Alice");

    assert_eq!(a11 == a12, true );
    assert_eq!(a11 != a31, true );

}