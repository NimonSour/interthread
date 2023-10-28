
//_____________________________________________________________
pub struct AnyOtherType;

pub struct Aa<T>(pub T);

impl<T>  Aa <T>
where T: std::ops::AddAssign,
{
    pub fn add(&mut self, v: T){
        self.0 += v;
    }
}
pub struct Bb<T>(pub T);

impl<T>  Bb <T>
where T: std::ops::AddAssign,
{
    pub fn add(&mut self, v: T){
        self.0 += v;
    }
}
pub struct Cc<T>(pub T);
impl<T> Cc <T>
where T: std::ops::AddAssign,
{
    pub fn add(&mut self, v: T){
        self.0 += v;
    }
}
pub struct AaBbCc<Ta,Tb,Tc> {
    pub a: Aa<Ta>,
    pub b: Bb<Tb>,
    pub c: Cc<Tc>,
    any: AnyOtherType,
}

#[interthread::group( file= "examples/intro_group_generic.rs" )]

impl <Ta,Tb,Tc> AaBbCc  <Ta,Tb,Tc> 
where Ta: std::ops::AddAssign,
      Tb: std::ops::AddAssign,
      Tc: std::ops::AddAssign,
{

    pub fn new( ma:Ta, mb:Tb, mc:Tc ) -> Self {
        let a = Aa(ma);
        let b = Bb(mb);
        let c = Cc(mc);
        let any = AnyOtherType;

        Self{ a,b,c,any }
    }
    pub fn add_to_a(&mut self, v:Ta){
        self.a.0 += v;
    }
    pub fn get_value_of_a(&mut self)-> Ta {
        self.a.0
    }
}



// #[interthread::example(main(path = "examples/intro_group_generic.rs"))]

pub fn main(){

    let group = AaBbCcGroupLive::new();



}