# interthread

The [`actor`](https://docs.rs/interthread/latest/interthread/attr.actor.html)  macro provided by this crate automates the implementation of an Actor Model for a given struct or enum. It handles the intricacies of message routing and synchronization, empowering developers to swiftly prototype the core functionality of their applications.
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
interthread = "3.1.0"
oneshot     = "0.1.11" 
```

Filename: main.rs
```rust

pub struct MyActor {
    value: i8,
}

#[interthread::actor(show)] // <-  this is it 
impl MyActor {
    
    /// This is my comment
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

// try hovering over model parts to see
// generated code as a comment
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
 Additionally, the `edit` option, when combined with the `file` option, facilitates writing the requested part of the generated code to the file. To substitute the macro with code on file, utilize `edit(file)` within the macro. 

The same example can be run in 
- [tokio](https://crates.io/crates/tokio)
- [async-std](https://crates.io/crates/async-std) 
- [smol](https://crates.io/crates/smol) 

with the only difference being that the methods will 
be marked as `async` and need to be `await`ed for 
asynchronous execution.






