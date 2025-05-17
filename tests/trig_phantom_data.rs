
use interthread::actor as life;


#[test]
fn actor_private_generics() {


    pub struct Actor<A,B,C>{
        #[allow(dead_code)]
        a: A,
        #[allow(dead_code)]
        b: B,
        #[allow(dead_code)]
        c: C,
    }


    #[life(show)]
    impl <A,B,C> Actor <A,B,C> 
    {
        pub fn new( a: A, b: B, c: C ) -> Self{
            Self { a, b, c }
        }
        pub fn method(&self, s: &'static str) -> String {
            format!("Just return a {s} !")
        }
    }


    let live = ActorLive::new(0u8,0u16,0u32);

    assert_eq!("Just return a string !",live.method("string"));
 
}