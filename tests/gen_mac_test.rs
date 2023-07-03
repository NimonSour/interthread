


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


#[test]
fn self_word_in_arg_type() {
    #[derive(Debug,Clone,PartialEq, Eq)]
    pub struct ActorSelf(i8);
    #[life(channel=3)]
    impl ActorSelf {
        pub fn new() -> Self{Self(0)}
        pub fn input(&mut self, v:i8){self.0 = v}
        pub fn output(&self)->i8{self.0}
        pub fn in_out(&self,v:ActorSelf)->ActorSelf{v}
        pub fn add(&mut self, v:i8) -> i8{self.0 += v;self.0}
    }
    let a = ActorSelf::new();
    let mut live = ActorSelfLive::new();
    live.input(3); 
    assert_eq!( live.output(),  3); 
    assert_eq!( live.in_out(a.clone()), a); 
    assert_eq!( live.add(5),    8); 
}


// ID Inter 
#[test]
fn id_actor_sync_inter(){
    pub struct Actor(i8);
    #[life(id=true)]
    impl Actor {
        pub fn new() -> Self{Self(0)}
        pub fn input(&mut self, v:i8){self.0 = v}
        pub fn output(&self)->i8{self.0}
        pub fn in_out(&self,v:i8)->i8{v}
        pub fn add(&mut self, v:i8) -> i8{self.0 += v;self.0}
    }

    let actor1 = ActorLive::new();
    let actor2 = ActorLive::new();
    let actor3 = ActorLive::new();



    // Equality comparisons
    assert_eq!(actor1 == actor2, false); 
    assert_eq!(actor2 == actor3, false); 

    // Not Eq Comparison
    assert_eq!(actor1 != actor2, true); 
    assert_eq!(actor2 != actor3, true); 

    // Lesser Comparison
    assert_eq!(actor1 < actor2, false); 
    assert_eq!(actor1 < actor3, false); 
    assert_eq!(actor2 < actor3, false); 

    // Greater Comparison
    assert_eq!(actor2 > actor1, false); 
    assert_eq!(actor3 > actor1, false); 
    assert_eq!(actor3 > actor2, false); 

    // Lesser or Equal Comparison
    assert_eq!(actor2 <= actor1, true); 
    assert_eq!(actor3 <= actor1, true); 
    assert_eq!(actor3 <= actor2, true); 

    // Greater or Equal Comparison
    assert_eq!(actor1 >= actor2, true); 
    assert_eq!(actor1 >= actor3, true); 
    assert_eq!(actor2 >= actor3, true); 



    // CLONE
    let actor3_c = actor3.clone();
    let actor2_c = actor2.clone();
    let actor1_c = actor1.clone();

    // Equality comparisons
    assert_eq!(actor1 == actor1_c, true);
    assert_eq!(actor2 == actor2_c, true);
    assert_eq!(actor3 == actor3_c, true);

    assert_eq!(actor1_c == actor2_c, false); 
    assert_eq!(actor2_c == actor3_c, false); 

    // Not Eq Comparison
    assert_eq!(actor1_c != actor2_c, true); 
    assert_eq!(actor2_c != actor3_c, true); 

    // Lesser Compari_cson
    assert_eq!(actor2_c < actor1_c, true); 
    assert_eq!(actor3_c < actor1_c, true); 
    assert_eq!(actor3_c < actor2_c, true); 

    // Greater Compar_cison
    assert_eq!(actor1_c > actor2_c, true); 
    assert_eq!(actor1_c > actor3_c, true); 
    assert_eq!(actor2_c > actor3_c, true);

    // Lesser or Equa_cl Comparison
    assert_eq!(actor1_c <= actor2_c, false); 
    assert_eq!(actor1_c <= actor3_c, false); 
    assert_eq!(actor2_c <= actor3_c, false); 

    // Greater or Equ_cal Comparison
    assert_eq!(actor2_c >= actor1_c, false); 
    assert_eq!(actor3_c >= actor1_c, false); 
    assert_eq!(actor3_c >= actor2_c, false); 
    

    // DEBUT TIME

    // Lesser Comparison
    assert_eq!(actor1.debut < actor2.debut, !false); 
    assert_eq!(actor1.debut < actor3.debut, !false); 
    assert_eq!(actor2.debut < actor3.debut, !false); 

    // Greater Comparison
    assert_eq!(actor2.debut > actor1.debut, !false); 
    assert_eq!(actor3.debut > actor1.debut, !false); 
    assert_eq!(actor3.debut > actor2.debut, !false); 

    // Lesser or Equal Comparison
    assert_eq!(actor2.debut <= actor1.debut, !true); 
    assert_eq!(actor3.debut <= actor1.debut, !true); 
    assert_eq!(actor3.debut <= actor2.debut, !true); 
    
    // Greater or Equal Comparison
    assert_eq!(actor1.debut >= actor2.debut, !true); 
    assert_eq!(actor1.debut >= actor3.debut, !true); 
    assert_eq!(actor2.debut >= actor3.debut, !true); 
}


// ID STD
fn id_actor_sync_bounded() {
    pub struct Actor(i8);
    #[life(channel=3,id=true)]
    impl Actor {
        pub fn new() -> Self{Self(0)}
        pub fn input(&mut self, v:i8){self.0 = v}
        pub fn output(&self)->i8{self.0}
        pub fn in_out(&self,v:i8)->i8{v}
        pub fn add(&mut self, v:i8) -> i8{self.0 += v;self.0}
    }
    let actor1 = ActorLive::new();
    let actor2 = ActorLive::new();
    let actor3 = ActorLive::new();



    // Equality comparisons
    assert_eq!(actor1 == actor2, false); 
    assert_eq!(actor2 == actor3, false); 

    // Not Eq Comparison
    assert_eq!(actor1 != actor2, true); 
    assert_eq!(actor2 != actor3, true); 

    // Lesser Comparison
    assert_eq!(actor1 < actor2, false); 
    assert_eq!(actor1 < actor3, false); 
    assert_eq!(actor2 < actor3, false); 

    // Greater Comparison
    assert_eq!(actor2 > actor1, false); 
    assert_eq!(actor3 > actor1, false); 
    assert_eq!(actor3 > actor2, false); 

    // Lesser or Equal Comparison
    assert_eq!(actor2 <= actor1, true); 
    assert_eq!(actor3 <= actor1, true); 
    assert_eq!(actor3 <= actor2, true); 

    // Greater or Equal Comparison
    assert_eq!(actor1 >= actor2, true); 
    assert_eq!(actor1 >= actor3, true); 
    assert_eq!(actor2 >= actor3, true); 



    // CLONE
    let actor3_c = actor3.clone();
    let actor2_c = actor2.clone();
    let actor1_c = actor1.clone();

    // Equality comparisons
    assert_eq!(actor1 == actor1_c, true);
    assert_eq!(actor2 == actor2_c, true);
    assert_eq!(actor3 == actor3_c, true);

    assert_eq!(actor1_c == actor2_c, false); 
    assert_eq!(actor2_c == actor3_c, false); 

    // Not Eq Comparison
    assert_eq!(actor1_c != actor2_c, true); 
    assert_eq!(actor2_c != actor3_c, true); 

    // Lesser Compari_cson
    assert_eq!(actor2_c < actor1_c, true); 
    assert_eq!(actor3_c < actor1_c, true); 
    assert_eq!(actor3_c < actor2_c, true); 

    // Greater Compar_cison
    assert_eq!(actor1_c > actor2_c, true); 
    assert_eq!(actor1_c > actor3_c, true); 
    assert_eq!(actor2_c > actor3_c, true);

    // Lesser or Equa_cl Comparison
    assert_eq!(actor1_c <= actor2_c, false); 
    assert_eq!(actor1_c <= actor3_c, false); 
    assert_eq!(actor2_c <= actor3_c, false); 

    // Greater or Equ_cal Comparison
    assert_eq!(actor2_c >= actor1_c, false); 
    assert_eq!(actor3_c >= actor1_c, false); 
    assert_eq!(actor3_c >= actor2_c, false); 
    

    // DEBUT TIME

    // Lesser Comparison
    assert_eq!(actor1.debut < actor2.debut, !false); 
    assert_eq!(actor1.debut < actor3.debut, !false); 
    assert_eq!(actor2.debut < actor3.debut, !false); 

    // Greater Comparison
    assert_eq!(actor2.debut > actor1.debut, !false); 
    assert_eq!(actor3.debut > actor1.debut, !false); 
    assert_eq!(actor3.debut > actor2.debut, !false); 

    // Lesser or Equal Comparison
    assert_eq!(actor2.debut <= actor1.debut, !true); 
    assert_eq!(actor3.debut <= actor1.debut, !true); 
    assert_eq!(actor3.debut <= actor2.debut, !true); 
    
    // Greater or Equal Comparison
    assert_eq!(actor1.debut >= actor2.debut, !true); 
    assert_eq!(actor1.debut >= actor3.debut, !true); 
    assert_eq!(actor2.debut >= actor3.debut, !true);
}

//ID TOKIO
#[test]
fn id_actor_tokio_bounded() {
    pub struct Actor(i8);
    #[life(channel=3,lib="tokio",id=true)]
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

            let actor1 = ActorLive::new();
            let actor2 = ActorLive::new();
            let actor3 = ActorLive::new();
        
        
        
            // Equality comparisons
            assert_eq!(actor1 == actor2, false); 
            assert_eq!(actor2 == actor3, false); 
        
            // Not Eq Comparison
            assert_eq!(actor1 != actor2, true); 
            assert_eq!(actor2 != actor3, true); 
        
            // Lesser Comparison
            assert_eq!(actor1 < actor2, false); 
            assert_eq!(actor1 < actor3, false); 
            assert_eq!(actor2 < actor3, false); 
        
            // Greater Comparison
            assert_eq!(actor2 > actor1, false); 
            assert_eq!(actor3 > actor1, false); 
            assert_eq!(actor3 > actor2, false); 
        
            // Lesser or Equal Comparison
            assert_eq!(actor2 <= actor1, true); 
            assert_eq!(actor3 <= actor1, true); 
            assert_eq!(actor3 <= actor2, true); 
        
            // Greater or Equal Comparison
            assert_eq!(actor1 >= actor2, true); 
            assert_eq!(actor1 >= actor3, true); 
            assert_eq!(actor2 >= actor3, true); 
        
        
        
            // CLONE
            let actor3_c = actor3.clone();
            let actor2_c = actor2.clone();
            let actor1_c = actor1.clone();
        
            // Equality comparisons
            assert_eq!(actor1 == actor1_c, true);
            assert_eq!(actor2 == actor2_c, true);
            assert_eq!(actor3 == actor3_c, true);
        
            assert_eq!(actor1_c == actor2_c, false); 
            assert_eq!(actor2_c == actor3_c, false); 
        
            // Not Eq Comparison
            assert_eq!(actor1_c != actor2_c, true); 
            assert_eq!(actor2_c != actor3_c, true); 
        
            // Lesser Compari_cson
            assert_eq!(actor2_c < actor1_c, true); 
            assert_eq!(actor3_c < actor1_c, true); 
            assert_eq!(actor3_c < actor2_c, true); 
        
            // Greater Compar_cison
            assert_eq!(actor1_c > actor2_c, true); 
            assert_eq!(actor1_c > actor3_c, true); 
            assert_eq!(actor2_c > actor3_c, true);
        
            // Lesser or Equa_cl Comparison
            assert_eq!(actor1_c <= actor2_c, false); 
            assert_eq!(actor1_c <= actor3_c, false); 
            assert_eq!(actor2_c <= actor3_c, false); 
        
            // Greater or Equ_cal Comparison
            assert_eq!(actor2_c >= actor1_c, false); 
            assert_eq!(actor3_c >= actor1_c, false); 
            assert_eq!(actor3_c >= actor2_c, false); 
            
        
            // DEBUT TIME
        
            // Lesser Comparison
            assert_eq!(actor1.debut < actor2.debut, !false); 
            assert_eq!(actor1.debut < actor3.debut, !false); 
            assert_eq!(actor2.debut < actor3.debut, !false); 
        
            // Greater Comparison
            assert_eq!(actor2.debut > actor1.debut, !false); 
            assert_eq!(actor3.debut > actor1.debut, !false); 
            assert_eq!(actor3.debut > actor2.debut, !false); 
        
            // Lesser or Equal Comparison
            assert_eq!(actor2.debut <= actor1.debut, !true); 
            assert_eq!(actor3.debut <= actor1.debut, !true); 
            assert_eq!(actor3.debut <= actor2.debut, !true); 
            
            // Greater or Equal Comparison
            assert_eq!(actor1.debut >= actor2.debut, !true); 
            assert_eq!(actor1.debut >= actor3.debut, !true); 
            assert_eq!(actor2.debut >= actor3.debut, !true); 
    });
}

