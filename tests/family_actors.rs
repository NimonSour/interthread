

#[test]
fn std_family_mutex(){
    mod actor_scope{

        use std::sync::{ Arc, Mutex };
        pub struct Actor {
            pub value: u16
        }
        // bound and unbound channels 
        #[interthread::family( show, Mutex, debut,
            actor( first_name = "User" , channel=3),
            actor( first_name = "User1", channel=3),
            actor( first_name = "User2", channel=0,include(met_stat_slf_o_pub_opt)),
            actor( first_name = "User3", channel=0),
            actor( first_name = "User4", channel=3,include(met_stat_slf_o_pub_res_string)),
            actor( first_name = "User5", channel=3),
            actor( first_name = "User6", channel=0,include(met_stat_slf_o_pub_res_str)),
            actor( first_name = "Admin", include(met_ref_o)),
        )] 

        #[interthread::family( show, Mutex, debut, name="Actor2",
            actor( first_name = "User1", include(met_stat_slf_io_prv)),
            actor( first_name = "User2", include(met_stat_slf_i_prv)),
            actor( first_name = "User3", include(met_stat_slf_o_prv)),
            actor( first_name = "User4", include(met_stat_slf_void_prv)),
            actor( first_name = "Admin", include(met_ref_o)),
        )] 
        
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
            pub fn _met_slf_io_prv(mut self, v: u16) -> u16 {
                self.value += v;
                self.value
            }
        
            pub fn _met_slf_i_prv(mut self, v: u16) {
                self.value += v;
            }
        
            pub fn _met_slf_o_prv(self) -> u16 {
                self.value
            }
        
            pub fn _met_slf_void_prv(mut  self) {
                self.value += 1;
            }
            
        
            //----------------------------------------------------------------
            // self public option
            pub fn _met_slf_io_pub_opt(mut self, v: u16) -> Option<u16> {
                self.value += v;
                Some(self.value)
            }
        
            pub fn _met_slf_o_pub_opt(self) -> Option<u16> {
                Some(self.value)
            }
        
            // self public result string
            pub fn _met_slf_io_pub_res_string(mut self, v: u16) -> Result<u16,String> {
                self.value += v;
                Ok(self.value)
            }
        
            pub fn _met_slf_o_pub_res_string(self) -> Result<u16,String> {
                Ok(self.value)
            }
        
            // self public result str
            pub fn _met_slf_io_pub_res_str(mut self, v: u16) -> Result<u16,&'static str> {
                self.value += v;
                Ok(self.value)
            }
        
            pub fn _met_slf_o_pub_res_str(self) -> Result<u16,&'static str> {
                Ok(self.value)
            }
        
            //----------------------------------------------------------------
            // static reference
            pub fn met_stat_ref_io( actor: &Arc<Mutex<Self>>, v: u16) -> u16 {
                let mut actor = actor.lock().unwrap();
                actor.value += v;
                actor.value
            }
        
            pub fn met_stat_ref_i(actor: &Arc<Mutex<Self>>, v: u16) {
                let mut actor = actor.lock().unwrap();
                actor.value += v;
            }
        
            pub fn met_stat_ref_o(actor: &Arc<Mutex<Self>>) -> u16 {
                let actor = actor.lock().unwrap();
                actor.value
            }
        
            pub fn met_stat_ref_void(actor : &Arc<Mutex<Self>>) {
                let mut actor = actor.lock().unwrap();
                actor.value += 1;
            }
        
            //----------------------------------------------------------------
            // static self consuming 
            pub fn met_stat_slf_io_prv( actor: Arc<Mutex<Self>>, v: u16) -> u16 {
                let mut actor = actor.lock().unwrap();
                actor.value += v;
                actor.value
            }
        
            pub fn met_stat_slf_i_prv(actor: Arc<Mutex<Self>>, v: u16) {
                let mut actor = actor.lock().unwrap();
                actor.value += v;
            }
        
            pub fn met_stat_slf_o_prv(actor: Arc<Mutex<Self>>) -> u16 {
                let actor = actor.lock().unwrap();
                actor.value
            }
        
            pub fn met_stat_slf_void_prv(actor : Arc<Mutex<Self>>) {
                let mut actor = actor.lock().unwrap();
                actor.value += 1;
            }
        
            //----------------------------------------------------------------
            // static self public option
            pub fn met_stat_slf_io_pub_opt(actor : Arc<Mutex<Self>>, v: u16) -> Option<u16> {
                let mut actor = actor.lock().unwrap();
                actor.value += v;
                Some(actor.value)
            }
        
            pub fn met_stat_slf_o_pub_opt(actor : Arc<Mutex<Self>>) -> Option<u16> {
                let actor = actor.lock().unwrap();
                Some(actor.value)
            }
        
            // static self public result string
            pub fn met_stat_slf_io_pub_res_string(actor : Arc<Mutex<Self>>, v: u16) -> Result<u16,String> {
                let mut actor = actor.lock().unwrap();
                actor.value += v;
                Ok(actor.value)
            }
        
            pub fn met_stat_slf_o_pub_res_string(actor : Arc<Mutex<Self>>) -> Result<u16,String> {
                let actor = actor.lock().unwrap();
                Ok(actor.value)
            }
        
            // static self public result str
            pub fn met_stat_slf_io_pub_res_str(actor : Arc<Mutex<Self>>, v: u16) -> Result<u16,&'static str> {
                let mut actor = actor.lock().unwrap();
                actor.value += v;
                Ok(actor.value)
            }
        
            pub fn met_stat_slf_o_pub_res_str(actor : Arc<Mutex<Self>>) -> Result<u16,&'static str> {
                let actor = actor.lock().unwrap();
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
            let fam = Actor2Family::new();

            let Actor2Family{ 
                user1, 
                user2, 
                user3, 
                user4, 
                
                admin} = fam;
            
            //-----------------------------------------
            // static self private 
            //-----------------------------------------
    
            assert_eq!(user1.met_stat_slf_io_prv(1),1);
            user2.met_stat_slf_i_prv(1);
            user4.met_stat_slf_void_prv();
            
            assert_eq!(user3.met_stat_slf_o_prv(),3);
            assert_eq!(admin.met_ref_o(),3);
        }

    }

    use actor_scope::{ActorFamily,UserActorLive,check_private_methods};
    
    // check private methods 
    check_private_methods();

    let fam = ActorFamily::new();
    let ActorFamily{ 
        mut user, 
        user1, 
        user2, 
        user3, 
        user4, 
        user5, 
        user6, 
        
        admin} = fam;

    //------------------------------------------
    // reference
    //-----------------------------------------

    let _ = user.met_ref_io(1);
    user.met_ref_i(1);
    user.met_ref_void();

    assert_eq!(user.met_ref_o(), 3);
    assert_eq!(user.met_ref_o(),admin.met_ref_o());

    //-----------------------------------------
    // reference local generic
    //-----------------------------------------

    let _ :Vec<u16> = user.met_ref_gen_io(1u8);
    user.met_ref_gen_i(1u8);
    user.met_ref_gen_void::<String>();

    assert_eq!(user.met_ref_gen_o::<u32>(), 6u32);
    assert_eq!(user.met_ref_gen_o::<u64>(),admin.met_ref_o() as u64);


    // `self` consuming methods are ignored by the family actors  
    // instead "static self private" and "static self public"

    /*
    
    //------------------------------------------
    // self private
    //-----------------------------------------

    let _ = user._met_slf_io_prv(1);
    user._met_slf_i_prv(1);
    user._met_slf_o_prv();
    user._met_slf_void_prv();

    //------------------------------------------
    // self public (option)
    //-----------------------------------------

    let _ = user._met_slf_io_pub_opt(1);
    user._met_slf_o_pub_opt();

    //------------------------------------------
    // self public (result string)
    //------------------------------------------

    let _ = user._met_slf_io_pub_res_string(1);
    user._met_slf_o_pub_res_string();
        
    //------------------------------------------
    // self public (result str)
    //------------------------------------------

    let _ = user._met_slf_io_pub_res_str(1);
    user._met_slf_o_pub_res_str();

    */

    //-----------------------------------------
    // static reference
    //-----------------------------------------

    let _ = user.met_stat_ref_io(1);
    user.met_stat_ref_i(1);
    user.met_stat_ref_void();
    assert_eq!(user.met_stat_ref_o(),9);
    assert_eq!(user.met_stat_ref_o(), admin.met_ref_o());

    //-----------------------------------------
    // static self public (option)
    //-----------------------------------------

    let _ = user1.met_stat_slf_io_pub_opt(1); 
    /* is Clone because all self consuming methods are NOO */
    let user2_clone = user2.clone(); 
    assert_eq!(user2_clone.met_stat_slf_o_pub_opt(),None);

    assert_eq!(user2.met_stat_slf_o_pub_opt(),Some(10));

    //-----------------------------------------
    // static self public (result string)
    //-----------------------------------------

    let _ = user3.met_stat_slf_io_pub_res_string(1);
    /* is Clone because all self consuming methods are NOO */
    let user4_clone = user4.clone();  
    assert_eq!(user4_clone.met_stat_slf_o_pub_res_string(),Result::Err("multiple `Live` instances own the actor".to_string()));
    
    assert_eq!(user4.met_stat_slf_o_pub_res_string(),Result::Ok(11));

    //-----------------------------------------
    // static self public (result str)
    //-----------------------------------------

    let _ = user5.met_stat_slf_io_pub_res_str(1);
    /* is Clone because all self consuming methods are NOO */
    let user6_clone = user6.clone();  
    assert_eq!(user6_clone.met_stat_slf_o_pub_res_str(),Result::Err("multiple `Live` instances own the actor"));
    
    assert_eq!(user6.met_stat_slf_o_pub_res_str(),Result::Ok(12));
    
    
    //-----------------------------------------
    // static 
    //-----------------------------------------

    assert_eq!(UserActorLive::met_stat_io(1),1);
    assert_eq!(UserActorLive::met_stat_i(1),());
    assert_eq!(UserActorLive::met_stat_o(),0);
    assert_eq!(UserActorLive::met_stat_void(),());

    // final checks
    assert_eq!(user.met_stat_ref_o(),12);
    assert_eq!(user.met_stat_ref_o(), admin.met_ref_o());

}

#[test]
fn std_family_rw_lock(){
    mod actor_scope{

        use std::sync::{ Arc, RwLock };
        pub struct Actor {
            pub value: u16
        }
        // bound and unbound channels 
        #[interthread::family( show, debut,
            actor( first_name = "User" , channel=3),
            actor( first_name = "User1", channel=3),
            actor( first_name = "User2", channel=0,include(met_stat_slf_o_pub_opt)),
            actor( first_name = "User3", channel=0),
            actor( first_name = "User4", channel=3,include(met_stat_slf_o_pub_res_string)),
            actor( first_name = "User5", channel=3),
            actor( first_name = "User6", channel=0,include(met_stat_slf_o_pub_res_str)),
            actor( first_name = "Admin", include(met_ref_o)),
        )] 

        #[interthread::family(show, debut, name="Actor2",
        actor( first_name = "User1", include(met_stat_slf_io_prv)),
        actor( first_name = "User2", include(met_stat_slf_i_prv)),
        actor( first_name = "User3", include(met_stat_slf_o_prv)),
        actor( first_name = "User4", include(met_stat_slf_void_prv)),
        actor( first_name = "Admin", include(met_ref_o)),
        )]
        
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
            pub fn _met_slf_io_prv(mut self, v: u16) -> u16 {
                self.value += v;
                self.value
            }
        
            pub fn _met_slf_i_prv(mut self, v: u16) {
                self.value += v;
            }
        
            pub fn _met_slf_o_prv(self) -> u16 {
                self.value
            }
        
            pub fn _met_slf_void_prv(mut  self) {
                self.value += 1;
            }
            
            
            //----------------------------------------------------------------
            // self public option
            pub fn _met_slf_io_pub_opt(mut self, v: u16) -> Option<u16> {
                self.value += v;
                Some(self.value)
            }
        
            pub fn _met_slf_o_pub_opt(self) -> Option<u16> {
                Some(self.value)
            }
        
            // self public result string
            pub fn _met_slf_io_pub_res_string(mut self, v: u16) -> Result<u16,String> {
                self.value += v;
                Ok(self.value)
            }
        
            pub fn _met_slf_o_pub_res_string(self) -> Result<u16,String> {
                Ok(self.value)
            }
        
            // self public result str
            pub fn _met_slf_io_pub_res_str(mut self, v: u16) -> Result<u16,&'static str> {
                self.value += v;
                Ok(self.value)
            }
        
            pub fn _met_slf_o_pub_res_str(self) -> Result<u16,&'static str> {
                Ok(self.value)
            }
            
            
            //----------------------------------------------------------------
            // static reference
            pub fn met_stat_ref_io( actor: &Arc<RwLock<Self>>, v: u16) -> u16 {
                let mut actor = actor.write().unwrap();
                actor.value += v;
                actor.value
            }
        
            pub fn met_stat_ref_i(actor: &Arc<RwLock<Self>>, v: u16) {
                let mut actor = actor.write().unwrap();
                actor.value += v;
            }
        
            pub fn met_stat_ref_o(actor: &Arc<RwLock<Self>>) -> u16 {
                let actor = actor.read().unwrap();
                actor.value
            }
        
            pub fn met_stat_ref_void(actor : &Arc<RwLock<Self>>) {
                let mut actor = actor.write().unwrap();
                actor.value += 1;
            }
        
            //----------------------------------------------------------------
            // static self consuming 
            pub fn met_stat_slf_io_prv( actor: Arc<RwLock<Self>>, v: u16) -> u16 {
                let mut actor = actor.write().unwrap();
                actor.value += v;
                actor.value
            }
        
            pub fn met_stat_slf_i_prv(actor: Arc<RwLock<Self>>, v: u16) {
                let mut actor = actor.write().unwrap();
                actor.value += v;
            }
        
            pub fn met_stat_slf_o_prv(actor: Arc<RwLock<Self>>) -> u16 {
                let actor = actor.write().unwrap();
                actor.value
            }
        
            pub fn met_stat_slf_void_prv(actor : Arc<RwLock<Self>>) {
                let mut actor = actor.write().unwrap();
                actor.value += 1;
            }
        
            //----------------------------------------------------------------
            // static self public option
            pub fn met_stat_slf_io_pub_opt(actor : Arc<RwLock<Self>>, v: u16) -> Option<u16> {
                let mut actor = actor.write().unwrap();
                actor.value += v;
                Some(actor.value)
            }
        
            pub fn met_stat_slf_o_pub_opt(actor : Arc<RwLock<Self>>) -> Option<u16> {
                let actor = actor.read().unwrap();
                Some(actor.value)
            }
        
            // static self public result string
            pub fn met_stat_slf_io_pub_res_string(actor : Arc<RwLock<Self>>, v: u16) -> Result<u16,String> {
                let mut actor = actor.write().unwrap();
                actor.value += v;
                Ok(actor.value)
            }
        
            pub fn met_stat_slf_o_pub_res_string(actor : Arc<RwLock<Self>>) -> Result<u16,String> {
                let actor = actor.read().unwrap();
                Ok(actor.value)
            }
        
            // static self public result str
            pub fn met_stat_slf_io_pub_res_str(actor : Arc<RwLock<Self>>, v: u16) -> Result<u16,&'static str> {
                let mut actor = actor.write().unwrap();
                actor.value += v;
                Ok(actor.value)
            }
        
            pub fn met_stat_slf_o_pub_res_str(actor : Arc<RwLock<Self>>) -> Result<u16,&'static str> {
                let actor = actor.read().unwrap();
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
            let fam = Actor2Family::new();

            let Actor2Family{ 
                user1, 
                user2, 
                user3, 
                user4, 
                
                admin} = fam;
            
            //-----------------------------------------
            // static self private 
            //-----------------------------------------
    
            assert_eq!(user1.met_stat_slf_io_prv(1),1);
            user2.met_stat_slf_i_prv(1);
            user4.met_stat_slf_void_prv();
            
            assert_eq!(user3.met_stat_slf_o_prv(),3);
            assert_eq!(admin.met_ref_o(),3);
        }


    }

    use actor_scope::{ActorFamily,UserActorLive,check_private_methods};
    check_private_methods();

    let fam = ActorFamily::new();

    let ActorFamily{ 
        mut user, 
        user1, 
        user2, 
        user3, 
        user4, 
        user5, 
        user6, 
        
        admin} = fam;

    //------------------------------------------
    // reference
    //-----------------------------------------

    let _ = user.met_ref_io(1);
    user.met_ref_i(1);
    user.met_ref_void();

    assert_eq!(user.met_ref_o(), 3);
    assert_eq!(user.met_ref_o(),admin.met_ref_o());

    //-----------------------------------------
    // reference local generic
    //-----------------------------------------

    let _ :Vec<u16> = user.met_ref_gen_io(1u8);
    user.met_ref_gen_i(1u8);
    user.met_ref_gen_void::<String>();

    assert_eq!(user.met_ref_gen_o::<u32>(), 6u32);
    assert_eq!(user.met_ref_gen_o::<u64>(),admin.met_ref_o() as u64);


    // `self` consuming methods are ignored by the family actors  
    // instead "static self private" and "static self public"

    /*

    //------------------------------------------
    // self private
    //-----------------------------------------

    let _ = user._met_slf_io_prv(1);
    user._met_slf_i_prv(1);
    user._met_slf_o_prv();
    user._met_slf_void_prv();

    //------------------------------------------
    // self public (option)
    //-----------------------------------------

    let _ = user._met_slf_io_pub_opt(1);
    user._met_slf_o_pub_opt();

    //------------------------------------------
    // self public (result string)
    //------------------------------------------

    let _ = user._met_slf_io_pub_res_string(1);
    user._met_slf_o_pub_res_string();
        
    //------------------------------------------
    // self public (result str)
    //------------------------------------------

    let _ = user._met_slf_io_pub_res_str(1);
    user._met_slf_o_pub_res_str();

    */
    
    //-----------------------------------------
    // static reference
    //-----------------------------------------

    let _ = user.met_stat_ref_io(1);
    user.met_stat_ref_i(1);
    user.met_stat_ref_void();
    assert_eq!(user.met_stat_ref_o(),9);
    assert_eq!(user.met_stat_ref_o(), admin.met_ref_o());


    //-----------------------------------------
    // static self public (option)
    //-----------------------------------------

    let _ = user1.met_stat_slf_io_pub_opt(1);
    /* is Clone because all self consuming methods are NOO */
    let user2_clone = user2.clone();  
    assert_eq!(user2_clone.met_stat_slf_o_pub_opt(),None);

    assert_eq!(user2.met_stat_slf_o_pub_opt(),Some(10));

    //-----------------------------------------
    // static self public (result string)
    //-----------------------------------------

    let _ = user3.met_stat_slf_io_pub_res_string(1);
    /* is Clone because all self consuming methods are NOO */
    let user4_clone = user4.clone();  
    assert_eq!(user4_clone.met_stat_slf_o_pub_res_string(),Result::Err("multiple `Live` instances own the actor".to_string()));
    
    assert_eq!(user4.met_stat_slf_o_pub_res_string(),Result::Ok(11));

    //-----------------------------------------
    // static self public (result str)
    //-----------------------------------------

    let _ = user5.met_stat_slf_io_pub_res_str(1);
    /* is Clone because all self consuming methods are NOO */
    let user6_clone = user6.clone();  
    assert_eq!(user6_clone.met_stat_slf_o_pub_res_str(),Result::Err("multiple `Live` instances own the actor"));
    
    assert_eq!(user6.met_stat_slf_o_pub_res_str(),Result::Ok(12));
    

    //-----------------------------------------
    // static 
    //-----------------------------------------

    assert_eq!(UserActorLive::met_stat_io(1),1);
    assert_eq!(UserActorLive::met_stat_i(1),());
    assert_eq!(UserActorLive::met_stat_o(),0);
    assert_eq!(UserActorLive::met_stat_void(),());

    // final checks
    assert_eq!(user.met_stat_ref_o(),12);
    assert_eq!(user.met_stat_ref_o(), admin.met_ref_o());

}


#[test]
fn std_generic_family_rw_lock(){

    mod actor_scope {

        use std::sync::{ Arc, RwLock };

        pub struct Actor<X,Y, const N:usize> {
            pub value: X,
            _private_generic_field:Y,
        }
        
        // bound and unbound channels 
        #[interthread::family( show, debut,
            actor( first_name = "User" , channel=3),
            actor( first_name = "User1", channel=3),
            actor( first_name = "User2", channel=0,include(met_stat_slf_o_pub_opt)),
            actor( first_name = "User3", channel=0),
            actor( first_name = "User4", channel=3,include(met_stat_slf_o_pub_res_string)),
            actor( first_name = "User5", channel=3),
            actor( first_name = "User6", channel=0,include(met_stat_slf_o_pub_res_str)),
            actor( first_name = "Admin", include(met_ref_o)),
        )] 
        
        #[interthread::family(show, debut, name="Actor2",
        actor( first_name = "User1", include(met_stat_slf_io_prv)),
        actor( first_name = "User2", include(met_stat_slf_i_prv)),
        actor( first_name = "User3", include(met_stat_slf_o_prv)),
        actor( first_name = "User4", include(met_stat_slf_void_prv)),
        actor( first_name = "Admin", include(met_ref_o)),
        )]
        
        impl<X,Y,const N:usize> Actor<X,Y,N> 
        where X: std::ops::AddAssign+ std::ops::Neg<Output = X> + Default + Copy
        {
        
            pub fn new<A,B>(value:X, _private_generic_field: Y,_a:A,_b:B) -> Self {
               Self { value, _private_generic_field } 
            }
        
            //----------------------------------------------------------------
            // reference
            pub fn met_ref_io(&mut self, v: X) -> X {
                self.value += v;
                self.value
            }
        
            pub fn met_ref_i(&mut self, v: X) {
                self.value += v;
            }
        
            pub fn met_ref_o(&self) -> X {
                self.value
            }
        
            pub fn met_ref_void(&mut self) {
                self.value = -self.value;
            }
        
            //----------------------------------------------------------------
            // reference local generic
            pub fn met_ref_gen_io<I,S>(&mut self, v: I) -> S 
            where I: Into<X>,
                  S:  FromIterator<X>,
            {
                self.value += v.into();
                std::iter::once(self.value).collect()
            }
        
            pub fn met_ref_gen_i<I:Into<X>>(&mut self, v: I) {
                self.value += v.into();
            }
        
            pub fn met_ref_gen_o<S:From<X>>(&self) -> S {
                self.value.into()
            }
        
            pub fn met_ref_gen_void<A>(&mut self) {
                self.value = -self.value;
                let msg = std::any::type_name::<A>();
                println!("invoked by - {msg}");
            }
        
            //----------------------------------------------------------------
            // self private
            pub fn _met_slf_io_prv(mut self, v: X) -> X {
                self.value += v;
                self.value
            }
        
            pub fn _met_slf_i_prv(mut self, v: X) {
                self.value += v;
            }
        
            pub fn _met_slf_o_prv(self) -> X {
                self.value
            }
        
            pub fn _met_slf_void_prv(mut  self) {
                self.value = -self.value;
            }
            
            
            //----------------------------------------------------------------
            // self public option
            pub fn _met_slf_io_pub_opt(mut self, v: X) -> Option<X> {
                self.value += v;
                Some(self.value)
            }
        
            pub fn _met_slf_o_pub_opt(self) -> Option<X> {
                Some(self.value)
            }
        
            // self public result string
            pub fn _met_slf_io_pub_res_string(mut self, v: X) -> Result<X,String> {
                self.value += v;
                Ok(self.value)
            }
        
            pub fn _met_slf_o_pub_res_string(self) -> Result<X,String> {
                Ok(self.value)
            }
        
            // self public result str
            pub fn _met_slf_io_pub_res_str(mut self, v: X) -> Result<X,&'static str> {
                self.value += v;
                Ok(self.value)
            }
        
            pub fn _met_slf_o_pub_res_str(self) -> Result<X,&'static str> {
                Ok(self.value)
            }
            
            
            //----------------------------------------------------------------
            // static reference
            pub fn met_stat_ref_io( actor: &Arc<RwLock<Self>>, v: X) -> X {
                let mut actor = actor.write().unwrap();
                actor.value += v;
                actor.value
            }
        
            pub fn met_stat_ref_i(actor: &Arc<RwLock<Self>>, v: X) {
                let mut actor = actor.write().unwrap();
                actor.value += v;
            }
        
            pub fn met_stat_ref_o(actor: &Arc<RwLock<Self>>) -> X {
                let actor = actor.read().unwrap();
                actor.value
            }
        
            pub fn met_stat_ref_void(actor : &Arc<RwLock<Self>>) {
                let mut actor = actor.write().unwrap();
                actor.value = -actor.value;
            }
        
            //----------------------------------------------------------------
            // static self consuming 
            pub fn met_stat_slf_io_prv( actor: Arc<RwLock<Self>>, v: X) -> X {
                let mut actor = actor.write().unwrap();
                actor.value += v;
                actor.value
            }
        
            pub fn met_stat_slf_i_prv(actor: Arc<RwLock<Self>>, v: X) {
                let mut actor = actor.write().unwrap();
                actor.value += v;
            }
        
            pub fn met_stat_slf_o_prv(actor: Arc<RwLock<Self>>) -> X {
                let actor = actor.write().unwrap();
                actor.value
            }
        
            pub fn met_stat_slf_void_prv(actor : Arc<RwLock<Self>>) {
                let mut actor = actor.write().unwrap();
                actor.value = -actor.value;
            }
        
            //----------------------------------------------------------------
            // static self public option
            pub fn met_stat_slf_io_pub_opt(actor : Arc<RwLock<Self>>, v: X) -> Option<X> {
                let mut actor = actor.write().unwrap();
                actor.value += v;
                Some(actor.value)
            }
        
            pub fn met_stat_slf_o_pub_opt(actor : Arc<RwLock<Self>>) -> Option<X> {
                let actor = actor.read().unwrap();
                Some(actor.value)
            }
        
            // static self public result string
            pub fn met_stat_slf_io_pub_res_string(actor : Arc<RwLock<Self>>, v: X) -> Result<X,String> {
                let mut actor = actor.write().unwrap();
                actor.value += v;
                Ok(actor.value)
            }
        
            pub fn met_stat_slf_o_pub_res_string(actor : Arc<RwLock<Self>>) -> Result<X,String> {
                let actor = actor.read().unwrap();
                Ok(actor.value)
            }
        
            // static self public result str
            pub fn met_stat_slf_io_pub_res_str(actor : Arc<RwLock<Self>>, v: X) -> Result<X,&'static str> {
                let mut actor = actor.write().unwrap();
                actor.value += v;
                Ok(actor.value)
            }
        
            pub fn met_stat_slf_o_pub_res_str(actor : Arc<RwLock<Self>>) -> Result<X,&'static str> {
                let actor = actor.read().unwrap();
                Ok(actor.value)
            }
        
            //----------------------------------------------------------------
            // static 
            pub fn met_stat_io( v: X ) -> X { v }
        
            pub fn met_stat_i( _v: X ) {}
        
            pub fn met_stat_o() -> X { X::default() }
        
            pub fn met_stat_void() {}
        
            //----------------------------------------------------------------
        }

        pub fn check_private_methods() {

            let fam = 
            // we need the first `i32` the other types are just to test the compilation 
            Actor2Family::<i32,String,11usize>::new(0i32,"bla".to_string(),[0u8;2], ("bla",2u32));

            let Actor2Family{ 
                user1, 
                user2, 
                user3, 
                user4, 
                
                admin} = fam;
            
            //-----------------------------------------
            // static self private 
            //-----------------------------------------
    
            assert_eq!(user1.met_stat_slf_io_prv(1),1);
            user2.met_stat_slf_i_prv(1);
            user4.met_stat_slf_void_prv();
            
            assert_eq!(user3.met_stat_slf_o_prv(),-2);
            assert_eq!(admin.met_ref_o(),-2);
        }


    }

    use actor_scope::{ActorFamily,UserActorLive,check_private_methods};
    check_private_methods();

    let fam = 
    // we need the first `i32` the other types are just to test the compilation 
    ActorFamily::<i32,String,11usize>::new(0i32,"bla".to_string(),[0u8;2], ("bla",2u32));

    let ActorFamily{ 
        mut user, 
        user1, 
        user2, 
        user3, 
        user4, 
        user5, 
        user6, 
        
        admin} = fam;

    //------------------------------------------
    // reference
    //-----------------------------------------

    let _ = user.met_ref_io(1);
    user.met_ref_i(1);
    user.met_ref_void();

    assert_eq!(user.met_ref_o(), -2);
    assert_eq!(user.met_ref_o(),admin.met_ref_o());

    //-----------------------------------------
    // reference local generic
    //-----------------------------------------

    let _ :Vec<i32> = user.met_ref_gen_io(2u8);
    user.met_ref_gen_i(1u8);
    user.met_ref_gen_void::<String>();

    assert_eq!(user.met_ref_gen_o::<i64>(), -1i64);
    assert_eq!(user.met_ref_gen_o::<i64>(),admin.met_ref_o() as i64);


    // `self` consuming methods are ignored by the family actors  
    // instead "static self private" and "static self public"

    /*

    //------------------------------------------
    // self private
    //-----------------------------------------

    let _ = user._met_slf_io_prv(1);
    user._met_slf_i_prv(1);
    user._met_slf_o_prv();
    user._met_slf_void_prv();

    //------------------------------------------
    // self public (option)
    //-----------------------------------------

    let _ = user._met_slf_io_pub_opt(1);
    user._met_slf_o_pub_opt();

    //------------------------------------------
    // self public (result string)
    //------------------------------------------

    let _ = user._met_slf_io_pub_res_string(1);
    user._met_slf_o_pub_res_string();
        
    //------------------------------------------
    // self public (result str)
    //------------------------------------------

    let _ = user._met_slf_io_pub_res_str(1);
    user._met_slf_o_pub_res_str();

    */
    
    //-----------------------------------------
    // static reference
    //-----------------------------------------

    let _ = user.met_stat_ref_io(1);
    user.met_stat_ref_i(1);
    user.met_stat_ref_void();
    assert_eq!(user.met_stat_ref_o(),-1);
    assert_eq!(user.met_stat_ref_o(), admin.met_ref_o());


    //-----------------------------------------
    // static self public (option)
    //-----------------------------------------

    let _ = user1.met_stat_slf_io_pub_opt(2);
    /* is Clone because all self consuming methods are NOO */
    let user2_clone = user2.clone();  
    assert_eq!(user2_clone.met_stat_slf_o_pub_opt(),None);

    assert_eq!(user2.met_stat_slf_o_pub_opt(),Some(1i32));

    //-----------------------------------------
    // static self public (result string)
    //-----------------------------------------

    let _ = user3.met_stat_slf_io_pub_res_string(1);
    /* is Clone because all self consuming methods are NOO */
    let user4_clone = user4.clone();  
    assert_eq!(user4_clone.met_stat_slf_o_pub_res_string(),Result::Err("multiple `Live` instances own the actor".to_string()));
    
    assert_eq!(user4.met_stat_slf_o_pub_res_string(),Result::Ok(2i32));

    //-----------------------------------------
    // static self public (result str)
    //-----------------------------------------

    let _ = user5.met_stat_slf_io_pub_res_str(1);
    /* is Clone because all self consuming methods are NOO */
    let user6_clone = user6.clone();  
    assert_eq!(user6_clone.met_stat_slf_o_pub_res_str(),Result::Err("multiple `Live` instances own the actor"));
    
    assert_eq!(user6.met_stat_slf_o_pub_res_str(),Result::Ok(3i32));
    

    //-----------------------------------------
    // static 
    //-----------------------------------------

    assert_eq!(UserActorLive::<i32,String,11usize>::met_stat_io(1),1);
    assert_eq!(UserActorLive::<i32,String,11usize>::met_stat_i(1),());
    assert_eq!(UserActorLive::<i32,String,11usize>::met_stat_o(),0);
    assert_eq!(UserActorLive::<i32,String,11usize>::met_stat_void(),());

    // final checks
    assert_eq!(user.met_stat_ref_o(),3i32);
    assert_eq!(user.met_stat_ref_o(), admin.met_ref_o());

}


#[test]
fn tokio_std_generic_family_rw_lock(){

    mod actor_scope {

        use std::sync::Arc;
        use tokio::sync::RwLock;

        pub struct Actor<X,Y, const N:usize> {
            pub value: X,
            _private_generic_field:Y,
        }
        
        // bound and unbound channels 
        #[interthread::family( show, debut,lib="tokio",
            actor( first_name = "User" , channel=3),
            actor( first_name = "User1", channel=3),
            actor( first_name = "User2", channel=0,include(met_stat_slf_o_pub_opt)),
            actor( first_name = "User3", channel=0),
            actor( first_name = "User4", channel=3,include(met_stat_slf_o_pub_res_string)),
            actor( first_name = "User5", channel=3),
            actor( first_name = "User6", channel=0,include(met_stat_slf_o_pub_res_str)),
            actor( first_name = "Admin", include(met_ref_o)),
        )] 
        
        #[interthread::family(show, debut,lib="tokio", name="Actor2",
        actor( first_name = "User1", include(met_stat_slf_io_prv)),
        actor( first_name = "User2", include(met_stat_slf_i_prv)),
        actor( first_name = "User3", include(met_stat_slf_o_prv)),
        actor( first_name = "User4", include(met_stat_slf_void_prv)),
        actor( first_name = "Admin", include(met_ref_o)),
        )]
        
        impl<X,Y,const N:usize> Actor<X,Y,N> 
        where X: std::ops::AddAssign+ std::ops::Neg<Output = X> + Default + Copy
        {
        
            pub fn new<A,B>(value:X, _private_generic_field: Y,_a:A,_b:B) -> Self {
               Self { value, _private_generic_field } 
            }
        
            //----------------------------------------------------------------
            // reference
            pub fn met_ref_io(&mut self, v: X) -> X {
                self.value += v;
                self.value
            }
        
            pub fn met_ref_i(&mut self, v: X) {
                self.value += v;
            }
        
            pub fn met_ref_o(&self) -> X {
                self.value
            }
        
            pub fn met_ref_void(&mut self) {
                self.value = -self.value;
            }
        
            //----------------------------------------------------------------
            // reference local generic
            pub fn met_ref_gen_io<I,S>(&mut self, v: I) -> S 
            where I: Into<X>,
                  S:  FromIterator<X>,
            {
                self.value += v.into();
                std::iter::once(self.value).collect()
            }
        
            pub fn met_ref_gen_i<I:Into<X>>(&mut self, v: I) {
                self.value += v.into();
            }
        
            pub fn met_ref_gen_o<S:From<X>>(&self) -> S {
                self.value.into()
            }
        
            pub fn met_ref_gen_void<A>(&mut self) {
                self.value = -self.value;
                let msg = std::any::type_name::<A>();
                println!("invoked by - {msg}");
            }
        
            //----------------------------------------------------------------
            // self private
            pub fn _met_slf_io_prv(mut self, v: X) -> X {
                self.value += v;
                self.value
            }
        
            pub fn _met_slf_i_prv(mut self, v: X) {
                self.value += v;
            }
        
            pub fn _met_slf_o_prv(self) -> X {
                self.value
            }
        
            pub fn _met_slf_void_prv(mut  self) {
                self.value = -self.value;
            }
            
            
            //----------------------------------------------------------------
            // self public option
            pub fn _met_slf_io_pub_opt(mut self, v: X) -> Option<X> {
                self.value += v;
                Some(self.value)
            }
        
            pub fn _met_slf_o_pub_opt(self) -> Option<X> {
                Some(self.value)
            }
        
            // self public result string
            pub fn _met_slf_io_pub_res_string(mut self, v: X) -> Result<X,String> {
                self.value += v;
                Ok(self.value)
            }
        
            pub fn _met_slf_o_pub_res_string(self) -> Result<X,String> {
                Ok(self.value)
            }
        
            // self public result str
            pub fn _met_slf_io_pub_res_str(mut self, v: X) -> Result<X,&'static str> {
                self.value += v;
                Ok(self.value)
            }
        
            pub fn _met_slf_o_pub_res_str(self) -> Result<X,&'static str> {
                Ok(self.value)
            }
            
            
            //----------------------------------------------------------------
            // static reference
            pub async fn met_stat_ref_io( actor: &Arc<RwLock<Self>>, v: X) -> X {
                let mut actor = actor.write().await;
                actor.value += v;
                actor.value
            }
        
            pub async fn met_stat_ref_i(actor: &Arc<RwLock<Self>>, v: X) {
                let mut actor = actor.write().await;
                actor.value += v;
            }
        
            pub async fn met_stat_ref_o(actor: &Arc<RwLock<Self>>) -> X {
                let actor = actor.read().await;
                actor.value
            }
        
            pub async fn met_stat_ref_void(actor : &Arc<RwLock<Self>>) {
                let mut actor = actor.write().await;
                actor.value = -actor.value;
            }
        
            //----------------------------------------------------------------
            // static self consuming 
            pub async fn met_stat_slf_io_prv( actor: Arc<RwLock<Self>>, v: X) -> X {
                let mut actor = actor.write().await;
                actor.value += v;
                actor.value
            }
        
            pub async fn met_stat_slf_i_prv(actor: Arc<RwLock<Self>>, v: X) {
                let mut actor = actor.write().await;
                actor.value += v;
            }
        
            pub async fn met_stat_slf_o_prv(actor: Arc<RwLock<Self>>) -> X {
                let actor = actor.write().await;
                actor.value
            }
        
            pub async fn met_stat_slf_void_prv(actor : Arc<RwLock<Self>>) {
                let mut actor = actor.write().await;
                actor.value = -actor.value;
            }
        
            //----------------------------------------------------------------
            // static self public option
            pub async fn met_stat_slf_io_pub_opt(actor : Arc<RwLock<Self>>, v: X) -> Option<X> {
                let mut actor = actor.write().await;
                actor.value += v;
                Some(actor.value)
            }
        
            pub async fn met_stat_slf_o_pub_opt(actor : Arc<RwLock<Self>>) -> Option<X> {
                let actor = actor.read().await;
                Some(actor.value)
            }
        
            // static self public result string
            pub async fn met_stat_slf_io_pub_res_string(actor : Arc<RwLock<Self>>, v: X) -> Result<X,String> {
                let mut actor = actor.write().await;
                actor.value += v;
                Ok(actor.value)
            }
        
            pub async fn met_stat_slf_o_pub_res_string(actor : Arc<RwLock<Self>>) -> Result<X,String> {
                let actor = actor.read().await;
                Ok(actor.value)
            }
        
            // static self public result str
            pub async fn met_stat_slf_io_pub_res_str(actor : Arc<RwLock<Self>>, v: X) -> Result<X,&'static str> {
                let mut actor = actor.write().await;
                actor.value += v;
                Ok(actor.value)
            }
        
            pub async fn met_stat_slf_o_pub_res_str(actor : Arc<RwLock<Self>>) -> Result<X,&'static str> {
                let actor = actor.read().await;
                Ok(actor.value)
            }
        
            //----------------------------------------------------------------
            // static 
            pub fn met_stat_io( v: X ) -> X { v }
        
            pub fn met_stat_i( _v: X ) {}
        
            pub fn met_stat_o() -> X { X::default() }
        
            pub fn met_stat_void() {}
        
            //----------------------------------------------------------------
        }

        pub async fn check_private_methods() {

               
            let fam = 
            // we need the first `i32` the other types are just to test the compilation 
            Actor2Family::<i32,String,11usize>::new(0i32,"bla".to_string(),[0u8;2], ("bla",2u32));

            let Actor2Family{ 
                user1, 
                user2, 
                user3, 
                user4, 
                
                admin} = fam;
            
            //-----------------------------------------
            // static self private 
            //-----------------------------------------
    
            assert_eq!(user1.met_stat_slf_io_prv(1).await,1);
            user2.met_stat_slf_i_prv(1).await;
            user4.met_stat_slf_void_prv().await;
            
            assert_eq!(user3.met_stat_slf_o_prv().await,-2);
            assert_eq!(admin.met_ref_o().await,-2);
        }


    }

    use actor_scope::{ActorFamily,UserActorLive,check_private_methods};

    tokio::runtime::Runtime::new()
    .unwrap()
    .block_on(async {

        check_private_methods().await;

        let fam = 
        // we need the first `i32` the other types are just to test the compilation 
        ActorFamily::<i32,String,11usize>::new(0i32,"bla".to_string(),[0u8;2], ("bla",2u32));

        let ActorFamily{ 
            mut user, 
            user1, 
            user2, 
            user3, 
            user4, 
            user5, 
            user6, 
            
            admin} = fam;

        //------------------------------------------
        // reference
        //-----------------------------------------

        let _ = user.met_ref_io(1).await;
        user.met_ref_i(1).await;
        user.met_ref_void().await;

        assert_eq!(user.met_ref_o().await, -2);
        assert_eq!(user.met_ref_o().await,admin.met_ref_o().await);

        //-----------------------------------------
        // reference local generic
        //-----------------------------------------

        let _ :Vec<i32> = user.met_ref_gen_io(2u8).await;
        user.met_ref_gen_i(1u8).await;
        user.met_ref_gen_void::<String>().await;

        assert_eq!(user.met_ref_gen_o::<i64>().await, -1i64);
        assert_eq!(user.met_ref_gen_o::<i64>().await,admin.met_ref_o().await as i64);


        // `self` consuming methods are ignored by the family actors  
        // instead "static self private" and "static self public"

        /*

        //-----------------------------------------
        // self private
        //-----------------------------------------

        let _ = user._met_slf_io_prv(1);
        user._met_slf_i_prv(1);
        user._met_slf_o_prv();
        user._met_slf_void_prv();

        //-----------------------------------------
        // self public (option)
        //-----------------------------------------

        let _ = user._met_slf_io_pub_opt(1);
        user._met_slf_o_pub_opt();

        //------------------------------------------
        // self public (result string)
        //------------------------------------------

        let _ = user._met_slf_io_pub_res_string(1);
        user._met_slf_o_pub_res_string();
            
        //------------------------------------------
        // self public (result str)
        //------------------------------------------

        let _ = user._met_slf_io_pub_res_str(1);
        user._met_slf_o_pub_res_str();

        */
        
        //-----------------------------------------
        // static reference
        //-----------------------------------------

        let _ = user.met_stat_ref_io(1).await;
        user.met_stat_ref_i(1).await;
        user.met_stat_ref_void().await;
        assert_eq!(user.met_stat_ref_o().await,-1);
        assert_eq!(user.met_stat_ref_o().await, admin.met_ref_o().await);

        
        //-----------------------------------------
        // static self public (option)
        //-----------------------------------------

        let _ = user1.met_stat_slf_io_pub_opt(2).await;
        /* is Clone because all self consuming methods are NOO */
        let user2_clone = user2.clone();  
        assert_eq!(user2_clone.met_stat_slf_o_pub_opt().await,None);

        assert_eq!(user2.met_stat_slf_o_pub_opt().await,Some(1i32));

        
        //-----------------------------------------
        // static self public (result string)
        //-----------------------------------------

        let _ = user3.met_stat_slf_io_pub_res_string(1).await;
        /* is Clone because all self consuming methods are NOO */
        let user4_clone = user4.clone();  
        assert_eq!(user4_clone.met_stat_slf_o_pub_res_string().await,Result::Err("multiple `Live` instances own the actor".to_string()));
        
        assert_eq!(user4.met_stat_slf_o_pub_res_string().await,Result::Ok(2i32));

        //-----------------------------------------
        // static self public (result str)
        //-----------------------------------------

        let _ = user5.met_stat_slf_io_pub_res_str(1).await;
        /* is Clone because all self consuming methods are NOO */
        let user6_clone = user6.clone();  
        assert_eq!(user6_clone.met_stat_slf_o_pub_res_str().await,Result::Err("multiple `Live` instances own the actor"));
        
        assert_eq!(user6.met_stat_slf_o_pub_res_str().await,Result::Ok(3i32));
        

        //-----------------------------------------
        // static 
        //-----------------------------------------

        assert_eq!(UserActorLive::<i32,String,11usize>::met_stat_io(1),1);
        assert_eq!(UserActorLive::<i32,String,11usize>::met_stat_i(1),());
        assert_eq!(UserActorLive::<i32,String,11usize>::met_stat_o(),0);
        assert_eq!(UserActorLive::<i32,String,11usize>::met_stat_void(),());

        // final checks
        assert_eq!(user.met_stat_ref_o().await,3i32);
        assert_eq!(user.met_stat_ref_o().await, admin.met_ref_o().await);

        


    });

}


#[test]
fn tokio_async_generic_family_rw_lock(){

    mod actor_scope {

        use std::sync::Arc;
        use tokio::sync::RwLock;

        pub struct Actor<X,Y, const N:usize> {
            pub value: X,
            _private_generic_field:Y,
        }
        
        // bound and unbound channels 
        #[interthread::family( show, debut,lib="tokio",
            actor( first_name = "User" , channel=3),
            actor( first_name = "User1", channel=3),
            actor( first_name = "User2", channel=0,include(met_stat_slf_o_pub_opt)),
            actor( first_name = "User3", channel=0),
            actor( first_name = "User4", channel=3,include(met_stat_slf_o_pub_res_string)),
            actor( first_name = "User5", channel=3),
            actor( first_name = "User6", channel=0,include(met_stat_slf_o_pub_res_str)),
            actor( first_name = "Admin", include(met_ref_o)),
        )] 
        
        #[interthread::family(show, debut,lib="tokio", name="Actor2",
        actor( first_name = "User1", include(met_stat_slf_io_prv)),
        actor( first_name = "User2", include(met_stat_slf_i_prv)),
        actor( first_name = "User3", include(met_stat_slf_o_prv)),
        actor( first_name = "User4", include(met_stat_slf_void_prv)),
        actor( first_name = "Admin", include(met_ref_o)),
        )]
        
        impl<X,Y,const N:usize> Actor<X,Y,N> 
        where X: std::ops::AddAssign+ std::ops::Neg<Output = X> + Default + Copy
        {
        
            pub fn new<A,B>(value:X, _private_generic_field: Y,_a:A,_b:B) -> Self {
               Self { value, _private_generic_field } 
            }
        
            //----------------------------------------------------------------
            // reference
            pub async fn met_ref_io(&mut self, v: X) -> X {
                self.value += v;
                self.value
            }
        
            pub async fn met_ref_i(&mut self, v: X) {
                self.value += v;
            }
        
            pub async fn met_ref_o(&self) -> X {
                self.value
            }
        
            pub async fn met_ref_void(&mut self) {
                self.value = -self.value;
            }
        
            //----------------------------------------------------------------
            // reference local generic
            pub async fn met_ref_gen_io<I,S>(&mut self, v: I) -> S 
            where I: Into<X>,
                  S:  FromIterator<X>,
            {
                self.value += v.into();
                std::iter::once(self.value).collect()
            }
        
            pub async fn met_ref_gen_i<I:Into<X>>(&mut self, v: I) {
                self.value += v.into();
            }
        
            pub async fn met_ref_gen_o<S:From<X>>(&self) -> S {
                self.value.into()
            }
        
            pub async fn met_ref_gen_void<A>(&mut self) {
                self.value = -self.value;
                let msg = std::any::type_name::<A>();
                println!("invoked by - {msg}");
            }
        
            //----------------------------------------------------------------
            // self private
            pub async fn _met_slf_io_prv(mut self, v: X) -> X {
                self.value += v;
                self.value
            }
        
            pub async fn _met_slf_i_prv(mut self, v: X) {
                self.value += v;
            }
        
            pub async fn _met_slf_o_prv(self) -> X {
                self.value
            }
        
            pub async fn _met_slf_void_prv(mut  self) {
                self.value = -self.value;
            }
            
            
            //----------------------------------------------------------------
            // self public option
            pub async fn _met_slf_io_pub_opt(mut self, v: X) -> Option<X> {
                self.value += v;
                Some(self.value)
            }
        
            pub async fn _met_slf_o_pub_opt(self) -> Option<X> {
                Some(self.value)
            }
        
            // self public result string
            pub async fn _met_slf_io_pub_res_string(mut self, v: X) -> Result<X,String> {
                self.value += v;
                Ok(self.value)
            }
        
            pub async fn _met_slf_o_pub_res_string(self) -> Result<X,String> {
                Ok(self.value)
            }
        
            // self public result str
            pub async fn _met_slf_io_pub_res_str(mut self, v: X) -> Result<X,&'static str> {
                self.value += v;
                Ok(self.value)
            }
        
            pub async fn _met_slf_o_pub_res_str(self) -> Result<X,&'static str> {
                Ok(self.value)
            }
            
            
            //----------------------------------------------------------------
            // static reference
            pub async fn met_stat_ref_io( actor: &Arc<RwLock<Self>>, v: X) -> X {
                let mut actor = actor.write().await;
                actor.value += v;
                actor.value
            }
        
            pub async fn met_stat_ref_i(actor: &Arc<RwLock<Self>>, v: X) {
                let mut actor = actor.write().await;
                actor.value += v;
            }
        
            pub async fn met_stat_ref_o(actor: &Arc<RwLock<Self>>) -> X {
                let actor = actor.read().await;
                actor.value
            }
        
            pub async fn met_stat_ref_void(actor : &Arc<RwLock<Self>>) {
                let mut actor = actor.write().await;
                actor.value = -actor.value;
            }
        
            //----------------------------------------------------------------
            // static self consuming 
            pub async fn met_stat_slf_io_prv( actor: Arc<RwLock<Self>>, v: X) -> X {
                let mut actor = actor.write().await;
                actor.value += v;
                actor.value
            }
        
            pub async fn met_stat_slf_i_prv(actor: Arc<RwLock<Self>>, v: X) {
                let mut actor = actor.write().await;
                actor.value += v;
            }
        
            pub async fn met_stat_slf_o_prv(actor: Arc<RwLock<Self>>) -> X {
                let actor = actor.write().await;
                actor.value
            }
        
            pub async fn met_stat_slf_void_prv(actor : Arc<RwLock<Self>>) {
                let mut actor = actor.write().await;
                actor.value = -actor.value;
            }
        
            //----------------------------------------------------------------
            // static self public option
            pub async fn met_stat_slf_io_pub_opt(actor : Arc<RwLock<Self>>, v: X) -> Option<X> {
                let mut actor = actor.write().await;
                actor.value += v;
                Some(actor.value)
            }
        
            pub async fn met_stat_slf_o_pub_opt(actor : Arc<RwLock<Self>>) -> Option<X> {
                let actor = actor.read().await;
                Some(actor.value)
            }
        
            // static self public result string
            pub async fn met_stat_slf_io_pub_res_string(actor : Arc<RwLock<Self>>, v: X) -> Result<X,String> {
                let mut actor = actor.write().await;
                actor.value += v;
                Ok(actor.value)
            }
        
            pub async fn met_stat_slf_o_pub_res_string(actor : Arc<RwLock<Self>>) -> Result<X,String> {
                let actor = actor.read().await;
                Ok(actor.value)
            }
        
            // static self public result str
            pub async fn met_stat_slf_io_pub_res_str(actor : Arc<RwLock<Self>>, v: X) -> Result<X,&'static str> {
                let mut actor = actor.write().await;
                actor.value += v;
                Ok(actor.value)
            }
        
            pub async fn met_stat_slf_o_pub_res_str(actor : Arc<RwLock<Self>>) -> Result<X,&'static str> {
                let actor = actor.read().await;
                Ok(actor.value)
            }
        
            //----------------------------------------------------------------
            // static 
            pub async fn met_stat_io( v: X ) -> X { v }
        
            pub async fn met_stat_i( _v: X ) {}
        
            pub async fn met_stat_o() -> X { X::default() }
        
            pub async fn met_stat_void() {}
        
            //----------------------------------------------------------------
        }

        pub async fn check_private_methods() {

               
            let fam = 
            // we need the first `i32` the other types are just to test the compilation 
            Actor2Family::<i32,String,11usize>::new(0i32,"bla".to_string(),[0u8;2], ("bla",2u32));

            let Actor2Family{ 
                user1, 
                user2, 
                user3, 
                user4, 
                
                admin} = fam;
            
            //-----------------------------------------
            // static self private 
            //-----------------------------------------
    
            assert_eq!(user1.met_stat_slf_io_prv(1).await,1);
            user2.met_stat_slf_i_prv(1).await;
            user4.met_stat_slf_void_prv().await;
            
            assert_eq!(user3.met_stat_slf_o_prv().await,-2);
            assert_eq!(admin.met_ref_o().await,-2);
        }


    }

    use actor_scope::{ActorFamily,UserActorLive,check_private_methods};

    tokio::runtime::Runtime::new()
    .unwrap()
    .block_on(async {

        check_private_methods().await;

        let fam = 
        // we need the first `i32` the other types are just to test the compilation 
        ActorFamily::<i32,String,11usize>::new(0i32,"bla".to_string(),[0u8;2], ("bla",2u32));

        let ActorFamily{ 
            mut user, 
            user1, 
            user2, 
            user3, 
            user4, 
            user5, 
            user6, 
            
            admin} = fam;

        //------------------------------------------
        // reference
        //-----------------------------------------

        let _ = user.met_ref_io(1).await;
        user.met_ref_i(1).await;
        user.met_ref_void().await;

        assert_eq!(user.met_ref_o().await, -2);
        assert_eq!(user.met_ref_o().await,admin.met_ref_o().await);

        //-----------------------------------------
        // reference local generic
        //-----------------------------------------

        let _ :Vec<i32> = user.met_ref_gen_io(2u8).await;
        user.met_ref_gen_i(1u8).await;
        user.met_ref_gen_void::<String>().await;

        assert_eq!(user.met_ref_gen_o::<i64>().await, -1i64);
        assert_eq!(user.met_ref_gen_o::<i64>().await,admin.met_ref_o().await as i64);


        // `self` consuming methods are ignored by the family actors  
        // instead "static self private" and "static self public"

        /*

        //-----------------------------------------
        // self private
        //-----------------------------------------

        let _ = user._met_slf_io_prv(1);
        user._met_slf_i_prv(1);
        user._met_slf_o_prv();
        user._met_slf_void_prv();

        //-----------------------------------------
        // self public (option)
        //-----------------------------------------

        let _ = user._met_slf_io_pub_opt(1);
        user._met_slf_o_pub_opt();

        //------------------------------------------
        // self public (result string)
        //------------------------------------------

        let _ = user._met_slf_io_pub_res_string(1);
        user._met_slf_o_pub_res_string();
            
        //------------------------------------------
        // self public (result str)
        //------------------------------------------

        let _ = user._met_slf_io_pub_res_str(1);
        user._met_slf_o_pub_res_str();

        */
        
        //-----------------------------------------
        // static reference
        //-----------------------------------------

        let _ = user.met_stat_ref_io(1).await;
        user.met_stat_ref_i(1).await;
        user.met_stat_ref_void().await;
        assert_eq!(user.met_stat_ref_o().await,-1);
        assert_eq!(user.met_stat_ref_o().await, admin.met_ref_o().await);

        
        //-----------------------------------------
        // static self public (option)
        //-----------------------------------------

        let _ = user1.met_stat_slf_io_pub_opt(2).await;
        /* is Clone because all self consuming methods are NOO */
        let user2_clone = user2.clone();  
        assert_eq!(user2_clone.met_stat_slf_o_pub_opt().await,None);

        assert_eq!(user2.met_stat_slf_o_pub_opt().await,Some(1i32));

        
        //-----------------------------------------
        // static self public (result string)
        //-----------------------------------------

        let _ = user3.met_stat_slf_io_pub_res_string(1).await;
        /* is Clone because all self consuming methods are NOO */
        let user4_clone = user4.clone();  
        assert_eq!(user4_clone.met_stat_slf_o_pub_res_string().await,Result::Err("multiple `Live` instances own the actor".to_string()));
        
        assert_eq!(user4.met_stat_slf_o_pub_res_string().await,Result::Ok(2i32));

        //-----------------------------------------
        // static self public (result str)
        //-----------------------------------------

        let _ = user5.met_stat_slf_io_pub_res_str(1).await;
        /* is Clone because all self consuming methods are NOO */
        let user6_clone = user6.clone();  
        assert_eq!(user6_clone.met_stat_slf_o_pub_res_str().await,Result::Err("multiple `Live` instances own the actor"));
        
        assert_eq!(user6.met_stat_slf_o_pub_res_str().await,Result::Ok(3i32));
        

        //-----------------------------------------
        // static 
        //-----------------------------------------

        assert_eq!(UserActorLive::<i32,String,11usize>::met_stat_io(1).await,1);
        assert_eq!(UserActorLive::<i32,String,11usize>::met_stat_i(1).await,());
        assert_eq!(UserActorLive::<i32,String,11usize>::met_stat_o().await,0);
        assert_eq!(UserActorLive::<i32,String,11usize>::met_stat_void().await,());

        // final checks
        assert_eq!(user.met_stat_ref_o().await,3i32);
        assert_eq!(user.met_stat_ref_o().await, admin.met_ref_o().await);

    });

}


#[test]
fn async_std_std_generic_family_rw_lock(){

    mod actor_scope {

        use std::sync::Arc;
        use async_std::sync::RwLock;

        pub struct Actor<X,Y, const N:usize> {
            pub value: X,
            _private_generic_field:Y,
        }
        
        // bound and unbound channels 
        #[interthread::family( show, debut,lib="async_std",
            actor( first_name = "User" , channel=3),
            actor( first_name = "User1", channel=3),
            actor( first_name = "User2", channel=0,include(met_stat_slf_o_pub_opt)),
            actor( first_name = "User3", channel=0),
            actor( first_name = "User4", channel=3,include(met_stat_slf_o_pub_res_string)),
            actor( first_name = "User5", channel=3),
            actor( first_name = "User6", channel=0,include(met_stat_slf_o_pub_res_str)),
            actor( first_name = "Admin", include(met_ref_o)),
        )] 
        
        #[interthread::family(show, debut,lib="async_std", name="Actor2",
        actor( first_name = "User1", include(met_stat_slf_io_prv)),
        actor( first_name = "User2", include(met_stat_slf_i_prv)),
        actor( first_name = "User3", include(met_stat_slf_o_prv)),
        actor( first_name = "User4", include(met_stat_slf_void_prv)),
        actor( first_name = "Admin", include(met_ref_o)),
        )]
        
        impl<X,Y,const N:usize> Actor<X,Y,N> 
        where X: std::ops::AddAssign+ std::ops::Neg<Output = X> + Default + Copy
        {
        
            pub fn new<A,B>(value:X, _private_generic_field: Y,_a:A,_b:B) -> Self {
               Self { value, _private_generic_field } 
            }
        
            //----------------------------------------------------------------
            // reference
            pub fn met_ref_io(&mut self, v: X) -> X {
                self.value += v;
                self.value
            }
        
            pub fn met_ref_i(&mut self, v: X) {
                self.value += v;
            }
        
            pub fn met_ref_o(&self) -> X {
                self.value
            }
        
            pub fn met_ref_void(&mut self) {
                self.value = -self.value;
            }
        
            //----------------------------------------------------------------
            // reference local generic
            pub fn met_ref_gen_io<I,S>(&mut self, v: I) -> S 
            where I: Into<X>,
                  S:  FromIterator<X>,
            {
                self.value += v.into();
                std::iter::once(self.value).collect()
            }
        
            pub fn met_ref_gen_i<I:Into<X>>(&mut self, v: I) {
                self.value += v.into();
            }
        
            pub fn met_ref_gen_o<S:From<X>>(&self) -> S {
                self.value.into()
            }
        
            pub fn met_ref_gen_void<A>(&mut self) {
                self.value = -self.value;
                let msg = std::any::type_name::<A>();
                println!("invoked by - {msg}");
            }
        
            //----------------------------------------------------------------
            // self private
            pub fn _met_slf_io_prv(mut self, v: X) -> X {
                self.value += v;
                self.value
            }
        
            pub fn _met_slf_i_prv(mut self, v: X) {
                self.value += v;
            }
        
            pub fn _met_slf_o_prv(self) -> X {
                self.value
            }
        
            pub fn _met_slf_void_prv(mut  self) {
                self.value = -self.value;
            }
            
            
            //----------------------------------------------------------------
            // self public option
            pub fn _met_slf_io_pub_opt(mut self, v: X) -> Option<X> {
                self.value += v;
                Some(self.value)
            }
        
            pub fn _met_slf_o_pub_opt(self) -> Option<X> {
                Some(self.value)
            }
        
            // self public result string
            pub fn _met_slf_io_pub_res_string(mut self, v: X) -> Result<X,String> {
                self.value += v;
                Ok(self.value)
            }
        
            pub fn _met_slf_o_pub_res_string(self) -> Result<X,String> {
                Ok(self.value)
            }
        
            // self public result str
            pub fn _met_slf_io_pub_res_str(mut self, v: X) -> Result<X,&'static str> {
                self.value += v;
                Ok(self.value)
            }
        
            pub fn _met_slf_o_pub_res_str(self) -> Result<X,&'static str> {
                Ok(self.value)
            }
            
            
            //----------------------------------------------------------------
            // static reference
            pub async fn met_stat_ref_io( actor: &Arc<RwLock<Self>>, v: X) -> X {
                let mut actor = actor.write().await;
                actor.value += v;
                actor.value
            }
        
            pub async fn met_stat_ref_i(actor: &Arc<RwLock<Self>>, v: X) {
                let mut actor = actor.write().await;
                actor.value += v;
            }
        
            pub async fn met_stat_ref_o(actor: &Arc<RwLock<Self>>) -> X {
                let actor = actor.read().await;
                actor.value
            }
        
            pub async fn met_stat_ref_void(actor : &Arc<RwLock<Self>>) {
                let mut actor = actor.write().await;
                actor.value = -actor.value;
            }
        
            //----------------------------------------------------------------
            // static self consuming 
            pub async fn met_stat_slf_io_prv( actor: Arc<RwLock<Self>>, v: X) -> X {
                let mut actor = actor.write().await;
                actor.value += v;
                actor.value
            }
        
            pub async fn met_stat_slf_i_prv(actor: Arc<RwLock<Self>>, v: X) {
                let mut actor = actor.write().await;
                actor.value += v;
            }
        
            pub async fn met_stat_slf_o_prv(actor: Arc<RwLock<Self>>) -> X {
                let actor = actor.write().await;
                actor.value
            }
        
            pub async fn met_stat_slf_void_prv(actor : Arc<RwLock<Self>>) {
                let mut actor = actor.write().await;
                actor.value = -actor.value;
            }
        
            //----------------------------------------------------------------
            // static self public option
            pub async fn met_stat_slf_io_pub_opt(actor : Arc<RwLock<Self>>, v: X) -> Option<X> {
                let mut actor = actor.write().await;
                actor.value += v;
                Some(actor.value)
            }
        
            pub async fn met_stat_slf_o_pub_opt(actor : Arc<RwLock<Self>>) -> Option<X> {
                let actor = actor.read().await;
                Some(actor.value)
            }
        
            // static self public result string
            pub async fn met_stat_slf_io_pub_res_string(actor : Arc<RwLock<Self>>, v: X) -> Result<X,String> {
                let mut actor = actor.write().await;
                actor.value += v;
                Ok(actor.value)
            }
        
            pub async fn met_stat_slf_o_pub_res_string(actor : Arc<RwLock<Self>>) -> Result<X,String> {
                let actor = actor.read().await;
                Ok(actor.value)
            }
        
            // static self public result str
            pub async fn met_stat_slf_io_pub_res_str(actor : Arc<RwLock<Self>>, v: X) -> Result<X,&'static str> {
                let mut actor = actor.write().await;
                actor.value += v;
                Ok(actor.value)
            }
        
            pub async fn met_stat_slf_o_pub_res_str(actor : Arc<RwLock<Self>>) -> Result<X,&'static str> {
                let actor = actor.read().await;
                Ok(actor.value)
            }
        
            //----------------------------------------------------------------
            // static 
            pub fn met_stat_io( v: X ) -> X { v }
        
            pub fn met_stat_i( _v: X ) {}
        
            pub fn met_stat_o() -> X { X::default() }
        
            pub fn met_stat_void() {}
        
            //----------------------------------------------------------------
        }

        pub async fn check_private_methods() {

               
            let fam = 
            // we need the first `i32` the other types are just to test the compilation 
            Actor2Family::<i32,String,11usize>::new(0i32,"bla".to_string(),[0u8;2], ("bla",2u32));

            let Actor2Family{ 
                user1, 
                user2, 
                user3, 
                user4, 
                
                admin} = fam;
            
            //-----------------------------------------
            // static self private 
            //-----------------------------------------
    
            assert_eq!(user1.met_stat_slf_io_prv(1).await,1);
            user2.met_stat_slf_i_prv(1).await;
            user4.met_stat_slf_void_prv().await;
            
            assert_eq!(user3.met_stat_slf_o_prv().await,-2);
            assert_eq!(admin.met_ref_o().await,-2);
        }


    }

    use actor_scope::{ActorFamily,UserActorLive,check_private_methods};

    tokio::runtime::Runtime::new()
    .unwrap()
    .block_on(async {

        check_private_methods().await;

        let fam = 
        // we need the first `i32` the other types are just to test the compilation 
        ActorFamily::<i32,String,11usize>::new(0i32,"bla".to_string(),[0u8;2], ("bla",2u32));

        let ActorFamily{ 
            mut user, 
            user1, 
            user2, 
            user3, 
            user4, 
            user5, 
            user6, 
            
            admin} = fam;

        //------------------------------------------
        // reference
        //-----------------------------------------

        let _ = user.met_ref_io(1).await;
        user.met_ref_i(1).await;
        user.met_ref_void().await;

        assert_eq!(user.met_ref_o().await, -2);
        assert_eq!(user.met_ref_o().await,admin.met_ref_o().await);

        //-----------------------------------------
        // reference local generic
        //-----------------------------------------

        let _ :Vec<i32> = user.met_ref_gen_io(2u8).await;
        user.met_ref_gen_i(1u8).await;
        user.met_ref_gen_void::<String>().await;

        assert_eq!(user.met_ref_gen_o::<i64>().await, -1i64);
        assert_eq!(user.met_ref_gen_o::<i64>().await,admin.met_ref_o().await as i64);


        // `self` consuming methods are ignored by the family actors  
        // instead "static self private" and "static self public"

        /*

        //-----------------------------------------
        // self private
        //-----------------------------------------

        let _ = user._met_slf_io_prv(1);
        user._met_slf_i_prv(1);
        user._met_slf_o_prv();
        user._met_slf_void_prv();

        //-----------------------------------------
        // self public (option)
        //-----------------------------------------

        let _ = user._met_slf_io_pub_opt(1);
        user._met_slf_o_pub_opt();

        //------------------------------------------
        // self public (result string)
        //------------------------------------------

        let _ = user._met_slf_io_pub_res_string(1);
        user._met_slf_o_pub_res_string();
            
        //------------------------------------------
        // self public (result str)
        //------------------------------------------

        let _ = user._met_slf_io_pub_res_str(1);
        user._met_slf_o_pub_res_str();

        */
        
        //-----------------------------------------
        // static reference
        //-----------------------------------------

        let _ = user.met_stat_ref_io(1).await;
        user.met_stat_ref_i(1).await;
        user.met_stat_ref_void().await;
        assert_eq!(user.met_stat_ref_o().await,-1);
        assert_eq!(user.met_stat_ref_o().await, admin.met_ref_o().await);

        
        //-----------------------------------------
        // static self public (option)
        //-----------------------------------------

        let _ = user1.met_stat_slf_io_pub_opt(2).await;
        /* is Clone because all self consuming methods are NOO */
        let user2_clone = user2.clone();  
        assert_eq!(user2_clone.met_stat_slf_o_pub_opt().await,None);

        assert_eq!(user2.met_stat_slf_o_pub_opt().await,Some(1i32));

        
        //-----------------------------------------
        // static self public (result string)
        //-----------------------------------------

        let _ = user3.met_stat_slf_io_pub_res_string(1).await;
        /* is Clone because all self consuming methods are NOO */
        let user4_clone = user4.clone();  
        assert_eq!(user4_clone.met_stat_slf_o_pub_res_string().await,Result::Err("multiple `Live` instances own the actor".to_string()));
        
        assert_eq!(user4.met_stat_slf_o_pub_res_string().await,Result::Ok(2i32));

        //-----------------------------------------
        // static self public (result str)
        //-----------------------------------------

        let _ = user5.met_stat_slf_io_pub_res_str(1).await;
        /* is Clone because all self consuming methods are NOO */
        let user6_clone = user6.clone();  
        assert_eq!(user6_clone.met_stat_slf_o_pub_res_str().await,Result::Err("multiple `Live` instances own the actor"));
        
        assert_eq!(user6.met_stat_slf_o_pub_res_str().await,Result::Ok(3i32));
        

        //-----------------------------------------
        // static 
        //-----------------------------------------

        assert_eq!(UserActorLive::<i32,String,11usize>::met_stat_io(1),1);
        assert_eq!(UserActorLive::<i32,String,11usize>::met_stat_i(1),());
        assert_eq!(UserActorLive::<i32,String,11usize>::met_stat_o(),0);
        assert_eq!(UserActorLive::<i32,String,11usize>::met_stat_void(),());

        // final checks
        assert_eq!(user.met_stat_ref_o().await,3i32);
        assert_eq!(user.met_stat_ref_o().await, admin.met_ref_o().await);

        


    });

}


#[test]
fn async_std_async_generic_family_rw_lock(){

    mod actor_scope {

        use std::sync::Arc;
        use async_std::sync::RwLock;

        pub struct Actor<X,Y, const N:usize> {
            pub value: X,
            _private_generic_field:Y,
        }
        
        // bound and unbound channels 
        #[interthread::family( show, debut,lib="async_std",
            actor( first_name = "User" , channel=3),
            actor( first_name = "User1", channel=3),
            actor( first_name = "User2", channel=0,include(met_stat_slf_o_pub_opt)),
            actor( first_name = "User3", channel=0),
            actor( first_name = "User4", channel=3,include(met_stat_slf_o_pub_res_string)),
            actor( first_name = "User5", channel=3),
            actor( first_name = "User6", channel=0,include(met_stat_slf_o_pub_res_str)),
            actor( first_name = "Admin", include(met_ref_o)),
        )] 
        
        #[interthread::family(show, debut,lib="async_std", name="Actor2",
        actor( first_name = "User1", include(met_stat_slf_io_prv)),
        actor( first_name = "User2", include(met_stat_slf_i_prv)),
        actor( first_name = "User3", include(met_stat_slf_o_prv)),
        actor( first_name = "User4", include(met_stat_slf_void_prv)),
        actor( first_name = "Admin", include(met_ref_o)),
        )]
        
        impl<X,Y,const N:usize> Actor<X,Y,N> 
        where X: std::ops::AddAssign+ std::ops::Neg<Output = X> + Default + Copy
        {
        
            pub fn new<A,B>(value:X, _private_generic_field: Y,_a:A,_b:B) -> Self {
               Self { value, _private_generic_field } 
            }
        
            //----------------------------------------------------------------
            // reference
            pub async fn met_ref_io(&mut self, v: X) -> X {
                self.value += v;
                self.value
            }
        
            pub async fn met_ref_i(&mut self, v: X) {
                self.value += v;
            }
        
            pub async fn met_ref_o(&self) -> X {
                self.value
            }
        
            pub async fn met_ref_void(&mut self) {
                self.value = -self.value;
            }
        
            //----------------------------------------------------------------
            // reference local generic
            pub async fn met_ref_gen_io<I,S>(&mut self, v: I) -> S 
            where I: Into<X>,
                  S:  FromIterator<X>,
            {
                self.value += v.into();
                std::iter::once(self.value).collect()
            }
        
            pub async fn met_ref_gen_i<I:Into<X>>(&mut self, v: I) {
                self.value += v.into();
            }
        
            pub async fn met_ref_gen_o<S:From<X>>(&self) -> S {
                self.value.into()
            }
        
            pub async fn met_ref_gen_void<A>(&mut self) {
                self.value = -self.value;
                let msg = std::any::type_name::<A>();
                println!("invoked by - {msg}");
            }
        
            //----------------------------------------------------------------
            // self private
            pub async fn _met_slf_io_prv(mut self, v: X) -> X {
                self.value += v;
                self.value
            }
        
            pub async fn _met_slf_i_prv(mut self, v: X) {
                self.value += v;
            }
        
            pub async fn _met_slf_o_prv(self) -> X {
                self.value
            }
        
            pub async fn _met_slf_void_prv(mut  self) {
                self.value = -self.value;
            }
            
            
            //----------------------------------------------------------------
            // self public option
            pub async fn _met_slf_io_pub_opt(mut self, v: X) -> Option<X> {
                self.value += v;
                Some(self.value)
            }
        
            pub async fn _met_slf_o_pub_opt(self) -> Option<X> {
                Some(self.value)
            }
        
            // self public result string
            pub async fn _met_slf_io_pub_res_string(mut self, v: X) -> Result<X,String> {
                self.value += v;
                Ok(self.value)
            }
        
            pub async fn _met_slf_o_pub_res_string(self) -> Result<X,String> {
                Ok(self.value)
            }
        
            // self public result str
            pub async fn _met_slf_io_pub_res_str(mut self, v: X) -> Result<X,&'static str> {
                self.value += v;
                Ok(self.value)
            }
        
            pub async fn _met_slf_o_pub_res_str(self) -> Result<X,&'static str> {
                Ok(self.value)
            }
            
            
            //----------------------------------------------------------------
            // static reference
            pub async fn met_stat_ref_io( actor: &Arc<RwLock<Self>>, v: X) -> X {
                let mut actor = actor.write().await;
                actor.value += v;
                actor.value
            }
        
            pub async fn met_stat_ref_i(actor: &Arc<RwLock<Self>>, v: X) {
                let mut actor = actor.write().await;
                actor.value += v;
            }
        
            pub async fn met_stat_ref_o(actor: &Arc<RwLock<Self>>) -> X {
                let actor = actor.read().await;
                actor.value
            }
        
            pub async fn met_stat_ref_void(actor : &Arc<RwLock<Self>>) {
                let mut actor = actor.write().await;
                actor.value = -actor.value;
            }
        
            //----------------------------------------------------------------
            // static self consuming 
            pub async fn met_stat_slf_io_prv( actor: Arc<RwLock<Self>>, v: X) -> X {
                let mut actor = actor.write().await;
                actor.value += v;
                actor.value
            }
        
            pub async fn met_stat_slf_i_prv(actor: Arc<RwLock<Self>>, v: X) {
                let mut actor = actor.write().await;
                actor.value += v;
            }
        
            pub async fn met_stat_slf_o_prv(actor: Arc<RwLock<Self>>) -> X {
                let actor = actor.write().await;
                actor.value
            }
        
            pub async fn met_stat_slf_void_prv(actor : Arc<RwLock<Self>>) {
                let mut actor = actor.write().await;
                actor.value = -actor.value;
            }
        
            //----------------------------------------------------------------
            // static self public option
            pub async fn met_stat_slf_io_pub_opt(actor : Arc<RwLock<Self>>, v: X) -> Option<X> {
                let mut actor = actor.write().await;
                actor.value += v;
                Some(actor.value)
            }
        
            pub async fn met_stat_slf_o_pub_opt(actor : Arc<RwLock<Self>>) -> Option<X> {
                let actor = actor.read().await;
                Some(actor.value)
            }
        
            // static self public result string
            pub async fn met_stat_slf_io_pub_res_string(actor : Arc<RwLock<Self>>, v: X) -> Result<X,String> {
                let mut actor = actor.write().await;
                actor.value += v;
                Ok(actor.value)
            }
        
            pub async fn met_stat_slf_o_pub_res_string(actor : Arc<RwLock<Self>>) -> Result<X,String> {
                let actor = actor.read().await;
                Ok(actor.value)
            }
        
            // static self public result str
            pub async fn met_stat_slf_io_pub_res_str(actor : Arc<RwLock<Self>>, v: X) -> Result<X,&'static str> {
                let mut actor = actor.write().await;
                actor.value += v;
                Ok(actor.value)
            }
        
            pub async fn met_stat_slf_o_pub_res_str(actor : Arc<RwLock<Self>>) -> Result<X,&'static str> {
                let actor = actor.read().await;
                Ok(actor.value)
            }
        
            //----------------------------------------------------------------
            // static 
            pub async fn met_stat_io( v: X ) -> X { v }
        
            pub async fn met_stat_i( _v: X ) {}
        
            pub async fn met_stat_o() -> X { X::default() }
        
            pub async fn met_stat_void() {}
        
            //----------------------------------------------------------------
        }

        pub async fn check_private_methods() {

               
            let fam = 
            // we need the first `i32` the other types are just to test the compilation 
            Actor2Family::<i32,String,11usize>::new(0i32,"bla".to_string(),[0u8;2], ("bla",2u32));

            let Actor2Family{ 
                user1, 
                user2, 
                user3, 
                user4, 
                
                admin} = fam;
            
            //-----------------------------------------
            // static self private 
            //-----------------------------------------
    
            assert_eq!(user1.met_stat_slf_io_prv(1).await,1);
            user2.met_stat_slf_i_prv(1).await;
            user4.met_stat_slf_void_prv().await;
            
            assert_eq!(user3.met_stat_slf_o_prv().await,-2);
            assert_eq!(admin.met_ref_o().await,-2);
        }


    }

    use actor_scope::{ActorFamily,UserActorLive,check_private_methods};

    tokio::runtime::Runtime::new()
    .unwrap()
    .block_on(async {

        check_private_methods().await;

        let fam = 
        // we need the first `i32` the other types are just to test the compilation 
        ActorFamily::<i32,String,11usize>::new(0i32,"bla".to_string(),[0u8;2], ("bla",2u32));

        let ActorFamily{ 
            mut user, 
            user1, 
            user2, 
            user3, 
            user4, 
            user5, 
            user6, 
            
            admin} = fam;

        //------------------------------------------
        // reference
        //-----------------------------------------

        let _ = user.met_ref_io(1).await;
        user.met_ref_i(1).await;
        user.met_ref_void().await;

        assert_eq!(user.met_ref_o().await, -2);
        assert_eq!(user.met_ref_o().await,admin.met_ref_o().await);

        //-----------------------------------------
        // reference local generic
        //-----------------------------------------

        let _ :Vec<i32> = user.met_ref_gen_io(2u8).await;
        user.met_ref_gen_i(1u8).await;
        user.met_ref_gen_void::<String>().await;

        assert_eq!(user.met_ref_gen_o::<i64>().await, -1i64);
        assert_eq!(user.met_ref_gen_o::<i64>().await,admin.met_ref_o().await as i64);


        // `self` consuming methods are ignored by the family actors  
        // instead "static self private" and "static self public"

        /*

        //-----------------------------------------
        // self private
        //-----------------------------------------

        let _ = user._met_slf_io_prv(1);
        user._met_slf_i_prv(1);
        user._met_slf_o_prv();
        user._met_slf_void_prv();

        //-----------------------------------------
        // self public (option)
        //-----------------------------------------

        let _ = user._met_slf_io_pub_opt(1);
        user._met_slf_o_pub_opt();

        //------------------------------------------
        // self public (result string)
        //------------------------------------------

        let _ = user._met_slf_io_pub_res_string(1);
        user._met_slf_o_pub_res_string();
            
        //------------------------------------------
        // self public (result str)
        //------------------------------------------

        let _ = user._met_slf_io_pub_res_str(1);
        user._met_slf_o_pub_res_str();

        */
        
        //-----------------------------------------
        // static reference
        //-----------------------------------------

        let _ = user.met_stat_ref_io(1).await;
        user.met_stat_ref_i(1).await;
        user.met_stat_ref_void().await;
        assert_eq!(user.met_stat_ref_o().await,-1);
        assert_eq!(user.met_stat_ref_o().await, admin.met_ref_o().await);

        
        //-----------------------------------------
        // static self public (option)
        //-----------------------------------------

        let _ = user1.met_stat_slf_io_pub_opt(2).await;
        /* is Clone because all self consuming methods are NOO */
        let user2_clone = user2.clone();  
        assert_eq!(user2_clone.met_stat_slf_o_pub_opt().await,None);

        assert_eq!(user2.met_stat_slf_o_pub_opt().await,Some(1i32));

        
        //-----------------------------------------
        // static self public (result string)
        //-----------------------------------------

        let _ = user3.met_stat_slf_io_pub_res_string(1).await;
        /* is Clone because all self consuming methods are NOO */
        let user4_clone = user4.clone();  
        assert_eq!(user4_clone.met_stat_slf_o_pub_res_string().await,Result::Err("multiple `Live` instances own the actor".to_string()));
        
        assert_eq!(user4.met_stat_slf_o_pub_res_string().await,Result::Ok(2i32));

        //-----------------------------------------
        // static self public (result str)
        //-----------------------------------------

        let _ = user5.met_stat_slf_io_pub_res_str(1).await;
        /* is Clone because all self consuming methods are NOO */
        let user6_clone = user6.clone();  
        assert_eq!(user6_clone.met_stat_slf_o_pub_res_str().await,Result::Err("multiple `Live` instances own the actor"));
        
        assert_eq!(user6.met_stat_slf_o_pub_res_str().await,Result::Ok(3i32));
        

        //-----------------------------------------
        // static 
        //-----------------------------------------

        assert_eq!(UserActorLive::<i32,String,11usize>::met_stat_io(1).await,1);
        assert_eq!(UserActorLive::<i32,String,11usize>::met_stat_i(1).await,());
        assert_eq!(UserActorLive::<i32,String,11usize>::met_stat_o().await,0);
        assert_eq!(UserActorLive::<i32,String,11usize>::met_stat_void().await,());

        // final checks
        assert_eq!(user.met_stat_ref_o().await,3i32);
        assert_eq!(user.met_stat_ref_o().await, admin.met_ref_o().await);

    });

}

