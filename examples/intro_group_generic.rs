
pub struct AnyOtherType;

pub struct Aa<T,V,M>(pub T,pub V,pub M);

impl<T,V,M>  Aa <T,V,M>
where T: std::ops::AddAssign + Clone,
      V: IntoIterator + Clone,
      M: std::ops::AddAssign + Clone,
{
    pub fn add(&mut self, v: T){
        self.0 += v;
    }
    pub fn push(&mut self, v: V)
    where V::Item: Into<M> + std::ops::AddAssign,
    {
        self.1 = v.clone();
        for i in v{
            self.2 += i.into()
        }
    }

    pub fn set < W: Into<T>, Z: Into<M>>(&mut self,input_first: W, input_last: Z) {
        self.0 = input_first.into();
        self.2 = input_last.into();
    }

    pub fn get_values(&self) -> (T,V,M){
        (self.0.clone(),self.1.clone(),self.2.clone())
    }
}
pub struct Bb<T,V,M>(pub T,pub V,pub M);

impl<T,V,M>  Bb <T,V,M>
where T: std::ops::AddAssign + Clone,
      V: IntoIterator + Clone,
      M: std::ops::AddAssign + Clone,
{
    pub fn add(&mut self, v: T){
        self.0 += v;
    }
    pub fn push(&mut self, v: V)
    where V::Item: Into<M> + std::ops::AddAssign,
    {
        self.1 = v.clone();
        for i in v{
            self.2 += i.into()
        }
    }

    pub fn set < W: Into<T>, Z: Into<M>>(&mut self,input_first: W, input_last: Z) {
        self.0 = input_first.into();
        self.2 = input_last.into();
    }

    pub fn get_values(&self) -> (T,V,M){
        (self.0.clone(),self.1.clone(),self.2.clone())
    }
}
pub struct Cc<T,V,M>(pub T,pub V,pub M);

impl<T,V,M>  Cc <T,V,M>
where T: std::ops::AddAssign + Clone,
      V: IntoIterator        + Clone,
      M: std::ops::AddAssign + Clone,
{
    pub fn add(&mut self, v: T){
        self.0 += v;
    }
    pub fn push(&mut self, v: V)
    where V::Item: Into<M> + std::ops::AddAssign,
    {
        self.1 = v.clone();
        for i in v{
            self.2 += i.into()
        }
    }

    pub fn set < W: Into<T>, Z: Into<M>>(&mut self,input_first: W, input_last: Z) {
        self.0 = input_first.into();
        self.2 = input_last.into();
    }

    pub fn get_values(&self) -> (T,V,M){
        (self.0.clone(),self.1.clone(),self.2.clone())
    }
}
pub struct AaBbCc<Ta,Va,Ma,Tb,Vb,Mb,Tc,Vc,Mc> {
    pub a: Aa<Ta,Va,Ma>,
    pub b: Bb<Tb,Vb,Mb>,
    pub c: Cc<Tc,Vc,Mc>,
    any: AnyOtherType,
}

#[interthread::group(file="examples/intro_group_generic.rs",debut)]  

impl <Ta,Va,Ma,Tb,Vb,Mb,Tc,Vc,Mc> AaBbCc  <Ta,Va,Ma,Tb,Vb,Mb,Tc,Vc,Mc> 
where 
    Tc: std::ops::AddAssign + Clone,
    Vc: IntoIterator        + Clone,
    Mc: std::ops::AddAssign + Clone,

    Tb: std::ops::AddAssign + Clone,
    Vb: IntoIterator        + Clone,
    Mb: std::ops::AddAssign + Clone,

    Ta: std::ops::AddAssign + Clone,
    Va: IntoIterator        + Clone,
    Ma: std::ops::AddAssign + Clone,
{

    pub fn new( a:  Aa<Ta,Va,Ma>, b: Bb<Tb,Vb,Mb>, c: Cc<Tc,Vc,Mc> ) -> Self {
        let a = a;
        let b = b;
        let c = c;
        let any = AnyOtherType;

        Self{ a,b,c,any }
    }

    pub fn add_to_a(&mut self, v:Ta){
        self.a.0 += v;
    }
    pub fn get_values_of_a(&mut self)-> (Ta,Va,Ma) {
        self.a.get_values()
    }
    pub fn get_values_of_b(&mut self)-> (Tb,Vb,Mb) {
        self.b.get_values()
    }
    pub fn get_values_of_c(&mut self)-> (Tc,Vc,Mc) {
        self.c.get_values()
    }
}


// #[interthread::example(main(path = "examples/intro_group_generic.rs"))]

pub fn main(){

    let aa  = Aa(8u8, vec![1u16,1,1],1u32);
    let bb = Bb(8u16,vec![1u16,1,1],1u32);
    let cc = Cc(8u32,vec![1u16,1,1],1u32);
    let mut group = AaBbCcGroupLive::new(aa,bb,cc);
    
    group.a.set(1u8,1u8);
    group.b.set(1u8,1u8);
    group.c.set(1u8,1u8);
    group.add_to_a(5);

    println!("Value of a - {:?}",group.get_values_of_a());
    println!("Value of b - {:?}",group.get_values_of_b());
    println!("Value of c - {:?}",group.get_values_of_c());
}