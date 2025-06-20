use interthread::actor as life;

use std::rc::Rc;




// retun Self 
#[test]
fn not_send_actor() {
    struct Actor {
        val: Rc<u8>,
    }
    #[life(ty="!Send")]
    impl Actor {
        pub fn new( val: u8 ) -> Self {
            Self { val: Rc::new(val) }
        }
    
        pub fn set(&mut self, val: u8 ){
            self.val = Rc::new(val);
        }
    
        pub fn get(& self ) -> u8 {
            *self.val
        }
    }

    let mut act = ActorLive::new(10);

    assert_eq!(10, act.get());
    act.set(5);
    assert_eq!(5, act.get());
}

#[test]
fn not_send_actor_opt() {

    struct Actor {
        val: Rc<u8>,
    }
    #[life(ty="!Send")]
    impl Actor {

        pub fn new( val : u8 ) -> Option<Self> {
            if val < 10 { return None; }
            
            Some(Self { val: Rc::new( val ) })
        }
    
        pub fn set(&mut self, val: u8 ){
            self.val = Rc::new(val);
        }
    
        pub fn get(& self ) -> u8 {
            *self.val
        }
    }

    let mut act = ActorLive::new(10).expect("Failed to initiate the Actor");
    
    assert_eq!(10, act.get());
    act.set(5);
    assert_eq!(5, act.get());

    let act_2 = ActorLive::new(9);
    assert_eq!(true, act_2.is_none());
    
}

// return Result<Self, E> 
#[test]
fn not_send_actor_res_slf_err() {

    #[derive(Debug,PartialEq,Eq)]
    enum MyError{
        TooSmall(String)
    }

    struct Actor {
        val: Rc<u8>,
    }
    #[life(ty="!Send")]
    impl Actor {

        pub fn new( val : u8 ) -> Result<Self,MyError> {
            if val < 10 { return Result::Err(MyError::TooSmall("Val must be > 10 !".into())); }
            
            Ok(Self { val: Rc::new( val ) })
        }
    
        pub fn set(&mut self, val: u8 ){
            self.val = Rc::new(val);
        }
    
        pub fn get(& self ) -> u8 {
            *self.val
        }
    }

    let mut act = ActorLive::new(10).expect("Failed to initiate the Actor");
    assert_eq!(10, act.get());
    act.set(5);
    assert_eq!(5, act.get());

    match ActorLive::new(9){
        Ok(_) => { assert!(false) },
        Err(e) => { assert_eq!(e,MyError::TooSmall("Val must be > 10 !".into())) }
    }
}

// return Result<Self> 
#[test]
// this will work for Result types that are of the form 
// type Result<T> = std::result::Result<T, some_crate::Error>
// like in crate "anyhow"
fn not_send_actor_res_slf() {
    
    mod utils {
        #[derive(Debug,PartialEq,Eq)]
        pub enum MyError { TooSmall(String) }

        pub type Result<T> = std::result::Result<T, MyError>;
    }

    struct Actor {
        val: Rc<u8>,
    }
    #[life(ty="!Send")]
    impl Actor {

        pub fn new( val : u8 ) -> utils::Result<Self> {
            if val < 10 { return utils::Result::Err(utils::MyError::TooSmall("Val must be > 10 !".into())); }
            
            Ok(Self { val: Rc::new( val ) })
        }
    
        pub fn set(&mut self, val: u8 ){
            self.val = Rc::new(val);
        }
    
        pub fn get(& self ) -> u8 {
            *self.val
        }
    }

    let mut act = ActorLive::new(10).expect("Failed to initiate the Actor");
    assert_eq!(10, act.get());
    act.set(5);
    assert_eq!(5, act.get());

    match ActorLive::new(9){
        Ok(_) => { assert!(false) },
        Err(e) => { assert_eq!(e,utils::MyError::TooSmall("Val must be > 10 !".into())) }
    }


}