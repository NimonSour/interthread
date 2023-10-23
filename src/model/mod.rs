
pub mod argument;
pub mod attribute;
pub mod generics;
pub mod method;
pub mod name;
pub mod actor;
pub mod group;


pub use argument::*;
pub use attribute::*;
pub use generics::*;
pub use method::*;
pub use name::*;
pub use actor::*;
pub use group::*;


use proc_macro2::TokenStream;
use proc_macro_error::abort;
use syn::{Generics,Type,Ident};
use quote::{format_ident,quote};



// ----------------

pub fn get_channels_one_mpsc( 
            aaa: &ActorAttributeArguments, 
           vars: &Vars, 
    script_type: &Type) -> ( OneshotChannel, MpscChannel ){

    let Vars{  
            inter_send,
            inter_recv,.. } = vars;

    let ActorAttributeArguments{ channel, lib,..} = aaa;

    (
        OneshotChannel::new(inter_send,inter_recv,lib),
        MpscChannel::new(vars,aaa,script_type)   
    )
}
pub struct Cont {
    script_mets  : Vec<(Ident,TokenStream)>,
    script_trts  : Vec<(Ident,TokenStream)>,
    live_mets    : Vec<(Ident,TokenStream)>,
    live_trts    : Vec<(Ident,TokenStream)>,

    script_fields: Vec<TokenStream>,
    direct_arms  : Vec<TokenStream>,
    debug_arms   : Vec<TokenStream>,
}

impl Cont {

    pub fn new() -> Self{
        Self{
            script_mets  : vec![],
            script_trts  : vec![],
            live_mets    : vec![],
            live_trts    : vec![],
            script_fields: vec![],
            direct_arms  : vec![],
            debug_arms   : vec![],
        }
    }
}


// pub static INTER_GET_DEBUT: &'static str = "inter_get_debut";
// pub static INTER_GET_COUNT: &'static str = "inter_get_count";
// pub static INTER_SET_NAME: &'static str  = "inter_set_name";
// pub static INTER_GET_NAME: &'static str  = "inter_get_name";
// pub static INTER_NAME: &'static str      = "InterName";
// pub static DEBUT: &'static str           = "debut";

pub struct Vars {

    actor:           Ident,
    actor_name:      Ident,
    name:            Ident,
    debut:           Ident,
    debut_play:      Ident,
    sender:          Ident,
    receiver:        Ident,
    play:            Ident,
    direct:          Ident,
    inter_send:      Ident,
    inter_recv:      Ident,
    inter_name:      Ident,
    inter_debut:     Ident,
    inter_count:     Ident,
    inter_get_debut: Ident,
    inter_get_count: Ident,
    inter_set_name:  Ident,
    inter_get_name:  Ident,
    intername:       Ident,
    msg:             Ident,

    cust_name:       Ident,
    script_name:     Ident,
    live_name:       Ident,
}


impl Vars {

    pub fn new( aaa: &ActorAttributeArguments, actor_name: &Ident, mac: Model,model: &Model )  -> Self {
        let cust_name  = if aaa.name.is_some(){ aaa.name.clone().unwrap() } else { actor_name.clone() }; 
        let script_name;
        let live_name  ;
        let actor ;
        match (mac,model){
            (Model::Actor,Model::Actor) => {
                actor = format_ident!("actor");
                script_name = name::script(&cust_name);
                live_name   = name::live(&cust_name);
             },
            (Model::Actor,Model::Group)|
            (Model::Group,Model::Actor) => { 
                actor = format_ident!("actor");
                script_name = name::script_group(&cust_name);
                live_name   = name::live_group(&cust_name);
            },
            (Model::Group,Model::Group) => { 
                actor = format_ident!("group");
                script_name = name::group_script(&cust_name);
                live_name   = name::group_live(&cust_name);
            },
        }

        Self{

            actor,
            actor_name:      actor_name.clone(),
            name:            format_ident!("name"),
            debut:           format_ident!("debut"),
            debut_play:      format_ident!("debut_play"),
            sender:          format_ident!("sender"),
            receiver:        format_ident!("receiver"),
            play:            format_ident!("play"),
            direct:          format_ident!("direct"),
            inter_send:      format_ident!("inter_send"),
            inter_recv:      format_ident!("inter_recv"),
            inter_name:      format_ident!("inter_name"),
            inter_debut:     format_ident!("inter_debut"),
            inter_count:     format_ident!("inter_count"),
            inter_get_debut: format_ident!("inter_get_debut"),
            inter_get_count: format_ident!("inter_get_count"),
            inter_set_name:  format_ident!("inter_set_name"),
            inter_get_name:  format_ident!("inter_get_name"),
            intername:       format_ident!("InterName"),
            msg:             format_ident!("msg"),

            cust_name,
            script_name,
            live_name ,
        }


    }
}


pub struct GroupModelSdpl {

    // model: ActorModelSdpl,
    pub name:        Ident,
    // pub mac:         Model,
    pub edit:    EditGroup,
    // pub generics: Generics,
    pub actors: Vec<ActorModelSdpl>,
}

impl GroupModelSdpl {

    pub fn get_edit(&self) -> Edit {
        Edit::Group(self.edit.clone())
    }
}


// Sdpl Actor 

pub struct ActorModelSdpl {
    pub name:        Ident,
    pub asyncness: Option<TokenStream>,
    pub mac:         Model,
    pub edit:    EditActor,
    pub generics: Generics,
    pub script: (  TokenStream,  Vec<(Ident,TokenStream)>,  Vec<(Ident,TokenStream)> ),
    pub live:   (  TokenStream,  Vec<(Ident,TokenStream)>,  Vec<(Ident,TokenStream)> ),
}


impl ActorModelSdpl {

    pub fn get_edit(&self) -> Edit {
        Edit::Actor(self.edit.clone())
    }
    pub fn is_empty(&self) -> bool {
        self.script.0.is_empty() && self.live.0.is_empty() &&
        self.script.1.is_empty() && self.live.1.is_empty() &&
        self.script.2.is_empty() && self.live.2.is_empty()
    }

    pub fn split_edit(&mut self) -> (TokenStream,TokenStream){

        let mut edit_script_def  = None;
        let mut edit_script_mets = None;
        let mut edit_script_trts = None;
    
        let mut edit_live_def  = None;
        let mut edit_live_mets = None;
        let mut edit_live_trts = None;



        let (script,live) = 
        match &self.edit {  EditActor{ script, live, ..  } => {(script.clone(),live.clone())}};
        
        let select = 
        |
        edit_cont: (Option<Vec<(Ident,bool)>>,bool),
        model_cont: &mut Vec<(Ident,TokenStream)>,
        | -> Option<Vec<TokenStream>>
        {
            let cont = edit_select(edit_cont,model_cont);
            if cont.is_empty() { None } else { Some(cont) }
        };

        let diff = 
        | ((def,scope_def),mets,trts): ( (bool,bool), (Option<Vec<(syn::Ident,bool)>>,bool), (Option<Vec<(syn::Ident,bool)>>,bool) ),
          model_def:  &mut TokenStream,
          model_mets: &mut Vec<(Ident,TokenStream)>,
          model_trts: &mut Vec<(Ident,TokenStream)>,
          edit_def:   &mut Option<TokenStream>,
          edit_mets:  &mut Option<Vec<TokenStream>>,
          edit_trts:  &mut Option<Vec<TokenStream>>
        |{
            if def {
                let temp_def = Some(model_def.clone());
                *model_def  = quote!{}; 

                if scope_def {
                    *edit_def = temp_def;
                }
            }
            // original 
            *edit_mets = select(mets,model_mets);
            *edit_trts = select(trts,model_trts);
        };

        diff(
            script,
             &mut self.script.0,
            &mut self.script.1,
            &mut self.script.2,
              &mut edit_script_def,
             &mut edit_script_mets,
             &mut edit_script_trts 
        );

        diff(
            live,
             &mut self.live.0,
            &mut self.live.1,
            &mut self.live.2,
              &mut edit_live_def,
             &mut edit_live_mets,
             &mut edit_live_trts 
        );
        
        let coll_token_stream = 
        |coll: &Vec<(Ident,TokenStream)>| -> Vec<TokenStream> 
        { coll.iter().map(|x| x.1.clone()).collect::<Vec<_>>() };
        // Prepare Token Stream Vecs
        let script_def         = &self.script.0;
        let script_methods = coll_token_stream(&self.script.1);
        let script_traits  = coll_token_stream(&self.script.2);

        let live_def           = &self.live.0;
        let live_methods   = coll_token_stream(&self.live.1);
        let live_traits    = coll_token_stream(&self.live.2);
        
        let(impl_generics,ty_generics ,where_clause) = self.generics.split_for_impl();
        let (script_name,live_name) = name::get_actor_names(&self.name, &self.mac);


        let res_code = quote! {
    
            #script_def
            impl #impl_generics #script_name #ty_generics #where_clause {
                #(#script_methods)*
            }
            #(#script_traits)*
    
            #live_def
            impl #impl_generics #live_name #ty_generics #where_clause {
                #(#live_methods)*
            }
            #(#live_traits)*
    
        };
    
    
        let res_edit_script_mets =  
            edit_script_mets.as_ref().map(|mets| 
                quote!{ 
                    impl #impl_generics #script_name #ty_generics #where_clause {
                        #(#mets)* 
                    }
                }
            );

        let res_edit_script_trts = 
            edit_script_trts.as_ref().map(|trts| 
                quote!{ #(#trts)* }
            );
    
        let res_edit_live_mets = 

            edit_live_mets.as_ref().map(|mets| 
                quote!{ 
                    impl #impl_generics #live_name #ty_generics #where_clause {
                        #(#mets)* 
                    }
                }
            ); 

        let res_edit_live_trts = 
        edit_live_trts.as_ref().map(|trts| 
            quote!{ #(#trts)* }
        );

        let res_edit = quote!{
    
            #edit_script_def
            #res_edit_script_mets
            #res_edit_script_trts
    
            #edit_live_def
            #res_edit_live_mets
            #res_edit_live_trts
        };
        // let msg = res_code.to_string();
        
        // abort!(proc_macro::Span::call_site(),msg );
        (res_code, res_edit)
    
    }

}    


pub fn edit_select((edit_idents,scope): (Option<Vec<(Ident,bool)>>,bool), 
    ident_mets: &mut Vec<(Ident,TokenStream)> ) -> Vec<TokenStream> {

    let mut res = Vec::new();

    if let Some(idents) = edit_idents { 

        if idents.is_empty() {

            let temp_ident_mets = std::mem::replace(ident_mets,Vec::new());
            if scope {
                res = temp_ident_mets.into_iter().map(|x| x.1).collect::<Vec<_>>();
            }
        }

        for (ident,scp) in idents {
            if let Some(pos) = ident_mets.iter().position(|x| x.0 == ident){
                let (_,met)  = ident_mets.remove(pos);
                if scope || scp {
                    res.push(met);
                }
            } else {
                let msg = format!("No method named `{}` in Actor's methods.",ident.to_string());
                abort!(ident,msg);
            }
        }
    } 
    res
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_edit_group() {

        let attr: syn::Attribute = 
        syn::parse_quote!{#[actor( edit(script(imp), 
                                a::edit(live,script(def))))] };

        let mut edit = EditGroup::default();

        for meta in crate::model::attribute::attr_to_meta_list(&attr){

            if meta.path().is_ident("edit"){
                edit.parse(&meta);
            }
        }
        println!("Edit - {:?}", edit);  
    }


    #[test]
    fn parse_types () {

        let actor = quote::format_ident!{"Actor"};
        let actor_ty: syn::Type = syn::parse_quote!(#actor);


        match &actor_ty {

            syn::Type::Path(_) => println!(" True type "),
            _ => println!(" Not a type that I expect"),
        }
        let str_actor_ty = quote!(#actor_ty);

        println!("{str_actor_ty}");

    }
}

