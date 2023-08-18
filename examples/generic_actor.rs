
pub struct MaunActor<A, B, C>
where
    A: Clone + ToString,
    B: Clone + ToString,
    C: Clone + ToString,
{
    value_a: Option<A>,
    value_b: Option<B>,
    value_c: Option<C>,
}
#[interthread::actor(channel=2)]
impl<A, B, C> MaunActor <A, B, C>
where
    A: Clone + ToString,
    B: Clone + ToString,
    C: Clone + ToString,
{

    pub fn new() -> Self {
        MaunActor { 
            value_a: None, 
            value_b: None,
            value_c: None,
        }
    }

    pub fn get_a(&self) -> Option<A> {
        self.value_a.clone()
    }

    pub fn get_b(&self) -> Option<B> {
        self.value_b.clone()
    }  

    pub fn get_c(&self) -> Option<C> {
        self.value_c.clone()
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

    pub fn sentence(&self) -> String {

        let mut s = String::new();
        if let Some(v) = self.value_a.as_ref(){
            s += &v.to_string();
        }
        if let Some(v) = self.value_b.as_ref(){
            s += &v.to_string();
        }
        if let Some(v) = self.value_c.as_ref(){
            s += &v.to_string();
        }
        s
    }
}
fn main() {
    let act = MaunActorLive::<String,&'static str,char>::new();
    
    let mut one = act.clone();
    let mut two = act.clone();
    let mut tre = act.clone();

    let one_h = std::thread::spawn( move || { 
        one.set_a("I'm a generic".to_string());
    });

    let two_h = std::thread::spawn( move || {
        two.set_b(" actor - ");
    });

    let tre_h = std::thread::spawn( move || {
        tre.set_c('😀');
    });

    let _ = one_h.join();
    let _ = two_h.join();
    let _ = tre_h.join();


    assert_eq!(
        act.sentence(), 
        "I'm a generic actor - 😀".to_string()
    );
    // println!("{}",act.sentence());
}