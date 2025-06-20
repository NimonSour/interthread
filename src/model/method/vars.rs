
use crate::model::{ generics::turbofish, get_ident_type_generics, name, ActorAttributeArguments, Mac, ModelGenerics, MpscChannel, OneshotChannel };
use super::actor_method::{MethodNew, ModelMethod};

use proc_macro_error::abort_call_site;
use syn::{ parse_quote,Generics, Ident, ItemImpl, Type, Token, TypePath, Visibility};

use quote::{quote,format_ident};
use proc_macro2::TokenStream;
use std::collections::HashSet;

#[derive(Clone)]
pub struct ConstVars {

    pub actor:             Ident,
    pub short_actor:       Ident,
    pub short_error:       Ident,
    pub name:              Ident,
    pub debut:             Ident,
    pub sender:            Ident,
    pub receiver:          Ident,
    pub play:              Ident,
    pub direct:            Ident,
    pub inter_send:        Ident,
    pub inter_recv:        Ident,
    pub inter_get_debut:   Ident,
    pub inter_get_count:   Ident,
    pub inter_set_name:    Ident,
    pub inter_get_name:    Ident,
    pub inter_play_stop:   Ident,
    pub intername:         Ident,
    pub msg:               Ident,
    pub self_:             Ident,
    pub big_self:           Type,
    pub inter_msg:         Ident,

}


impl ConstVars {

    pub fn new()  -> Self {
        
        Self {
            actor:             format_ident!("actor"),
            short_actor:       format_ident!("act"),
            short_error:       format_ident!("e"),
            name:              format_ident!("name"),
            debut:             format_ident!("debut"),
            sender:            format_ident!("sender"),
            receiver:          format_ident!("receiver"),
            play:              format_ident!("play"),
            direct:            format_ident!("direct"),
            inter_send:        format_ident!("inter_send"),
            inter_recv:        format_ident!("inter_recv"),
            inter_get_debut:   format_ident!("inter_get_debut"),
            inter_get_count:   format_ident!("inter_get_count"),
            inter_set_name:    format_ident!("inter_set_name"),
            inter_get_name:    format_ident!("inter_get_name"),
            inter_play_stop:   format_ident!("inter_play_stop"),
            intername:         format_ident!("InterName"),
            msg:               format_ident!("msg"),
            self_:             format_ident!("self"),
            big_self:          parse_quote!(Self),
            inter_msg:         format_ident!("inter_msg"),

        }


    }

    pub fn get_inter_mets_set(&self, aaa: &ActorAttributeArguments) -> HashSet<Ident> {

        let mut mets = vec![self.inter_msg.clone()]; 
        if aaa.debut.active(){
            mets.extend(vec![
                self.inter_get_debut.clone(),
                self.inter_get_count.clone(),
                self.inter_set_name.clone(),
                self.inter_get_name.clone(),
            ].into_iter());
        } 
        mets.into_iter().collect()
    }

}


#[derive(Clone)]
pub struct ImplVars {

    pub _actor_name:          Ident,
    pub _actor_type:       TypePath,
    pub model_actor_type:  TypePath,
    pub actor_turbo:       TypePath,
    pub _actor_gen:        Generics,

    pub script_name:          Ident,
    pub script_turbo:      TypePath,
    pub script_type:       TypePath,


    pub live_name:            Ident,
    pub live_type:         TypePath,
    pub mod_gen:      ModelGenerics,

    pub actor_methods: Vec<ModelMethod>,
    pub met_new: Option<MethodNew>,
    
    pub oneshot: OneshotChannel,
    pub mpsc:       MpscChannel,

    pub const_vars: ConstVars,
    pub vis:        Visibility,
    pub async_decl: Option<TokenStream>,

    pub direct_play_mut_token: TokenStream,
    pub not_send_play_gen: Option<Generics>,

}

impl ImplVars {

    pub fn new( aaa: &ActorAttributeArguments,item_impl: &ItemImpl) -> Self {

        let const_vars = ConstVars::new();
        let (actor_name, actor_type,actor_gen) = get_ident_type_generics(item_impl);

        let model_actor_type = aaa.mod_receiver.get_model_type(&actor_type, &aaa.lib);

        let actor_turbo = turbofish::from_type_path(&actor_type);
        
        let (script_name,live_name) = get_script_live_name(&actor_name,aaa);

        let mut imw = super::ImplWork::new(&actor_type,&actor_gen,&aaa);
        imw.process_impl(item_impl);

        // model generics
        let mut mod_gen = imw.get_mod_gen();
        
        // model types
        let script_type  = get_type_path(&script_name, &mod_gen.script_gen);
        let live_type= get_type_path(&live_name,   &mod_gen.live_gen);
        let script_turbo = turbofish::from_type_path(&script_type);

        // channels 
        let (oneshot,mpsc) = get_channels_one_mpsc( &const_vars,&aaa,&script_type, &live_name);

        // methods 
        let ( mut met_new, actor_methods,async_decl ) = imw.get_methods(&oneshot);

        let vis = met_new.as_ref().unwrap().met.vis.clone();

        let direct_play_mut_token = if aaa.mod_receiver.is_slf(){ quote!{ mut } } else { quote!{} };

        let not_send_play_gen = aaa.ty_send.add_model_gen_bounds_and_update_play_gen( met_new.as_mut(), &mut mod_gen); 

        ImplVars {

            _actor_name: actor_name,
            _actor_type: actor_type,
            model_actor_type,
            actor_turbo,
            _actor_gen: actor_gen,

            script_name,
            script_turbo,
            script_type,

            live_name,
            live_type,
            mod_gen,

            
            actor_methods,
            met_new,
            oneshot,
            mpsc,
            const_vars,
            vis,
            async_decl,
            direct_play_mut_token,
            not_send_play_gen
        }
    }

    pub fn get_await_call(&self) -> Option<TokenStream> {
        self.async_decl.as_ref().map(|_| quote!{.await})
    }

    pub fn get_gen_msg_script(&self, asyncness: &Option<Token![async]>, direct_actor_call: &TokenStream ) -> TokenStream { 
        let ConstVars{ actor,inter_msg,.. } = &self.const_vars;
        let ImplVars{ model_actor_type,script_turbo, direct_play_mut_token, ..} = self;

        let script_field_name = name::script_field(inter_msg);
        let async_move_tokens = asyncness.as_ref().map(|x| quote!{ #x move });
        let mut block =  quote!( #async_move_tokens { #direct_actor_call } );
        if async_move_tokens.is_some() { block = quote!{ std::boxed::Box::pin( #block ) }; }

        quote!{ 
            #script_turbo :: #script_field_name (
                std::boxed::Box::new ( move | #actor: & #direct_play_mut_token #model_actor_type | 
                    #block
                )
            )
        }
    }

}

pub fn get_channels_one_mpsc( const_vars: &ConstVars, aaa: &ActorAttributeArguments, script_type: &TypePath, live_name: &Ident ) 
    -> ( OneshotChannel, MpscChannel ){

    let ConstVars{  inter_send,inter_recv,.. } = const_vars;

    let script_type = &Type::Path(script_type.clone());

    (
        OneshotChannel::new(inter_send,inter_recv,&aaa.lib),
        MpscChannel::new(const_vars,aaa,script_type,live_name)   
    )
}

pub fn get_type_path( ident: &Ident, gen: &Generics ) -> TypePath {
    let (_,gen_ty,_) = gen.split_for_impl();
    parse_quote!( #ident #gen_ty )
}


pub fn get_script_live_name( actor_name: &Ident, aaa: &ActorAttributeArguments) ->( Ident, Ident ){
    let actor_name  = if let Some(name) = &aaa.name { name } else { actor_name }; 
    let actor_name = if let Some( first_name) = &aaa.first_name { &format_ident!("{first_name}{actor_name}")} else { actor_name };
    
    match aaa.mac {
        Mac::Actor            => (name::script(&actor_name),      name::live(&actor_name)),
        _ => { abort_call_site!("Internal Error. `vars::get_script_live_name`. unexpected Mac variant"); },
    }
}

