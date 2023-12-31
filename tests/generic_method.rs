
use interthread::actor as life;


#[test]
fn actor_sync_unbound_default() {
    pub struct Actor<T>(T);
    #[life(debut)]
    impl <T:Clone> Actor <T> 
      where T: core::ops::AddAssign,
    {

        pub fn new(v:T) -> Self{Self(v)}
        pub fn input<I:Into<T>>(&mut self, v:I){self.0 = v.into()}
        pub fn output(&self)->T{self.0.clone()}
        pub fn in_out(&self,v:T)->T{v}
        pub fn add(&mut self, v:T) -> T{self.0 += v;self.0.clone()}
    }
    let mut live = ActorLive::new(0);
    live.input(3); 
    assert_eq!( live.output(),  3); 
    assert_eq!( live.in_out(4), 4); 
    assert_eq!( live.add(5),    8); 
    live.input(5i8);
    assert_eq!( live.output(),5i32); 
 
}

#[test]
fn actor_sync_unbounded_int() {
    pub struct Actor<T>(T);
    #[life(channel=0, name="MyActor",debut)]
    impl <T:Clone> Actor <T> 
      where T: core::ops::AddAssign,
    {
        pub fn new(v:T) -> Self{Self(v)}
        pub fn input(&mut self, v:T){self.0 = v}
        pub fn output(&self)->T{self.0.clone()}
        pub fn in_out(&self,v:T)->T{v}
        pub fn add<I:Into<T>>(&mut self, v:I) -> T{self.0 += v.into();self.0.clone()}

    }
    let mut live = MyActorLive::new(0);
    live.input(3); 
    assert_eq!( live.output(),  3); 
    assert_eq!( live.in_out(4), 4); 
    assert_eq!( live.add(5i8),  8); 

}
//STD
#[test]
fn actor_sync_bounded() {
    pub struct Actor<T>(T);
    #[life(channel=3,debut)]
    impl <T:Clone> Actor <T> 
      where T: core::ops::AddAssign,
    {
        pub fn new(v:T) -> Self{Self(v)}
        pub fn input(&mut self, v:T){self.0 = v}
        pub fn output(&self)->T{self.0.clone()}
        pub fn in_out(&self,v:T)->T{v}
        pub fn add<I:Into<T>>(&mut self, v:I) -> T{self.0 += v.into();self.0.clone()}
    }
    let mut live = ActorLive::new(0);
    live.input(3); 
    assert_eq!( live.output(),  3); 
    assert_eq!( live.in_out(4), 4); 
    assert_eq!( live.add(5i8),  8); 

}
 

// TOKIO
#[test]
fn actor_tokio_bounded() {
    pub struct Actor<T>(T);
    #[life(channel=3,lib="tokio",debut)]
    impl <T:Clone> Actor <T> 
      where T: core::ops::AddAssign + std::fmt::Debug,
    {
        pub fn new(v:T) -> Self{Self(v)}
        pub fn input(&mut self, v:T){self.0 = v}
        pub fn output(&self)->T{self.0.clone()}
        pub fn in_out(&self,v:T)->T{v}
        pub fn add<I:Into<T>>(&mut self, v:I) -> T{self.0 += v.into();self.0.clone()}

    }
    tokio::runtime::Runtime::new()
    .unwrap()
    .block_on(
        async {
        let mut live = ActorLive::new(0);
        live.input(3).await; 
        assert_eq!( live.output().await,      3); 
        assert_eq!( live.in_out(4).await,     4); 
        assert_eq!( live.add(5i8).await,      8); 

    });
}

#[test]
fn actor_tokio_unbounded() {
    pub struct Actor<T>(T);
    #[life(lib="tokio",debut)]
    impl <T:Clone> Actor <T> 
      where T: core::ops::AddAssign + std::fmt::Debug,
    {
        pub fn new(v:T) -> Self{Self(v)}
        pub fn input(&mut self, v:T){self.0 = v}
        pub fn output(&self)->T{self.0.clone()}
        pub fn in_out(&self,v:T)->T{v}
        pub fn add<I:Into<T>>(&mut self, v:I) -> T{self.0 += v.into();self.0.clone()}

    }
    tokio::runtime::Runtime::new()
    .unwrap()
    .block_on( async {
        let mut live = ActorLive::new(0);
        live.input(3).await; 
        assert_eq!( live.output().await,  3); 
        assert_eq!( live.in_out(4).await, 4); 
        assert_eq!( live.add(5i8).await,  8); 
    });
}
//ASYNC-STD
#[test]
fn actor_async_std_bounded() {
    pub struct Actor<T>(T);
    #[life(channel=3,lib="async_std",debut)]
    impl <T:Clone> Actor <T> 
      where T: core::ops::AddAssign + std::fmt::Debug,
    {
        pub fn new(v:T) -> Self{Self(v)}
        pub fn input(&mut self, v:T){self.0 = v}
        pub fn output(&self)->T{self.0.clone()}
        pub fn in_out(&self,v:T)->T{v}
        pub fn add<I:Into<T>>(&mut self, v:I) -> T{self.0 += v.into();self.0.clone()}

    }
    async_std::task::block_on(async {
            let mut live = ActorLive::new(0);
            live.input(3).await; 
            assert_eq!( live.output().await,  3); 
            assert_eq!( live.in_out(4).await, 4); 
            assert_eq!( live.add(5i8).await,  8);  
    });
}
 #[test]
fn actor_async_std_unbounded() {
    pub struct Actor<T>(T);
    #[life(lib="async_std",debut)]
    impl <T:Clone> Actor <T> 
      where T: core::ops::AddAssign + std::fmt::Debug,
    {
        pub fn new(v:T) -> Self{Self(v)}
        pub fn input(&mut self, v:T){self.0 = v}
        pub fn output(&self)->T{self.0.clone()}
        pub fn in_out(&self,v:T)->T{v}
        pub fn add<I:Into<T>>(&mut self, v:I) -> T{self.0 += v.into();self.0.clone()}

    }
    async_std::task::block_on(async {
            let mut live = ActorLive::new(0);
            live.input(3).await; 
            assert_eq!( live.output().await,  3); 
            assert_eq!( live.in_out(4).await, 4); 
            assert_eq!( live.add(5i8).await,      8); 
    });
}
//SMOL
#[test]
fn actor_smol_bounded() {
    pub struct Actor<T>(T);
    #[life(channel=3,lib="smol",debut)]
    impl <T:Clone> Actor <T> 
      where T: core::ops::AddAssign + std::fmt::Debug,
    {
        pub fn new(v:T) -> Self{Self(v)}
        pub fn input(&mut self, v:T){self.0 = v}
        pub fn output(&self)->T{self.0.clone()}
        pub fn in_out(&self,v:T)->T{v}
        pub fn add<I:Into<T>>(&mut self, v:I) -> T{self.0 += v.into();self.0.clone()}

    }
    async_std::task::block_on(async {
            let mut live = ActorLive::new(0);
            live.input(3).await; 
            assert_eq!( live.output().await,  3); 
            assert_eq!( live.in_out(4).await, 4); 
            assert_eq!( live.add(5i8).await,  8);  
    });
}
#[test]
fn actor_smol_unbounded() {
    pub struct Actor<T>(T);
    #[life(lib="smol",debut)]
    impl <T:Clone> Actor <T> 
      where T: core::ops::AddAssign + std::fmt::Debug,
    {
        pub fn new(v:T) -> Self{Self(v)}
        pub fn input(&mut self, v:T){self.0 = v}
        pub fn output(&self)->T{self.0.clone()}
        pub fn in_out(&self,v:T)->T{v}
        pub fn add<I:Into<T>>(&mut self, v:I) -> T{self.0 += v.into();self.0.clone()}

    }
    async_std::task::block_on(async {
            let mut live = ActorLive::new(0);
            live.input(3).await; 
            assert_eq!( live.output().await,  3); 
            assert_eq!( live.in_out(4).await, 4); 
            assert_eq!( live.add(5i8).await,  8);  
    });
}


// RESULT OPTION

#[test]
fn actor_async_std_unbounded_option() {
    pub struct Actor<T>(T);
    #[life(lib="async_std",debut)]
    impl <T:Clone> Actor <T> 
      where T: core::ops::AddAssign + std::fmt::Debug,
    {
        pub fn new(v:T) -> Option<Self>{ Some(Self(v))}
        pub fn input(&mut self, v:T){self.0 = v}
        pub fn output(&self)->T{self.0.clone()}
        pub fn in_out(&self,v:T)->T{v}
        pub fn add<I:Into<T>>(&mut self, v:I) -> T{self.0 += v.into();self.0.clone()}

    }
    async_std::task::block_on(async {
            let mut live = ActorLive::new(0).unwrap();
            live.input(3).await; 
            assert_eq!( live.output().await,  3); 
            assert_eq!( live.in_out(4).await, 4); 
            assert_eq!( live.add(5i8).await,      8); 
    });
}

#[test]
fn actor_tokio_unbounded_result() {
    pub struct Actor<T>(T);
    #[life(lib="tokio",debut)]
    impl <T:Clone> Actor <T> 
      where T: core::ops::AddAssign + std::fmt::Debug,
    {
        pub fn new(v:T) -> Result<Self,String>{Ok(Self(v))}
        pub fn input(&mut self, v:T){self.0 = v}
        pub fn output(&self)->T{self.0.clone()}
        pub fn in_out(&self,v:T)->T{v}
        pub fn add<I:Into<T>>(&mut self, v:I) -> T{self.0 += v.into();self.0.clone()}

    }
    tokio::runtime::Runtime::new()
    .unwrap()
    .block_on( async {
        let mut live = ActorLive::new(0).unwrap();
        live.input(3).await; 
        assert_eq!( live.output().await,  3); 
        assert_eq!( live.in_out(4).await, 4); 
        assert_eq!( live.add(5i8).await,  8); 
    });
}

