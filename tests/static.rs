
use interthread::actor as life;


#[test]
fn static_methods_generic_model() {

    struct Actor <T> { field: T }

    #[life]
    impl <T> Actor <T>
    where 
    T: ToString 
    {
        pub fn new( field: T) -> Self { Self{ field } }

        pub fn set_field(&mut self, val: T ){ self.field = val; }

        pub fn stat_method_io( val:u8) -> u16 { (val*2) as u16 }

        pub fn stat_method_i( _val:u8) {}

        pub fn stat_method_o( ) -> u16 { 144u16 }

        pub fn stat_method_none( ) {}
        
        pub fn generic_static<S:ToString>( s:S) -> String {
            s.to_string()
        }
    }

    let mut act = ActorLive::new(String::from("a"));
    
    let var = ActorLive::<String>::generic_static(1);
    act.set_field(var);

    assert_eq!(ActorLive::<String>::stat_method_io(3), 6u16 );
    assert_eq!(ActorLive::<String>::stat_method_i(3), () );
    assert_eq!(ActorLive::<String>::stat_method_o(), 144u16 );
    assert_eq!(ActorLive::<String>::stat_method_none(),());

    // if generic, the model becomes inflexible ((( 

}



#[test]
fn static_methods_async() {

    struct Actor;

    #[life(lib="tokio")]
    impl Actor {
        pub fn new() -> Self { Self }

        pub async fn stat_method_io( val:u8) -> u16 { (val*2) as u16 }

        pub async fn stat_method_i( _val:u8) {}

        pub async fn stat_method_o( ) -> u16 { 144u16 }

        pub async fn stat_method_none( ) {}
        
        pub async fn generic_static<S:ToString>( s:S) -> String {
            s.to_string()
        }
    }

    tokio::runtime::Runtime::new()
    .unwrap()
    .block_on(async {
        let _ = ActorLive::new();
        assert_eq!(ActorLive::stat_method_io(3).await, 6u16 );
        assert_eq!(ActorLive::stat_method_i(3).await, () );
        assert_eq!(ActorLive::stat_method_o().await, 144u16 );
        assert_eq!(ActorLive::stat_method_none().await,());
        assert_eq!(ActorLive::generic_static(1).await, 1.to_string());
    });
    
}