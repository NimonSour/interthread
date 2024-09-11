
use interthread::actor as life;


#[test]
fn actor_sync_unbound_default() {
    pub struct Actor<T>(T);
    #[life(debut)]
    impl <T:Clone> Actor <T> 
      where T: core::ops::AddAssign,
    {

        pub fn new(v:T) -> Self{Self(v)}
        pub fn input(&mut self, v:T){self.0 = v}
        pub fn output(&self)->T{self.0.clone()}
        pub fn in_out(&self,v:T)->T{v}
        pub fn add(&mut self, v:T) -> T{self.0 += v;self.0.clone()}
    }
    let mut live = ActorLive::new(0);
    live.input(3); 
    assert_eq!( live.output(),  3); 
    assert_eq!( live.in_out(4), 4); 
    assert_eq!( live.add(5),    8); 
}


#[test]
fn actor_sync_unbound_default_debut_plus_const_generic_param() {
    pub struct Actor<T, const N: usize>([Option<T>;N]);
    #[life(debut)]
    impl <T, const N: usize> Actor <T,N> 
    where
        T: Copy + Default, 
    {
        pub fn new() -> Self { Self ([None; N])}

        pub fn set_value_at(&mut self, index: usize, value: T) {
            if index < N {
                self.0[index] = Some(value);
            } else {
                panic!("Index out of bounds!");
            }
        }
        pub fn get_value_at(&self, index: usize) -> Option<T> {
            if index < N {
                self.0[index]
            } else {
                panic!("Index out of bounds!");
            }
        }

    }
    
    // Note the consts are allways moved to the left hs
    // ( for Actor<T,const N>  ->  ActorLive<const N,T> )
    // as a side effect not by design,
    // probably something to deal with in the future.
    // text editor will guide which is which 
    let mut live = ActorLive::<10usize,i32>::new();
    live.set_value_at(3, -3);
    live.set_value_at(7, -7);
    assert_eq!( live.get_value_at(3).unwrap(),  -3); 
    assert_eq!( live.get_value_at(7).unwrap(),  -7); 
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
        pub fn add(&mut self, v:T) -> T{self.0 += v;self.0.clone()}
    }
    let _live = MyActorLive::new(0);
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
        pub fn add(&mut self, v:T) -> T{self.0 += v;self.0.clone()}
    }
    let mut live = ActorLive::new(0);
    live.input(3); 
    assert_eq!( live.output(),  3); 
    assert_eq!( live.in_out(4), 4); 
    assert_eq!( live.add(5),    8); 
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
        pub fn add(&mut self, v:T) -> T{self.0 += v;self.0.clone()}
    }
    tokio::runtime::Runtime::new()
    .unwrap()
    .block_on(
        async {
        let mut live = ActorLive::new(0);
        live.input(3).await; 
        assert_eq!( live.output().await,  3); 
        assert_eq!( live.in_out(4).await, 4); 
        assert_eq!( live.add(5).await,    8); 
    });
}

#[test]
fn actor_tokio_unbounded() {
    pub struct Actor<T>(T);
    #[life(channel= 0,lib="tokio",debut)]
    impl <T:Clone> Actor <T> 
      where T: core::ops::AddAssign + std::fmt::Debug,
    {
        pub fn new(v:T) -> Self{Self(v)}
        pub fn input(&mut self, v:T){self.0 = v}
        pub fn output(&self)->T{self.0.clone()}
        pub fn in_out(&self,v:T)->T{v}
        pub fn add(&mut self, v:T) -> T{self.0 += v;self.0.clone()}
    }
    tokio::runtime::Runtime::new()
    .unwrap()
    .block_on( async {
        let mut live = ActorLive::new(0);
        live.input(3).await; 
        assert_eq!( live.output().await,  3); 
        assert_eq!( live.in_out(4).await, 4); 
        assert_eq!( live.add(5).await,    8); 
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
        pub fn add(&mut self, v:T) -> T{self.0 += v;self.0.clone()}
    }
    async_std::task::block_on(async {
            let mut live = ActorLive::new(0);
            live.input(3).await; 
            assert_eq!( live.output().await,  3); 
            assert_eq!( live.in_out(4).await, 4); 
            assert_eq!( live.add(5).await,    8); 
    });
}
 #[test]
fn actor_async_std_unbounded() {
    pub struct Actor<T>(T);
    #[life(channel=0,lib="async_std",debut)]
    impl <T:Clone> Actor <T> 
      where T: core::ops::AddAssign + std::fmt::Debug,
    {
        pub fn new(v:T) -> Self{Self(v)}
        pub fn input(&mut self, v:T){self.0 = v}
        pub fn output(&self)->T{self.0.clone()}
        pub fn in_out(&self,v:T)->T{v}
        pub fn add(&mut self, v:T) -> T{self.0 += v;self.0.clone()}
    }
    async_std::task::block_on(async {
            let mut live = ActorLive::new(0);
            live.input(3).await; 
            assert_eq!( live.output().await,  3); 
            assert_eq!( live.in_out(4).await, 4); 
            assert_eq!( live.add(5).await,    8); 
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
        pub fn add(&mut self, v:T) -> T{self.0 += v;self.0.clone()}
    }
    async_std::task::block_on(async {
            let mut live = ActorLive::new(0);
            live.input(3).await; 
            assert_eq!( live.output().await,  3); 
            assert_eq!( live.in_out(4).await, 4); 
            assert_eq!( live.add(5).await,    8); 
    });
}
#[test]
fn actor_smol_unbounded() {
    pub struct Actor<T>(T);
    #[life(channel=0,lib="smol",debut)]
    impl <T:Clone> Actor <T> 
      where T: core::ops::AddAssign + std::fmt::Debug,
    {
        pub fn new(v:T) -> Self{Self(v)}
        pub fn input(&mut self, v:T){self.0 = v}
        pub fn output(&self)->T{self.0.clone()}
        pub fn in_out(&self,v:T)->T{v}
        pub fn add(&mut self, v:T) -> T{self.0 += v;self.0.clone()}
    }
    async_std::task::block_on(async {
            let mut live = ActorLive::new(0);
            live.input(3).await; 
            assert_eq!( live.output().await,  3); 
            assert_eq!( live.in_out(4).await, 4); 
            assert_eq!( live.add(5).await,    8); 
    });
}



