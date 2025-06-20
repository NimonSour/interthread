
#[test]
fn std_family_mutex(){
    mod actor_scope{

        pub struct Actor {
            pub value: u16
        }

        #[interthread::actor(name="Actor1",debut)]
        #[interthread::actor(name="Actor2",debut,
        include( 

            // self public
            met_slf_io_pub_opt,
            met_slf_o_pub_opt,
            met_slf_io_pub_res_string,
            met_slf_o_pub_res_string,
            met_slf_io_pub_res_str,
            met_slf_o_pub_res_str,

            // stat self public
            met_stat_slf_io_pub_opt,
            met_stat_slf_o_pub_opt,
            met_stat_slf_io_pub_res_string,
            met_stat_slf_o_pub_res_string,
            met_stat_slf_io_pub_res_str,
            met_stat_slf_o_pub_res_str,
        ))]

        impl Actor {
        
            pub fn new() -> Self {
               Self { value: 0 } 
            }
        
            //----------------------------------------------------------------
            // reference
            pub fn met_ref_io(&mut self, v: u16) -> u16 {
                self.value += v;
                self.value
            }
        
            pub fn met_ref_i(&mut self, v: u16) {
                self.value += v;
            }
        
            pub fn met_ref_o(&self) -> u16 {
                self.value
            }
        
            pub fn met_ref_void(&mut self) {
                self.value += 1;
            }
        
            //----------------------------------------------------------------
            // reference local generic
            pub fn met_ref_gen_io<I,S>(&mut self, v: I) -> S 
            where I: Into<u16>,
                  S:  FromIterator<u16>,
            {
                self.value += v.into();
                std::iter::once(self.value).collect()
            }
        
            pub fn met_ref_gen_i<I:Into<u16>>(&mut self, v: I) {
                self.value += v.into();
            }
        
            pub fn met_ref_gen_o<S:From<u16>>(&self) -> S {
                self.value.into()
            }
        
            pub fn met_ref_gen_void<A>(&mut self) {
                self.value += 1;
                let msg = std::any::type_name::<A>();
                println!("invoked by - {msg}");
            }
        
            //----------------------------------------------------------------
            // self private
            pub fn met_slf_io_prv(mut self, v: u16) -> u16 {
                self.value += v;
                self.value
            }
        
            pub fn met_slf_i_prv(mut self, v: u16) {
                self.value += v;
            }
        
            pub fn met_slf_o_prv(self) -> u16 {
                self.value
            }
        
            pub fn met_slf_void_prv(mut  self) {
                self.value += 1;
            }
            
            //----------------------------------------------------------------
            // self public option
            pub fn met_slf_io_pub_opt(mut self, v: u16) -> Option<u16> {
                self.value += v;
                Some(self.value)
            }
        
            pub fn met_slf_o_pub_opt(self) -> Option<u16> {
                Some(self.value)
            }
        
            // self public result string
            pub fn met_slf_io_pub_res_string(mut self, v: u16) -> Result<u16,String> {
                self.value += v;
                Ok(self.value)
            }
        
            pub fn met_slf_o_pub_res_string(self) -> Result<u16,String> {
                Ok(self.value)
            }
        
            // self public result str
            pub fn met_slf_io_pub_res_str(mut self, v: u16) -> Result<u16,&'static str> {
                self.value += v;
                Ok(self.value)
            }
        
            pub fn met_slf_o_pub_res_str(self) -> Result<u16,&'static str> {
                Ok(self.value)
            }

            //----------------------------------------------------------------
            // static reference
            pub fn met_stat_ref_io(actor: &mut Self, v: u16) -> u16 {
                actor.value += v;
                actor.value
            }
        
            pub fn met_stat_ref_i(actor: &mut Self, v: u16) {
                actor.value += v;
            }
        
            pub fn met_stat_ref_o(actor: &mut Self) -> u16 {
                actor.value
            }
        
            pub fn met_stat_ref_void(actor : &mut Self) {
                actor.value += 1;
            }
        
            //----------------------------------------------------------------
            // static self consuming 
            pub fn met_stat_slf_io_prv( mut actor: Self, v: u16) -> u16 {
                actor.value += v;
                actor.value
            }
        
            pub fn met_stat_slf_i_prv(mut actor: Self, v: u16) {
                actor.value += v;
                println!("met_stat_slf_i_prv::actor.value - {}",actor.value);
            }
        
            pub fn met_stat_slf_o_prv(actor: Self) -> u16 {
                actor.value
            }
        
            pub fn met_stat_slf_void_prv(mut actor : Self) {
                actor.value += 1;
                println!("met_stat_slf_void_prv::actor.value - {}",actor.value);
            }

            //----------------------------------------------------------------
            // these methods have no logic 
            // static self public option
            pub fn met_stat_slf_io_pub_opt(mut actor : Self, v: u16) -> Option<u16> {
                actor.value += v;
                Some(actor.value)
            }
        
            pub fn met_stat_slf_o_pub_opt(actor : Self) -> Option<u16> {
                Some(actor.value)
            }
        
            // static self public result string
            pub fn met_stat_slf_io_pub_res_string(mut actor : Self, v: u16) -> Result<u16,String> {
                actor.value += v;
                Ok(actor.value)
            }
        
            pub fn met_stat_slf_o_pub_res_string(actor : Self) -> Result<u16,String> {
                Ok(actor.value)
            }
        
            // static self public result str
            pub fn met_stat_slf_io_pub_res_str(mut actor : Self, v: u16) -> Result<u16,&'static str> {
                actor.value += v;
                Ok(actor.value)
            }
        
            pub fn met_stat_slf_o_pub_res_str(actor : Self) -> Result<u16,&'static str> {
                Ok(actor.value)
            }
        
            //----------------------------------------------------------------
            // static 
            pub fn met_stat_io( v: u16 ) -> u16 { v }
        
            pub fn met_stat_i( _v: u16 ) {}
        
            pub fn met_stat_o() -> u16 { 0 }
        
            pub fn met_stat_void() {}
        
            //----------------------------------------------------------------
        }
        pub fn check_private_methods() {
            let actor_a = Actor1Live::new();
            let actor_b = Actor1Live::new();
            let actor_c = Actor1Live::new();
            let actor_d = Actor1Live::new();
            //-----------------------------------------
            // self private 
            //-----------------------------------------
    
            assert_eq!(actor_a.met_slf_io_prv(1),1);
            actor_b.met_slf_i_prv(1);
            actor_c.met_slf_void_prv();
            
            assert_eq!(actor_d.met_slf_o_prv(),0);

            let actor_a = Actor1Live::new();
            let actor_b = Actor1Live::new();
            let actor_c = Actor1Live::new();
            let actor_d = Actor1Live::new();
            //-----------------------------------------
            // static self private 
            //-----------------------------------------
    
            assert_eq!(actor_a.met_stat_slf_io_prv(1),1);
            actor_b.met_stat_slf_i_prv(1);
            actor_c.met_stat_slf_void_prv();
            
            assert_eq!(actor_d.met_stat_slf_o_prv(),0);
        }
    }
    
    use actor_scope::{Actor1Live,Actor2Live,check_private_methods};
    // private self consume methods
    check_private_methods();

    let mut actor = Actor1Live::new();
    // even if is `debut` Actor1Live is not Clone
    // act.clone();

    //-----------------------------------------
    // reference local generic
    //-----------------------------------------

    assert_eq!( actor.met_ref_gen_io::<u8,Vec<_>>(1u8),vec![1]);
    actor.met_ref_gen_i(1u8);
    actor.met_ref_gen_void::<String>();
    assert_eq!(actor.met_ref_gen_o::<u32>(),3u32);


    //-----------------------------------------
    // self public (option)
    //-----------------------------------------
    let act = Actor2Live::new();
    let act1 = Actor2Live::new();
    let act_clone = act.clone();
    let act1_clone = act1.clone();

    assert_eq!(act_clone.met_slf_io_pub_opt(1),None);
    assert_eq!(act.met_slf_io_pub_opt(1),Some(1));

    assert_eq!(act1_clone.met_slf_o_pub_opt(),None);
    assert_eq!(act1.met_slf_o_pub_opt(),Some(0));

    //------------------------------------------
    // self public (result string)
    //------------------------------------------

    let act = Actor2Live::new();
    let act1 = Actor2Live::new();
    let act_clone = act.clone();
    let act1_clone = act1.clone();

    assert_eq!(act_clone.met_slf_io_pub_res_string(1),Result::Err("multiple `Live` instances own the actor".to_string()));
    assert_eq!(act.met_slf_io_pub_res_string(1),Ok(1));

    assert_eq!(act1_clone.met_slf_o_pub_res_string(),Result::Err("multiple `Live` instances own the actor".to_string()));
    assert_eq!(act1.met_slf_o_pub_res_string(),Ok(0));
        
    //------------------------------------------
    // self public (result str)
    //------------------------------------------

    let act = Actor2Live::new();
    let act1 = Actor2Live::new();
    let act_clone = act.clone();
    let act1_clone = act1.clone();

    assert_eq!(act_clone.met_slf_io_pub_res_str(1),Result::Err("multiple `Live` instances own the actor"));
    assert_eq!(act.met_slf_io_pub_res_str(1),Ok(1));

    assert_eq!(act1_clone.met_slf_o_pub_res_str(),Result::Err("multiple `Live` instances own the actor"));
    assert_eq!(act1.met_slf_o_pub_res_str(),Ok(0));


    //-----------------------------------------( special case of static methods )
    //-----------------------------------------
    // static reference
    //-----------------------------------------

    assert_eq!(4,actor.met_stat_ref_io(1));
    actor.met_stat_ref_i(1);
    actor.met_stat_ref_void();
    assert_eq!(actor.met_stat_ref_o(),6);

    //-----------------------------------------
    // static self public (option)
    //-----------------------------------------

    let act = Actor2Live::new();
    let act1 = Actor2Live::new();
    let act_clone = act.clone();
    let act1_clone = act1.clone();

    assert_eq!(act_clone.met_stat_slf_io_pub_opt(1),None);
    assert_eq!(act.met_stat_slf_io_pub_opt(1),Some(1));

    assert_eq!(act1_clone.met_stat_slf_o_pub_opt(),None);
    assert_eq!(act1.met_stat_slf_o_pub_opt(),Some(0));

    //-----------------------------------------
    // static self public (result string)
    //-----------------------------------------

    let act = Actor2Live::new();
    let act1 = Actor2Live::new();
    let act_clone = act.clone();
    let act1_clone = act1.clone();

    assert_eq!(act_clone.met_stat_slf_io_pub_res_string(1),Result::Err("multiple `Live` instances own the actor".to_string()));
    assert_eq!(act.met_stat_slf_io_pub_res_string(1),Ok(1));

    assert_eq!(act1_clone.met_stat_slf_o_pub_res_string(),Result::Err("multiple `Live` instances own the actor".to_string()));
    assert_eq!(act1.met_stat_slf_o_pub_res_string(),Ok(0));

    //-----------------------------------------
    // static self public (result str)
    //-----------------------------------------

    let act = Actor2Live::new();
    let act1 = Actor2Live::new();
    let act_clone = act.clone();
    let act1_clone = act1.clone();    

    assert_eq!(act_clone.met_stat_slf_io_pub_res_str(1),Result::Err("multiple `Live` instances own the actor"));
    assert_eq!(act.met_stat_slf_io_pub_res_str(1),Ok(1));

    assert_eq!(act1_clone.met_stat_slf_o_pub_res_str(),Result::Err("multiple `Live` instances own the actor"));
    assert_eq!(act1.met_stat_slf_o_pub_res_str(),Ok(0));
    
    //-----------------------------------------
    // static 
    //-----------------------------------------

    assert_eq!(Actor1Live::met_stat_io(1),1);
    assert_eq!(Actor1Live::met_stat_i(1),());
    assert_eq!(Actor1Live::met_stat_o(),0);
    assert_eq!(Actor1Live::met_stat_void(),());

}
