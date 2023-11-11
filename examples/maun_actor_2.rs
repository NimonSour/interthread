
pub struct MaunActor<A, B, C> {
    value_a: Option<A>,
    value_b: Option<B>,
    value_c: Option<C>,
}


#[interthread::actor(channel=2)]
impl<A, B, C>  MaunActor <A, B, C>
where
    A: IntoIterator,
    B: ToString,
    C: ToString,
    A::Item: ToString,
    
{

    pub fn new() -> Self {
        MaunActor { 
            value_a: None, 
            value_b: None,
            value_c: None,
        }
    }

    pub fn set_a(&mut self, value: A) {
        self.value_a = Some(value);
    }

    pub fn set_b(&mut self, value: B) {
        self.value_b = Some(value);
    }

    pub fn set_c(&mut self, value: C) {
        self.value_c = Some(value);
    }

    pub fn sentence(&mut self) -> String {

        let mut s = String::new();

        let vec = self.value_a.take().unwrap();

            let a_strs: Vec<String> = 
            vec.into_iter()
               .map(|item| item.to_string()).collect();
            s += &a_strs.join("");

        if let Some(v) = self.value_b.as_ref(){
            s += &v.to_string();
        }
        if let Some(v) = self.value_c.as_ref(){
            s += &v.to_string();
        }
        s
    }
}

//  #[interthread::example(main(path="examples/maun_actor_2.rs"))] 
fn main() {

    let mut act = MaunActorLive::<Vec<&'static str>,&'static str,char>::new();
    
    let mut one = act.clone();
    let mut two = act.clone();
    let mut thr = act.clone();

    let one_h = std::thread::spawn( move || { 
        one.set_a(vec!["I'm"," a ","generic"]);
    });
    let _ = one_h.join();

    let two_h = std::thread::spawn( move || {
        two.set_b(" actor - ");
    });
    let _ = two_h.join();

    let thr_h = std::thread::spawn( move || {
        thr.set_c('😀');
    });
    let _ = thr_h.join();


    assert_eq!(
        act.sentence(), 
        "I'm a generic actor - 😀".to_string()
    );
    // println!("{}",act.sentence());
}