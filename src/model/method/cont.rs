
use super::{ModelMethod,ShowComment,ConstVars,ImplVars,args_to_pat_type};
use crate::{error, model::{ ModelPart,ActorAttributeArguments, MpscChannel,name}};
use syn::{parse_quote,Attribute,Visibility,Ident,Type,ItemImpl,ImplItemFn, Item, Pat};

use proc_macro2::TokenStream;
use quote::{quote,ToTokens};

pub struct Cont {
    
    script_mets  : Vec<(Ident,ImplItemFn)>,
    script_trts  : Vec<(Ident,Item)>,
    script_impl  : ItemImpl,

    live_mets    : Vec<(Ident,ImplItemFn)>,
    live_trts    : Vec<(Ident,Item)>,
    live_impl    : ItemImpl,

    pub script_fields: Vec<TokenStream>,
    pub direct_arms  : Vec<TokenStream>,
    pub debug_arms   : Vec<TokenStream>,

    show:    ShowComment,
    pub play_while_block: Box<dyn Fn(TokenStream) -> TokenStream>,
    pub live_clone_attr: Option<Attribute>,
    pub flag_slf: bool,
    pub flag_gen: bool,

}

impl Cont {

    pub fn new( aaa: &ActorAttributeArguments, impl_vars: &ImplVars) -> Self {
            let ImplVars {  script_type,live_type,mod_gen,.. } = &impl_vars;
            let (script_impl, live_impl) = mod_gen.get_script_live_impl_block(script_type,live_type);

        Self{
            
            script_mets  : vec![],
            script_trts  : vec![],
            script_impl,

            live_mets    : vec![],
            live_trts    : vec![],
            live_impl,

            script_fields: vec![],
            direct_arms  : vec![],
            debug_arms   : vec![],

            show: aaa.show.clone(),
            play_while_block: Box::new(|v| {v}),
            live_clone_attr:  Some( parse_quote!( #[derive(Clone)])),
            flag_slf     : false,
            flag_gen     : false,
        }
    }

    // Debug arm
    pub fn add_debug_arm(&mut self, script_name: &Ident, ident: &Ident ) {

        let script_field_name = name::script_field(ident);
        let str_field_name = format!("{}::{}",script_name.to_string() ,ident.to_string());
        let debug_arm = quote! {
            #script_name :: #script_field_name {..} => write!(f, #str_field_name),
        };
        self.debug_arms.push(debug_arm);
    }



pub fn to_raw_parts (&mut self, impl_vars: &ImplVars, aaa : &ActorAttributeArguments )
{  
    let ActorAttributeArguments{ lib,mod_receiver,.. } = &aaa;

    let ImplVars{ script_turbo, model_actor_type, actor_turbo,
        live_name,script_name,actor_methods,const_vars,mpsc, oneshot,async_decl,
        direct_play_mut_token, .. } = &impl_vars;
    
    let ConstVars {actor,inter_send,inter_msg, 
        inter_get_count, inter_play_stop,msg,debut, 
        self_,big_self, sender, receiver,..} = &const_vars;
    
    let MpscChannel{ sender_call,type_receiver,type_sender,.. } =  mpsc;

    let live_meth_send_recv = oneshot.decl(None);

    let invoke = 
    | met_ident: &Ident, args_idents: &Vec<Box<Pat>>,turbo_gen: &Option<TokenStream>, is_stat: bool| -> TokenStream 
    {
        if is_stat {
            quote!{ #actor_turbo :: #met_ident #turbo_gen ( #actor , #(#args_idents),*) }
        } else {
            quote!{ #actor . #met_ident #turbo_gen ( #(#args_idents),* ) }
        }
    };
    
    for mut method in actor_methods.clone() {
        
        let( ident, script_field_name) = method.get_ident_field_name();
        method.to_async(lib);
        let error_send = error::direct_send(&script_name,&script_field_name);

        match &mut method {

            ModelMethod::Io   { met, await_call, is_stat, is_mut, turbo_gen, inter_vars, args, output,.. } => {
                
                let (args_idents,_) = args_to_pat_type(&args);

                let actor_lock = mod_receiver.get_lock( &aaa.lib, const_vars, *is_mut, *is_stat );
                let actor_invoke = invoke( &ident, &args_idents, turbo_gen, *is_stat );

                let direct_actor_call = &quote!{ #actor_lock  #inter_send .send( #actor_invoke #await_call ) #error_send ;};

                let msg_variant = if turbo_gen.is_none(){
                    // Debug Arm push
                    if aaa.trait_debug { self.add_debug_arm(script_name,&ident);}

                    // Direct Arm
                    let arm_match  = quote! { #script_field_name { #(#args_idents),* ,  #inter_send } };
                    let direct_arm = quote! { #big_self :: #arm_match => { #direct_actor_call } };
                    self.push_direct_arm(direct_arm);

                    // Script Field 
                    let send_pat_type = oneshot.pat_type_send(&output);
                    let script_field = quote! { #script_field_name { #(#args),* , #send_pat_type, } };
                    self.push_script_field(script_field);

                    quote!{ #script_turbo :: #arm_match }
                } else {
                    self.flag_gen = true;
                    impl_vars.get_gen_msg_script(&met.sig.asyncness, direct_actor_call)
                };

                // Live Method
                let recv_output = oneshot.recv_call(live_name,&ident);
                let inter_gets = inter_vars.as_mut().map(|i| i.get_getters_decl());

                met.block = parse_quote!( 
                
                    {
                        #live_meth_send_recv
                        // getters  
                        #inter_gets
                        let #msg = #msg_variant ;
                        #sender_call
                        #recv_output
                    }
                );

                self.push_live_met(&ident,met);

            },

            ModelMethod::I { met, await_call, is_stat, is_mut, turbo_gen, inter_vars, args,..} => {

                let (args_idents,_) = args_to_pat_type(&args);
                let actor_lock = mod_receiver.get_lock( &aaa.lib,const_vars, *is_mut, *is_stat );
                let actor_invoke = invoke( &ident, &args_idents, turbo_gen,*is_stat );
                let direct_actor_call = &quote!{  #actor_lock #actor_invoke  #await_call; };

                let msg_variant = if turbo_gen.is_none(){

                    // Debug Arm push
                    if aaa.trait_debug { self.add_debug_arm(script_name,&ident);}

                    // Direct Arm
                    let arm_match = quote!{ #script_field_name{ #(#args_idents),* } };
                    let direct_arm = quote!{ #big_self :: #arm_match => { #direct_actor_call } };
                    self.push_direct_arm(direct_arm);

                    // Script Field Struct
                    let script_field = quote!{ #script_field_name { #(#args),* } };
                    self.push_script_field(script_field);

                    quote!{ #script_turbo :: #arm_match }
                } else {
                    self.flag_gen = true;
                    impl_vars.get_gen_msg_script(&met.sig.asyncness, direct_actor_call)
                };

                // Live Method
                let inter_gets = inter_vars.as_mut().map(|i| i.get_getters_decl());
                let ret_chan_end = if inter_vars.is_some() { inter_vars.as_ref().unwrap().some_ret_name() } else { None };

                met.block = parse_quote!{
                    {   
                        // getters
                        #inter_gets
                        let #msg = #msg_variant ;
                        #sender_call
                        #ret_chan_end
                    }
                };

                self.push_live_met(&ident,met);
            },

            ModelMethod::O { met, await_call, is_stat, is_mut, turbo_gen, output,.. } => {

                let actor_lock = mod_receiver.get_lock( &aaa.lib,const_vars, *is_mut, *is_stat );
                let actor_invoke = invoke( &ident, &vec![], turbo_gen,*is_stat );
                let direct_actor_call = &quote!{ #actor_lock  #inter_send .send( #actor_invoke #await_call ) #error_send ; };

                let msg_variant = if turbo_gen.is_none(){

                    // Debug Arm push
                    if aaa.trait_debug { self.add_debug_arm(script_name,&ident);}

                    // Direct Arm
                    let arm_match = quote!{ #script_field_name{ inter_send } };
                    let direct_arm = quote!{ #big_self :: #arm_match => { #direct_actor_call } };
                    self.push_direct_arm(direct_arm);

                    // Script Field Struct
                    let send_pat_type = oneshot.pat_type_send(&*output);
                    let script_field = quote!{ #script_field_name { #send_pat_type } };
                    self.push_script_field(script_field);

                    quote!{ #script_turbo :: #arm_match }
                } else {
                    self.flag_gen = true;
                    impl_vars.get_gen_msg_script(&met.sig.asyncness, direct_actor_call)
                };

                // Live Method
                let recv_output = oneshot.recv_call(live_name,&ident);

                met.block = parse_quote!{
                    {
                        #live_meth_send_recv
                        let #msg = #msg_variant ;
                        #sender_call
                        #recv_output
                    }
                };

                self.push_live_met(&ident,met);
            },

            ModelMethod::Void { met, await_call, is_stat, is_mut, turbo_gen,.. } => {

                let actor_lock = mod_receiver.get_lock( &aaa.lib,const_vars, *is_mut, *is_stat);
                let actor_invoke = invoke( &ident, &vec![], turbo_gen,*is_stat );
                let direct_actor_call = &quote!{ #actor_lock #actor_invoke #await_call; };

                let msg_variant = if turbo_gen.is_none(){

                    // Debug Arm push
                    if aaa.trait_debug { self.add_debug_arm(script_name,&ident);}

                    // Direct Arm
                    let arm_match = quote!{ #script_field_name {} };
                    let direct_arm = quote!{ #big_self :: #arm_match => { #direct_actor_call } };
                    self.push_direct_arm(direct_arm);

                    // Script Field Struct
                    let script_field = quote!{ #script_field_name {} };
                    self.push_script_field(script_field);

                    quote!{ #script_turbo :: #arm_match }
                } else {
                    self.flag_gen = true;
                    impl_vars.get_gen_msg_script(&met.sig.asyncness, direct_actor_call)
                };

                // Live Method
                met.block = parse_quote!{
                    {
                        let #msg = #msg_variant ;
                        #sender_call
                    }
                };

                self.push_live_met(&ident,met);
            },

            ModelMethod::Slf{ met, await_call, turbo_gen, is_stat, args_idents, mod_output,.. } => {
                self.flag_slf = true;

                let inter_play_tuple = if aaa.debut.active { quote!{( #actor,_,_,_)}} else { quote!{( #actor,_,_)} };
                let actor_invoke = invoke( &ident, args_idents, turbo_gen, *is_stat );
                let model_await_call = impl_vars.get_await_call();
                let stmts = quote!{
                    let #inter_play_tuple = #self_.inter_play_stop() #model_await_call  ;
                    return #actor_invoke #await_call ;
                };

                // decision on visibility 
                if aaa.debut.active && mod_output.would_return_if_err() {
                    let ret_statement = mod_output.return_err("multiple `Live` instances own the actor");

                    met.block = parse_quote! {
                        {
                            if #self_ . #inter_get_count () <= 1 { #stmts }
                            #ret_statement
                        } 
                    };

                } else {
                    met.block = parse_quote! { { #stmts } };
                    met.vis = Visibility::Inherited;
                    self.live_clone_attr = None;
                };

                self.push_live_met(&ident,met);
            },

            ModelMethod::Stat{ met, await_call, turbo_gen, args_idents,.. } => {

                met.block = parse_quote! { 
                    {
                        #actor_turbo :: #ident #turbo_gen (#(#args_idents),*) #await_call
                    }
                };
                self.push_live_met(&ident,met);

            },
        }
    } 

    // !!! keep this order !!!
    // create one generic script variant
    if self.flag_gen { 

        // Debug Arm push
        if aaa.trait_debug { self.add_debug_arm(script_name,&inter_msg); }

        let script_field_name = name::script_field(&inter_msg);

        // Direct Arm
        let arm_match  = quote! { 
            #script_field_name ( #msg )
        };
        let msg_await_call = impl_vars.get_await_call(); 
        let direct_arm = quote! { #big_self :: #arm_match => { #msg( #actor ) #msg_await_call ;} };

        self.push_direct_arm(direct_arm);


        let gen_msg_ty = if async_decl.is_some() {
            quote!{ std::boxed::Box<dyn for<'inter> FnOnce(&'inter #direct_play_mut_token #model_actor_type) -> ::std::pin::Pin<std::boxed::Box<dyn std::future::Future<Output = ()> + Send + 'inter>> + Send + 'static> }
        } else {
                quote!{ std::boxed::Box<(dyn for<'inter> FnOnce(&'inter #direct_play_mut_token #model_actor_type ) + Send +'static )> }
        };
        
        // Script Struct Field 
        let script_field = quote! {
            #script_field_name ( #gen_msg_ty )
        };

        self.push_script_field(script_field);
    }

    //  create inter_play_stop
    if self.flag_slf {

        let debut_path = aaa.debut.get_debut_path().as_mut().map(|x| quote!{,#x});
        let sig = quote!{     
            #async_decl fn inter_play_stop(self) -> ( #model_actor_type, #type_receiver, #type_sender #debut_path )
        }; 

        // Debug Arm push
        if aaa.trait_debug { self.add_debug_arm(script_name,&inter_play_stop); }

        let script_field_name = name::script_field(&inter_play_stop);
        let error_send = error::direct_send(&script_name,&script_field_name);


        // Direct Arm
        let arm_match  = quote! { 
            #script_field_name { #inter_send }
        };

        self.play_while_block = {
            let variant_match = quote! { let #big_self :: #arm_match = #msg };
            let send_actor_receiver = quote! { let _ = #inter_send .send( (#actor, #receiver) ) #error_send; };

            Box::new(
                move | msg_direct_call: TokenStream |  {
        
                    quote!{
                        if #variant_match{
                            #send_actor_receiver
                            return;
                        } else {
                            #msg_direct_call
                        }
                    }
                }
            )
        };

        let direct_arm = quote! { #big_self :: #script_field_name {..} => ()};
        self.push_direct_arm(direct_arm);

        let slf_debut = if aaa.debut.active { Some(quote!{ ,#self_ . #debut }) } else { None };
        let recv_output = oneshot.recv_call(live_name,inter_play_stop);
        let met: ImplItemFn = parse_quote!(
            #sig {
                #live_meth_send_recv
                let #msg = #script_name :: #arm_match ;
                #sender_call
                let (#actor,#receiver) = #recv_output;
                ( #actor, #receiver, #self_ . #sender #slf_debut )
            }
        );

        self.push_live_met(inter_play_stop, &met );



        // Script Struct Field 
        let output_type: Type =  parse_quote!( ( #model_actor_type, #type_receiver) );
        let send_pat_type = oneshot.pat_type_send(&output_type);

        let script_field = quote! {
            #script_field_name { #send_pat_type }
        };

        self.push_script_field(script_field);
    
    }

}

    pub fn push_script_met<T:ToTokens>(&mut self, ident: &Ident, tokens: &T){ 
        self.script_mets.push(  (ident.clone(), self.show.parse_method(tokens)) );
    }
    pub fn push_script_trt<T:ToTokens>(&mut self, ident: &Ident, tokens: &T){ 
        self.script_trts.push(  (ident.clone(), ShowComment::parse_item(tokens)) );
    }
    pub fn push_live_met<T:ToTokens>(&mut self, ident: &Ident, tokens: &T ){ 
        self.live_mets.push( (ident.clone(), self.show.parse_method(tokens)) );
    }
    pub fn insert_live_met<T:ToTokens>(&mut self, ident: &Ident, tokens: &T ){ 
        self.live_mets.insert(0,(ident.clone(), self.show.parse_method(tokens)) );
    }
    pub fn push_live_trt<T:ToTokens>(&mut self, ident: &Ident, tokens: &T){ 
        self.live_trts.push( (ident.clone(), ShowComment::parse_item(tokens)) );
    }
    
    pub fn push_script_field(&mut self, v:TokenStream){ self.script_fields.push(v);}

    pub fn push_direct_arm(  &mut self, v:TokenStream){ self.direct_arms.push(v);}

    pub fn get_script_part<T:ToTokens>(&self, def: &T ) -> ModelPart {
        ModelPart::new(
            Some(self.show.parse_model_part( def, &self.script_mets, &self.script_trts)),
            self.script_mets.clone(),
            self.script_trts.clone(),
            self.script_impl.clone(),
        )
    }

    pub fn get_live_part<T:ToTokens>(&self, def: &T ) -> ModelPart {
        ModelPart::new(
            Some(self.show.parse_model_part( def, &self.live_mets, &self.live_trts)),
            self.live_mets.clone(),
            self.live_trts.clone(),
            self.live_impl.clone(),
        )
    }


}

