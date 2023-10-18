use interthread::actor as life;
//STD INTER

#[test]
fn actor_sync_unbound_default_tuple_struct() {


    pub struct MyType(i8);

    pub struct Actor(i8);
    #[life]
    impl Actor {
        pub fn new() -> Self{Self(0)}
        pub fn input(&mut self, v:i8){self.0 = v}
        pub fn output(&self,)->i8{self.0}
        pub fn in_out(&self,MyType(v):MyType)->i8{v}
        pub fn add(&mut self, v:i8) -> i8{self.0 += v;self.0}
    }
    let mut live = ActorLive::new();
    live.input(3); 
    assert_eq!( live.output(),  3); 
    assert_eq!( live.in_out(MyType(4)), 4); 
    assert_eq!( live.add(5),    8); 
}

#[test]
fn actor_sync_unbound_default_plus_tuple() {


    pub struct MyType(i8);

    pub struct Actor(i8);
    #[life]
    impl Actor {
        pub fn new() -> Self{Self(0)}
        pub fn input(&mut self, v:i8){self.0 = v}
        pub fn output(&self,)->i8{self.0}
        pub fn in_out(&self,MyType(v):MyType)->i8{v}
        pub fn add(&mut self, (a,b,c):(i8,i8,i8)) -> i8{
            self.0 += a;
            self.0 += b;
            self.0 += c;
            self.0
        }
    }
    let mut live = ActorLive::new();
    live.input(3); 
    assert_eq!( live.output(),  3); 
    assert_eq!( live.in_out(MyType(4)), 4); 
    assert_eq!( live.add((1,1,3)),    8); 
}

#[test]
fn actor_sync_unbound_default_plus_struct() {


    pub struct MyType{a:i8,b:i8,c:i8};

    pub struct Actor(i8);
    #[life]
    impl Actor {
        pub fn new() -> Self{Self(0)}
        pub fn input(&mut self, v:i8){self.0 = v}
        pub fn output(&self,)->i8{self.0}
        pub fn in_out(&self,MyType{a,b,c}:MyType)->i8{a+b+c}
        pub fn add(&mut self, (a,b,c):(i8,i8,i8)) -> i8{
            self.0 += a;
            self.0 += b;
            self.0 += c;
            self.0
        }
    }
    let mut live = ActorLive::new();
    live.input(3); 
    assert_eq!( live.output(),  3); 
    assert_eq!( live.in_out(MyType{a:1,b:2,c:3}), 6); 
    assert_eq!( live.add((1,1,3)),    8); 
}