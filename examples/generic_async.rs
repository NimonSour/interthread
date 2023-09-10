


use interthread::actor as life;
    pub struct Actor<T>(T);
    #[life(channel=3,lib="tokio",id)]
    impl <T:Clone> Actor <T> 
      where T: core::ops::AddAssign + std::fmt::Debug,
    {
        pub fn new(v:T) -> Self{Self(v)}
        pub fn input(&mut self, v:T){self.0 = v}
        pub fn output(&self)->T{self.0.clone()}
        pub fn in_out(&self,v:T)->T{v}
        pub fn add(&mut self, v:T) -> T{self.0 += v;self.0.clone()}
    }

// #[interthread::example(main(path="examples/generic_async.rs"))]
#[tokio::main]
async fn main() {

    let mut live = ActorLive::new(0);
    live.input(3).await; 
    assert_eq!( live.output().await,  3); 
    assert_eq!( live.in_out(4).await, 4); 
    assert_eq!( live.add(5).await,    8); 
}


