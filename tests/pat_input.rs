
use interthread::actor as life;

//STD
#[test]
fn actor_sync_bounded() {
    struct MyTupleStruct(i8,i8,i8);
    struct MyStruct{a:i8,b:i8,c:i8}


    struct Actor(i8);
    #[life(channel=3)]
    impl Actor {
        pub fn new() -> Self{Self(0)}
        pub fn input(&mut self, v:i8){self.0 = v}
        pub fn output(&self)->i8{self.0}
        pub fn in_out_tuple(&self,(a,b,c):(i8,i8,i8))->i8{a+b+c}
        pub fn in_out_struct(&self,MyStruct{a,b,c}:MyStruct)->i8{a+b+c}
        pub fn in_out_tuple_struct(&self,MyTupleStruct(a,b,c):MyTupleStruct)->i8{a+b+c}
        pub fn in_out_array(&self,[a,b,c]:[i8;3])->i8{a+b+c}
        pub fn in_out_array_with_slice(&self,[a,ref i @ ..,d]:[i8;4])->i8{let sum: i8 = i.iter().sum(); a+d + sum }
        pub fn add(&mut self, mut v:i8) -> i8{v = v;self.0 += v;self.0}
    }
    let mut live = ActorLive::new();
    live.input(3); 
    assert_eq!( live.output(),  3); 
    assert_eq!( live.in_out_tuple((1,2,3)), 6); 
    assert_eq!( live.in_out_struct(MyStruct{a:1,b:2,c:3}), 6); 
    assert_eq!( live.in_out_tuple_struct(MyTupleStruct(1,2,3)), 6); 
    assert_eq!( live.in_out_array([1,2,3i8]), 6); 
    assert_eq!( live.in_out_array_with_slice([1,2,3,4i8]), 10); 
    assert_eq!( live.add(5),    8); 
}
 

// TOKIO
#[test]
fn actor_tokio_bounded() {
    struct MyTupleStruct(i8,i8,i8);
    struct MyStruct{a:i8,b:i8,c:i8}


    struct Actor(i8);
    #[life(channel=3,lib="tokio")]
    impl Actor {
        pub fn new() -> Self{Self(0)}
        pub fn input(&mut self, v:i8){self.0 = v}
        pub async fn in_out_tuple(&self,(a,b,c):(i8,i8,i8))->i8{a+b+c}
        pub async fn in_out_struct(&self,MyStruct{a,b,c}:MyStruct)->i8{a+b+c}
        pub async fn in_out_tuple_struct(&self,MyTupleStruct(a,b,c):MyTupleStruct)->i8{a+b+c}
        pub async fn in_out_array(&self,[a,b,c]:[i8;3])->i8{a+b+c}
        pub async fn in_out_array_with_slice(&self,[a,ref i @ ..,d]:[i8;4])->i8{let sum: i8 = i.iter().sum(); a+d + sum }
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
        assert_eq!( live.in_out_array([1,2,3i8]).await, 6); 
        assert_eq!( live.in_out_array_with_slice([1,2,3,4i8]).await, 10);  
        assert_eq!( live.add(5).await,    8); 
    });
}


//ASYNC-STD
#[test]
fn actor_async_std_bounded() {
    struct MyTupleStruct(i8,i8,i8);
    struct MyStruct{a:i8,b:i8,c:i8}

    struct Actor(i8);
    #[life(channel=3,lib="async_std")]
    impl Actor {
        pub fn new() -> Self{Self(0)}
        pub fn input(&mut self, v:i8){self.0 = v}
        pub fn output(&self)->i8{self.0}
        pub async fn in_out_tuple(&self,(a,b,c):(i8,i8,i8))->i8{a+b+c}
        pub async fn in_out_struct(&self,MyStruct{a,b,c}:MyStruct)->i8{a+b+c}
        pub async fn in_out_tuple_struct(&self,MyTupleStruct(a,b,c):MyTupleStruct)->i8{a+b+c}
        pub async fn in_out_array(&self,[a,b,c]:[i8;3])->i8{a+b+c}
        pub async fn in_out_array_with_slice(&self,[a,ref i @ ..,d]:[i8;4])->i8{let sum: i8 = i.iter().sum(); a+d + sum }
        pub async fn add(&mut self, mut v:i8) -> i8{v = v;self.0 += v;self.0}
    }
    async_std::task::block_on(async {
            let mut live = ActorLive::new();
            live.input(3).await; 
            assert_eq!( live.in_out_tuple((1,2,3)).await, 6); 
            assert_eq!( live.in_out_struct(MyStruct{a:1,b:2,c:3}).await, 6); 
            assert_eq!( live.in_out_tuple_struct(MyTupleStruct(1,2,3)).await, 6); 
            assert_eq!( live.in_out_array([1,2,3i8]).await, 6); 
            assert_eq!( live.in_out_array_with_slice([1,2,3,4i8]).await, 10); 
            assert_eq!( live.add(5).await,    8); 
    });
}

#[test]
fn actor_async_std_bounded_gen() {
    struct MyTupleStruct<A:Into<i64>,B:Into<i64>,C:Into<i64>>(A,B,C);
    struct MyStruct<A,B,C>
    where 
        A:Into<i64>,
        B:Into<i64>,
        C:Into<i64>,
    {a:A,b:B,c:C}
    
    struct Actor(i64);
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



// ------ patterns with Rest
//STD
#[test]
fn actor_sync_bounded_rest() {
    struct MyTupleStruct(i8,i8,i8);
    struct MyStruct{a:i8,_b:i8,c:i8}


    struct Actor(i8);
    #[life(channel=3)]
    impl Actor {
        // initiate from a pattern 
        pub fn new( MyTupleStruct(a,b,..): MyTupleStruct ) -> Self{
            Self( a+b )
        }
        pub fn input(&mut self, v:i8){self.0 = v}
        pub fn output(&self)->i8{self.0}
        pub fn in_out_tuple(&self,(a,b,..):(i8,i8,i8))->i8{a+b}
        pub fn in_out_struct(&self,MyStruct{a,c,..}:MyStruct)->i8{a+c}
        pub fn in_out_tuple_struct(&self,MyTupleStruct(a,b,..):MyTupleStruct)->i8{a+b}
        pub fn in_out_tuple_struct_empty(&self,MyTupleStruct(..):MyTupleStruct)->i8{0i8}
        pub fn in_out_array(&self,[a,b,..]: [i8;4])->i8{a+b}
        pub fn add(&mut self, mut v:i8) -> i8{v = v;self.0 += v;self.0}
    }

    let mut live = ActorLive::new( MyTupleStruct(1,2,3));
    live.input(3); 

    assert_eq!( live.output(),  3); 
    assert_eq!( live.in_out_tuple((1,2,3)), 3); 
    assert_eq!( live.in_out_struct(MyStruct{a:1,_b:2,c:3}), 4); 
    assert_eq!( live.in_out_tuple_struct(MyTupleStruct(1,2,3)), 3); 
    assert_eq!( live.in_out_tuple_struct_empty(MyTupleStruct(1,2,3)), 0);  
    assert_eq!( live.in_out_array([1,2,3,4i8]), 3);  
    assert_eq!( live.add(5),    8);


}
 


#[test]
fn actor_async_std_bounded_gen_rest() {
    struct MyTupleStruct<A,B,C>(A,B,C);
    struct MyStruct<A,B,C>{a:A,_b:B,c:C}
    
    struct Actor(i64);
    #[life(channel=3,lib="async_std")]
    impl Actor { 

        pub fn new() -> Self{Self(0)}
        pub fn input(&mut self, v:i8){self.0 = v.into();}
        pub fn output(&self)->i64{self.0}
        pub async fn in_out_tuple(&self,(a,b,..):(i8,i8,i8))->i8{a+b}
        pub async fn in_out_struct<A,B,C>(&self,MyStruct{a,c,..}:MyStruct<A,B,C>)->i64
        where 
        A:Into<i64>,
        B:Into<i64>,
        C:Into<i64>,
        { let  mut s = 0i64; 
            s = s + a.into();
            s = s + c.into(); s}
        pub async fn in_out_tuple_struct<A,B,C>(&self,MyTupleStruct(a,b,..):MyTupleStruct<A,B,C>)->i64
        where 
        A:Into<i64>,
        B:Into<i64>,
        C:Into<i64>,
        { let  mut s = 0i64; 
            s = s + a.into();
            s = s + b.into(); s }

        pub async fn in_out_tuple_struct_empty<A,B,C>(&self,MyTupleStruct(..):MyTupleStruct<A,B,C>)->i64
        where 
        A:Into<i64>,
        B:Into<i64>,
        C:Into<i64>,
        { 0i64}
        pub async fn add<F:Into<i64> +std::ops::AddAssign>(&mut self, v:F) -> i64{
            self.0 += v.into();
            self.0
        }
    }

    async_std::task::block_on(async {
            let mut live = ActorLive::new();
            live.input(3).await; 
            assert_eq!( live.in_out_tuple((1,2,3)).await, 3); 
            assert_eq!( live.in_out_struct(MyStruct{a:1i8,_b:2i16,c:3i32}).await, 4); 
            assert_eq!( live.in_out_tuple_struct(MyTupleStruct(1i8,2i16,3i32)).await, 3); 
            assert_eq!( live.in_out_tuple_struct_empty(MyTupleStruct(1i8,2i16,3i32)).await, 0); 
            assert_eq!( live.add(5i8).await,    8); 
    });
}


