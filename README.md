# interthread

The [`actor`](https://docs.rs/interthread/latest/interthread/attr.actor.html)  macro provided by this crate automates the implementation of an Actor Model for a given struct. It handles the intricacies of message routing and synchronization, empowering developers to swiftly prototype the core functionality of their applications.
This fast sketching capability is
particularly useful when exploring different design options, 
experimenting with concurrency models, or implementing 
proof-of-concept systems. Not to mention, the cases where 
the importance of the program lies in the result of its work 
rather than its execution.

### Examples


Filename: Cargo.toml

```text
[dependencies]
interthread = "1.1.6"
oneshot     = "0.1.5" 
```

Filename: main.rs
```rust

pub struct MyActor {
    value: i8,
}

#[interthread::actor(channel=2)] // <-  this is it 
impl MyActor {

    pub fn new( v: i8 ) -> Self {
       Self { value: v } 
    }
    pub fn increment(&mut self) {
        self.value += 1;
    }
    pub fn add_number(&mut self, num: i8) -> i8 {
        self.value += num;
        self.value
    }
    pub fn get_value(&self) -> i8 {
        self.value
    }
}

// uncomment to see the generated code
//#[interthread::example(path="src/main.rs")] 
fn main() {

    let actor = MyActorLive::new(5);

    let mut actor_a = actor.clone();
    let mut actor_b = actor.clone();

    let handle_a = std::thread::spawn( move || { 
    actor_a.increment();
    });

    let handle_b = std::thread::spawn( move || {
    actor_b.add_number(5)
    });

    let _  = handle_a.join();
    let hb = handle_b.join().unwrap();

    // we never know which thread will
    // be first to call the actor so
    // hb = 10 or 11
    assert!(hb >= 10);

    assert_eq!(actor.get_value(), 11);
}

```
 

The same example can be run in 
[tokio](https://crates.io/crates/tokio),
[async-std](https://crates.io/cratesasync-std), 
and [smol](https://crates.io/cratessmol), 
with the only difference being that the methods will 
be marked as `async` and need to be `await`ed for 
asynchronous execution.


### Examples


Filename: Cargo.toml

```text
[dependencies]
interthread = "1.1.6"
oneshot     = "0.1.5" 
```

Filename: main.rs
```rust

pub struct MyActor {
    value: i8,
}

#[interthread::actor(channel=2,lib="tokio",id)] // <-  one line )
impl MyActor {

    pub fn new( v: i8 ) -> Self {
       Self { value: v } 
    }
    // if the "lib" is defined
    // object methods can be "async" 
    pub async fn increment(&mut self) {
        self.value += 1;
    }
    pub fn add_number(&mut self, num: i8) -> i8 {
        self.value += num;
        self.value
    }
    pub fn get_value(&self) -> i8 {
        self.value
    }
}

//  #[interthread::example(path="examples/intro_tokio.rs")]  

#[tokio::main]
async fn main() {

    let actor = MyActorLive::new(5);

    let mut actor_a = actor.clone();
    let mut actor_b = actor.clone();

    let handle_a = tokio::spawn( async move { 
    actor_a.increment().await;
    });

    let handle_b = tokio::spawn( async move {
    actor_b.add_number(5).await
    });

    let _  = handle_a.await;
    let hb = handle_b.await.unwrap();

    // hb = 10 or 11
    assert!(hb >= 10);

    assert_eq!(actor.get_value().await, 11);
}

```

While the [`actor`](https://docs.rs/interthread/latest/interthread/attr.actor.html) provides full support for generic types, but there are certain limitations to its flexibility. In Rust, clones of a generic type instance cannot differ. This means that once an instance of `ActorLive<A, B>` becomes `ActorLive<u8, u16>`, all clones will have the same generic type.

To address this behavior (which is by design, not an issue), one can manually adjust the inputs of the `Live` methods to the desired generic types, as demonstrated in the example below.
 

### Examples


Filename: Cargo.toml

```text
[dependencies]
interthread = "1.1.6"
oneshot     = "0.1.5" 
```

Filename: main.rs
```rust


pub struct Actor {
    str: String,
}

// writes to file when 'edit' is used
// in conjuction with 'file' argument

#[interthread::actor(channel=2,
    edit(live(imp(concat))))
]
impl Actor 

{
    pub fn new() -> Self {
        Actor { 
            str: String::new(), 
        }
    }

    pub fn concat(&mut self, s: String) {
        self.str += &s;
    }

    pub fn get_value(&self) -> String {
        self.str.clone()
    }
}


//++++++++++++++++++[ Interthread  Write to File ]+++++++++++++++++//
// Object Name   : MaunActor  
// Initiated By  : #[interthread::actor(channel=2,file="path/to/this/file.rs",edit(live(imp(concat))))]  


impl ActorLive {
    // pub fn concat(&mut self, s: String) {
    pub fn concat<S:ToString>(&mut self, s: S) {
        let msg = ActorScript::Concat {
            // input: (s)
            input: (s.to_string()),
        };
        let _ = self
            .sender
            .send(msg)
            .expect("'MaunActorLive::method.send'. Channel is closed!");
    }
}

// *///.............[ Interthread  End of Write  ].................//



fn main() {

    let act = ActorLive::new();
    
    let mut one = act.clone();
    let mut two = act.clone();
    let mut thr = act.clone();

    let one_h = std::thread::spawn( move || { 
        one.concat("I can handle any".to_string());
    });
    let _ = one_h.join();

    let two_h = std::thread::spawn( move || {
        two.concat(" 'ToString' - ");
    });
    let _ = two_h.join();

    let thr_h = std::thread::spawn( move || {
        thr.concat('😀');
    });
    let _ = thr_h.join();

    
    assert_eq!(
        act.get_value(), 
        "I can handle any 'ToString' - 😀".to_string()
    );
}

```

The same principles apply to the following example, which showcases a generic actor model, to tailor the Live methods to specific generic types, manual adjustments are necessary, as illustrated below.

### Examples


Filename: Cargo.toml

```text
[dependencies]
interthread = "1.1.6"
oneshot     = "0.1.5" 
```

Filename: main.rs
```rust

pub struct MaunActor<T> {
    value: T,
}
#[interthread::actor(channel=2,
    edit( live( imp( add_number))))]
impl<T> MaunActor<T>
where
    T: std::ops::AddAssign + Copy,
{
    pub fn new(v: T) -> Self {
        Self { value: v }
    }
    pub fn add_number(&mut self, num: T) {
        self.value += num;
    }
    pub fn get_value(&self) -> T {
        self.value
    }
}


//++++++++++++++++++[ Interthread  Write to File ]+++++++++++++++++//
// Object Name   : MaunActor  
// Initiated By  : #[interthread::actor(channel=2,file="path/to/this/file.rs",edit(live(imp(add_number))))]  


impl<T> MaunActorLive<T>
where
    T: std::ops::AddAssign + Copy + Send + Sync + 'static,
{
    // pub fn add_number(&mut self, num: T) {
    pub fn add_number<I: Into<T>>(&mut self, num:I) {
        let msg = MaunActorScript::AddNumber {
            // input: (num),
            input: (num.into()),
        };
        let _ = self
            .sender
            .send(msg)
            .expect("'ActorLive::method.send'. Channel is closed!");
    }
}

// *///.............[ Interthread  End of Write  ].................//


fn main() {

    let actor = MaunActorLive::new(0u128);

    let mut actor_a = actor.clone();
    let mut actor_b = actor.clone();

    let handle_a = std::thread::spawn( move || { 
    actor_a.add_number(1_u8);
    });

    let handle_b = std::thread::spawn( move || {
    actor_b.add_number(1_u64);
    });

    let _ = handle_a.join();
    let _ = handle_b.join();

    assert_eq!(actor.get_value(), 2_u128)
}

```


The [`actor`](https://docs.rs/interthread/latest/interthread/attr.actor.html) macro is applied to an impl block, allowing it to be used with both structs and enums to create actor implementations.

### Examples
Filename: Cargo.toml

```text
[dependencies]
interthread = "1.1.6"
oneshot     = "0.1.5" 
```

Filename: main.rs
```rust
#[derive(Debug)]
pub struct Dog(String);

impl Dog {
    fn say(&self) -> String {
        format!("{} says: Woof!", self.0)
    }
}

#[derive(Debug)]
pub struct Cat(String);

impl Cat {
    fn say(&self) -> String {
        format!("{} says: Meow!", self.0)
    }
}

#[derive(Debug)]
pub enum Pet {
    Dog(Dog),
    Cat(Cat),
}


#[interthread::actor(channel=2)]
impl Pet {
    // not in this case, but if 
    // the types used with `Pet` have different
    // parameters for the `new` method, 
    // simply pass a ready `Self` type
    // like this
    pub fn new( pet: Self) -> Self {
        pet
    }

    pub fn speak(&self) -> String {
        match self {
           Self::Dog(dog) => {
            format!("Dog {}",dog.say())
            },
           Self::Cat(cat) => {
            format!("Cat {}", cat.say())
            },
        }
    }
    pub fn swap(&mut self, pet: Self ) -> Self {
        std::mem::replace(self,pet)
    }
}


fn main() {

    let pet = PetLive::new( 
        Pet::Dog(Dog("Tango".to_string()))
    );

    let mut pet_a = pet.clone();
    let pet_b     = pet.clone();
    
    let handle_a = std::thread::spawn( move || {
        println!("Thread A - {}",pet_a.speak());
        // swap the the pet and return it  
        pet_a.swap(Pet::Cat(Cat("Kiki".to_string())))
    });

    let swapped_pet = handle_a.join().unwrap();

    let _handle_b = std::thread::spawn( move || {
        println!("Thread B - {}",pet_b.speak());
    }).join();

    //play with both pets now  
    println!("Thread MAIN - {}",pet.speak());
    println!("Thread MAIN - {}",swapped_pet.speak());

}
```
Outputs
```terminal
Thread A - Dog Tango says: Woof!
Thread B - Cat Kiki says: Meow!
Thread MAIN - Cat Kiki says: Meow!
Thread MAIN - Dog Tango says: Woof!
```

 
For more details, read the
[![Docs.rs](https://docs.rs/interthread/badge.svg)](https://docs.rs/interthread#sdpl-framework)


Join `interthread` on GitHub for more examples and discussions! [![GitHub](https://img.shields.io/badge/GitHub-%2312100E.svg?&style=plastic&logo=GitHub&logoColor=white)](https://github.com/NimonSour/interthread/discussions/1)

Please check regularly for new releases and upgrade to the latest version!

Happy coding! 


