


use interthread::actor as life;
//STD INTER
#[test]
fn actor_sync_inter_str() {
    pub struct Actor(i8);
    #[life(channel="inter")]
    impl Actor {
        pub fn new() -> Self{Self(0)}
        pub fn input(&mut self, v:i8){self.0 = v}
        pub fn output(&self)->i8{self.0}
        pub fn in_out(&self,v:i8)->i8{v}
        pub fn add(&mut self, v:i8) -> i8{self.0 += v;self.0}
    }
    let _live = ActorLive::new();
}
#[test]
fn actor_sync_inter() {
    pub struct Actor(i8);
    #[life]
    impl Actor {
        pub fn new() -> Self{Self(0)}
        pub fn input(&mut self, v:i8){self.0 = v}
        pub fn output(&self)->i8{self.0}
        pub fn in_out(&self,v:i8)->i8{v}
        pub fn add(&mut self, v:i8) -> i8{self.0 += v;self.0}
    }
    let mut live = ActorLive::new();
    live.input(3); 
    assert_eq!( live.output(),  3); 
    assert_eq!( live.in_out(4), 4); 
    assert_eq!( live.add(5),    8); 
}
//STD
#[test]
fn actor_sync_bounded() {
    pub struct Actor(i8);
    #[life(channel=3)]
    impl Actor {
        pub fn new() -> Self{Self(0)}
        pub fn input(&mut self, v:i8){self.0 = v}
        pub fn output(&self)->i8{self.0}
        pub fn in_out(&self,v:i8)->i8{v}
        pub fn add(&mut self, v:i8) -> i8{self.0 += v;self.0}
    }
    let mut live = ActorLive::new();
    live.input(3); 
    assert_eq!( live.output(),  3); 
    assert_eq!( live.in_out(4), 4); 
    assert_eq!( live.add(5),    8); 
}
#[test]
fn actor_sync_unbounded_int_name() {
    pub struct Actor(i8);
    #[life(channel=0, name="MyActor")]
    impl Actor {
        pub fn new() -> Self{Self(0)}
        pub fn input(&mut self, v:i8){self.0 = v}
        pub fn output(&self)->i8{self.0}
        pub fn in_out(&self,v:i8)->i8{v}
        pub fn add(&mut self, v:i8) -> i8{self.0 += v;self.0}
    }
    let _live = MyActorLive::new();
}
#[test]
fn actor_sync_unbounded() {
    pub struct Actor(i8);
    #[life(channel="unbounded")]
    impl Actor {
        pub fn new() -> Self{Self(0)}
        pub fn input(&mut self, v:i8){self.0 = v}
        pub fn output(&self)->i8{self.0}
        pub fn in_out(&self,v:i8)->i8{v}
        pub fn add(&mut self, v:i8) -> i8{self.0 += v;self.0}
    }
    let mut live = ActorLive::new();
    live.input(3); 
    assert_eq!( live.output(),  3); 
    assert_eq!( live.in_out(4), 4); 
    assert_eq!( live.add(5),    8); 
}   

// TOKIO
#[test]
fn actor_tokio_bounded() {
    pub struct Actor(i8);
    #[life(channel=3,lib="tokio")]
    impl Actor {
        pub fn new() -> Self{Self(0)}
        pub fn input(&mut self, v:i8){self.0 = v}
        pub fn output(&self)->i8{self.0}
        pub fn in_out(&self,v:i8)->i8{v}
        pub fn add(&mut self, v:i8) -> i8{self.0 += v;self.0}
    }
    tokio::runtime::Runtime::new()
    .unwrap()
    .block_on(
        async {
        let mut live = ActorLive::new();
        live.input(3).await; 
        assert_eq!( live.output().await,  3); 
        assert_eq!( live.in_out(4).await, 4); 
        assert_eq!( live.add(5).await,    8); 
    });
}

#[test]
fn actor_tokio_unbounded() {
    pub struct Actor(i8);
    #[life(channel="unbounded",lib="tokio")]
    impl Actor {
        pub fn new() -> Self{Self(0)}
        pub fn input(&mut self, v:i8){self.0 = v}
        pub fn output(&self)->i8{self.0}
        pub fn in_out(&self,v:i8)->i8{v}
        pub fn add(&mut self, v:i8) -> i8{self.0 += v;self.0}
    }
    tokio::runtime::Runtime::new()
    .unwrap()
    .block_on( async {
        let mut live = ActorLive::new();
        live.input(3).await; 
        assert_eq!( live.output().await,  3); 
        assert_eq!( live.in_out(4).await, 4); 
        assert_eq!( live.add(5).await,    8); 
    });
}
//ASYNC-STD
#[test]
fn actor_async_std_bounded() {
    pub struct Actor(i8);
    #[life(channel=3,lib="async_std")]
    impl Actor {
        pub fn new() -> Self{Self(0)}
        pub fn input(&mut self, v:i8){self.0 = v}
        pub fn output(&self)->i8{self.0}
        pub fn in_out(&self,v:i8)->i8{v}
        pub fn add(&mut self, v:i8) -> i8{self.0 += v;self.0}
    }
    async_std::task::block_on(async {
            let mut live = ActorLive::new();
            live.input(3).await; 
            assert_eq!( live.output().await,  3); 
            assert_eq!( live.in_out(4).await, 4); 
            assert_eq!( live.add(5).await,    8); 
    });
}
 #[test]
fn actor_async_std_unbounded() {
    pub struct Actor(i8);
    #[life(channel="unbounded",lib="async_std")]
    impl Actor {
        pub fn new() -> Self{Self(0)}
        pub fn input(&mut self, v:i8){self.0 = v}
        pub fn output(&self)->i8{self.0}
        pub fn in_out(&self,v:i8)->i8{v}
        pub fn add(&mut self, v:i8) -> i8{self.0 += v;self.0}
    }
    async_std::task::block_on(async {
            let mut live = ActorLive::new();
            live.input(3).await; 
            assert_eq!( live.output().await,  3); 
            assert_eq!( live.in_out(4).await, 4); 
            assert_eq!( live.add(5).await,    8); 
    });
}
//SMOL
#[test]
fn actor_smol_bounded() {
    pub struct Actor(i8);
    #[life(channel=3,lib="smol")]
    impl Actor {
        pub fn new() -> Self{Self(0)}
        pub fn input(&mut self, v:i8){self.0 = v}
        pub fn output(&self)->i8{self.0}
        pub fn in_out(&self,v:i8)->i8{v}
        pub fn add(&mut self, v:i8) -> i8{self.0 += v;self.0}
    }
    async_std::task::block_on(async {
            let mut live = ActorLive::new();
            live.input(3).await; 
            assert_eq!( live.output().await,  3); 
            assert_eq!( live.in_out(4).await, 4); 
            assert_eq!( live.add(5).await,    8); 
    });
}
#[test]
fn actor_smol_unbounded() {
    pub struct Actor(i8);
    #[life(channel="unbounded",lib="smol")]
    impl Actor {
        pub fn new() -> Self{Self(0)}
        pub fn input(&mut self, v:i8){self.0 = v}
        pub fn output(&self)->i8{self.0}
        pub fn in_out(&self,v:i8)->i8{v}
        pub fn add(&mut self, v:i8) -> i8{self.0 += v;self.0}
    }
    async_std::task::block_on(async {
            let mut live = ActorLive::new();
            live.input(3).await; 
            assert_eq!( live.output().await,  3); 
            assert_eq!( live.in_out(4).await, 4); 
            assert_eq!( live.add(5).await,    8); 
    });
}


// #[test]
// fn self_word_in_arg_type() {
//     #[derive(Debug,Clone,PartialEq, Eq)]
//     pub struct ActorSelf(i8);
//     #[life(channel=3)]
//     impl ActorSelf {
//         pub fn new() -> Self{Self(0)}
//         pub fn input(&mut self, v:i8){self.0 = v}
//         pub fn output(&self)->i8{self.0}
//         pub fn in_out(&self,v:ActorSelf)->ActorSelf{v}
//         pub fn add(&mut self, v:i8) -> i8{self.0 += v;self.0}
//     }
//     let a = ActorSelf::new();
//     let mut live = ActorSelfLive::new();
//     live.input(3); 
//     assert_eq!( live.output(),  3); 
//     assert_eq!( live.in_out(a.clone()), a); 
//     assert_eq!( live.add(5),    8); 
// }






