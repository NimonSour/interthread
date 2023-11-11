use interthread::actor as life;

//STD
#[test]
fn actor_sync_bounded() {
    pub struct MyTupleStruct(i8,i8,i8);
    pub struct MyStruct{a:i8,b:i8,c:i8}


    pub struct Actor(i8);
    #[life(channel=3)]
    impl Actor {
        pub fn new() -> Self{Self(0)}
        pub fn input(&mut self, v:i8){self.0 = v}
        pub fn output(&self)->i8{self.0}
        pub fn in_out_tuple(&self,(a,b,c):(i8,i8,i8))->i8{a+b+c}
        pub fn in_out_struct(&self,MyStruct{a,b,c}:MyStruct)->i8{a+b+c}
        pub fn in_out_tuple_struct(&self,MyTupleStruct(a,b,c):MyTupleStruct)->i8{a+b+c}
        pub fn add(&mut self, mut v:i8) -> i8{v = v;self.0 += v;self.0}
    }
    let mut live = ActorLive::new();
    live.input(3); 
    assert_eq!( live.output(),  3); 
    assert_eq!( live.in_out_tuple((1,2,3)), 6); 
    assert_eq!( live.in_out_struct(MyStruct{a:1,b:2,c:3}), 6); 
    assert_eq!( live.in_out_tuple_struct(MyTupleStruct(1,2,3)), 6); 
    assert_eq!( live.add(5),    8); 
}
 

// TOKIO
#[test]
fn actor_tokio_bounded() {
    pub struct MyTupleStruct(i8,i8,i8);
    pub struct MyStruct{a:i8,b:i8,c:i8}


    pub struct Actor(i8);
    #[life(channel=3,lib="tokio")]
    impl Actor {
        pub fn new() -> Self{Self(0)}
        pub fn input(&mut self, v:i8){self.0 = v}
        pub async fn in_out_tuple(&self,(a,b,c):(i8,i8,i8))->i8{a+b+c}
        pub async fn in_out_struct(&self,MyStruct{a,b,c}:MyStruct)->i8{a+b+c}
        pub async fn in_out_tuple_struct(&self,MyTupleStruct(a,b,c):MyTupleStruct)->i8{a+b+c}
        pub async fn add(&mut self, mut v:i8) -> i8{v = v;self.0 += v;self.0}
    }
    tokio::runtime::Runtime::new()
    .unwrap()
    .block_on(
        async {
        let mut live = ActorLive::new();
        live.input(3).await; 
        assert_eq!( live.in_out_tuple((1,2,3)).await, 6); 
        assert_eq!( live.in_out_struct(MyStruct{a:1,b:2,c:3}).await, 6); 
        assert_eq!( live.in_out_tuple_struct(MyTupleStruct(1,2,3)).await, 6); 
        assert_eq!( live.add(5).await,    8); 
    });
}


//ASYNC-STD
#[test]
fn actor_async_std_bounded() {
    pub struct MyTupleStruct(i8,i8,i8);
    pub struct MyStruct{a:i8,b:i8,c:i8}

    pub struct Actor(i8);
    #[life(channel=3,lib="async_std")]
    impl Actor {
        pub fn new() -> Self{Self(0)}
        pub fn input(&mut self, v:i8){self.0 = v}
        pub fn output(&self)->i8{self.0}
        pub async fn in_out_tuple(&self,(a,b,c):(i8,i8,i8))->i8{a+b+c}
        pub async fn in_out_struct(&self,MyStruct{a,b,c}:MyStruct)->i8{a+b+c}
        pub async fn in_out_tuple_struct(&self,MyTupleStruct(a,b,c):MyTupleStruct)->i8{a+b+c}
        pub async fn add(&mut self, mut v:i8) -> i8{v = v;self.0 += v;self.0}
    }
    async_std::task::block_on(async {
            let mut live = ActorLive::new();
            live.input(3).await; 
            assert_eq!( live.in_out_tuple((1,2,3)).await, 6); 
            assert_eq!( live.in_out_struct(MyStruct{a:1,b:2,c:3}).await, 6); 
            assert_eq!( live.in_out_tuple_struct(MyTupleStruct(1,2,3)).await, 6); 
            assert_eq!( live.add(5).await,    8); 
    });
}

#[test]
fn actor_async_std_bounded_gen() {
    pub struct MyTupleStruct<A:Into<i64>,B:Into<i64>,C:Into<i64>>(A,B,C);
    pub struct MyStruct<A,B,C>
    where 
        A:Into<i64>,
        B:Into<i64>,
        C:Into<i64>,
    {a:A,b:B,c:C}
    
    pub struct Actor(i64);
    #[life(channel=3,lib="async_std")]
    impl Actor { 

        pub fn new() -> Self{Self(0)}
        pub fn input(&mut self, v:i8){self.0 = v.into();}
        pub fn output(&self)->i64{self.0}
        pub async fn in_out_tuple(&self,(a,b,c):(i8,i8,i8))->i8{a+b+c}
        pub async fn in_out_struct<A,B,C>(&self,MyStruct{a,b,c}:MyStruct<A,B,C>)->i64
        where 
        A:Into<i64>,
        B:Into<i64>,
        C:Into<i64>,
        { let  mut s = 0i64; 
            s = s + a.into();
            s = s + b.into();
            s = s + c.into(); s}
        pub async fn in_out_tuple_struct<A,B,C>(&self,MyTupleStruct(a,b,c):MyTupleStruct<A,B,C>)->i64
        where 
        A:Into<i64>,
        B:Into<i64>,
        C:Into<i64>,
        { let  mut s = 0i64; 
            s = s + a.into();
            s = s + b.into();
            s = s + c.into(); s }

        pub async fn add<F:Into<i64> +std::ops::AddAssign>(&mut self, v:F) -> i64{
            self.0 += v.into();
            self.0
        }
    }
    async_std::task::block_on(async {
            let mut live = ActorLive::new();
            live.input(3).await; 
            assert_eq!( live.in_out_tuple((1,2,3)).await, 6); 
            assert_eq!( live.in_out_struct(MyStruct{a:1i8,b:2i16,c:3i32}).await, 6); 
            assert_eq!( live.in_out_tuple_struct(MyTupleStruct(1i8,2i16,3i32)).await, 6); 
            assert_eq!( live.add(5i8).await,    8); 
    });
}