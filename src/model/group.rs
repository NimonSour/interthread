
use crate::model::{self,name,get_ident_type_generics,MpscChannel,Vars,Cont,method,generics,live_static_method,Lib,Model,ActorAttributeArguments,GroupAttributeArguments};
use crate::error;

use syn::{ItemImpl,Type,Ident,Visibility};
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{quote,format_ident};
use std::collections::BTreeMap;



/*
// in other file 
pub struct Cc;

// in this file  
struct Aa;
struct Bb;

pub struct AaBbCc {
    pub a: Aa,
    pub b: Bb, 
    pub c: Cc,
    n: AnyOtherType,
}




1) 'file' argument for current file 
    #[interthread::group(
        file="path/to/this/file.rs",
        Cc = "path/to/other/file.rs"    or c = "path/to/other/file.rs" or     c(path="path/to/other/file.rs")
    )]

    path( a::path("path/to/a.rs"), b::path("path/to/b.rs"))
     

    path( a::path="path/to/a.rs", b::path="path/to/b.rs")


2) Find and get the fields of struct in file.

    a) get the name from item_impl 
    b) find enum or struct with the same name
                if enum return an error  group works for structs only  
    c) get first impl block of the object 
    d)  

    struct_ visibility
    field ( ident, type, visibility )

vis  group_name
pub GroupLive {
    inter_sender : mpsc::Sender,

    field_ident : 
    fielf: LiveGroup,
}

impl GroupLive {

    pub fn new() -> Self {

    }
}
*/


pub fn group_model( 
    gaa: &GroupAttributeArguments, 
    item_impl: &ItemImpl,
    mac: Model,
    mut new_vis: Option<Visibility>
){

    
        let mut members = BTreeMap::new();

        for key in gaa.members.keys().cloned(){

            let aaa = gaa.get_aaa(Some(&key));
            let ( item_impl,vis ) = gaa.members[&key].clone();

            let act = 
            crate::model::actor_model( aaa, &item_impl,Model::Group, Some(vis) );
        
            members.insert(key,act);
        }

        let aaa = gaa.get_aaa(None);


        //---------------(1)

        let mut cont = model::Cont::new();

        let (actor_name,
            actor_type,
            generics) = get_ident_type_generics(item_impl);
        
    
    
        let ( mut actor_methods, 
              mut met_new) =
             method::get_methods( &actor_type,item_impl.clone(),aaa.assoc ,&mac);
    
    
        let mut model_generics = generics.clone();
        // let actor_ty_generics  = generics.split_for_impl().1;
    
        let ( impl_generics,
                ty_generics,
               where_clause ) = {
    
            let mut sigs = actor_methods.iter_mut().map(|m| m.get_mut_sig()).collect::<Vec<_>>();
    
            if met_new.is_some() {
                let mut mn = met_new.unwrap();
                sigs.push(mn.get_mut_sig());
                generics::get_parts( &mut model_generics, sigs);
    
                met_new = Some(mn);
    
            } else { generics::get_parts( &mut model_generics, sigs); }
            
            model_generics.split_for_impl()
    
        };
        
        // Giving a new name if specified 
    
        let vars = &Vars::new(&aaa,&actor_name,Model::Actor,&mac) ;
        let Vars { script_name,live_name,.. } = vars;
        let script_type: Type = syn::parse_quote!{ #script_name #ty_generics };
        let live_type: Type   = syn::parse_quote!{ #live_name #ty_generics };
        let (oneshot,mpsc) = &model::get_channels_one_mpsc(&aaa,vars,&script_type);
        
    
        let async_decl   =  match &aaa.lib {
            Lib::Std => {
                if let Some(pos) = actor_methods.iter().position(|x| x.is_async()){
                    error::abort_async_no_lib(&actor_name,&actor_methods[pos]);
                }
                None
            },
            _ => { Some(quote!{async}) },
        };
    
        let Vars{actor,play,direct,
                 debut, msg,debut_play,
                 sender,receiver,name,..} = vars;
        
    
        method::to_raw_parts( vars,&mut cont,&aaa,actor_methods,oneshot,mpsc );
    
    
        // This is file_path for legend 
        let ( script_legend_file, live_legend_file ) = 
        if aaa.debut.is_legend(){
            let (s,l) = crate::show::check_legend_path(&mac, &vars.cust_name, &aaa.debut.path.as_ref().unwrap());
            (Some(s),Some(l))
        } else {
            (None, None)
        };
    
    //-----------(2)
        /*
        
        raw parts to

            &mut debug_arms,
            &mut direct_arms,
            &mut script_fields, 
        foo: MyType,
        Foo(MyTypeScriptGroup)
        */

    if Model::Actor.eq(&mac) { 

        if met_new.is_none() {

            let msg = format!("Can not find public/restricted  method `new` or `try_new` for {:?} object.",actor_name.to_string());
            let (note,help) = error::met_new_note_help(&actor_name);
            abort!(item_impl,msg;note=note;help=help);
        }
        
        // Change visibility of model methods 
        new_vis = met_new.as_ref().map(|m| m.vis.clone());

        let met_new         = met_new.unwrap();
        let new_sig             = &met_new.new_sig;
        let func_new_name           = &new_sig.ident;
        let (args_ident, _ )   = method::arguments_pat_type(&met_new.get_arguments());
        let unwrapped          = met_new.unwrap_sign();
        let vis                = &met_new.vis.clone();

        let (init_live, play_args) = {
            if aaa.debut.active() {
                (quote!{ Self { #sender,#debut: std::sync::Arc::clone(&#debut), #name : format!("{:?}",* #debut) }} ,
                    quote!{ #receiver, #actor, #debut_play})
            } else {

                (quote!{ Self{ #sender } }, 
                    quote!{ #receiver, #actor } )
            }
        };

        let spawn = aaa.lib.method_new_spawn(&play_args,script_name);
        let turbofish = ty_generics.as_turbofish();

        let vars_debut = 
        if aaa.debut.active() {
            quote!{let #debut =  #script_name #turbofish ::#debut();
                    let #debut_play = *std::sync::Arc::clone(&#debut); }
        } else {quote!{}};

        let return_statement   = met_new.live_ret_statement(&init_live);
        
        let MpscChannel{declaration, ..} = mpsc;
        let Cont{live_mets,..} = &mut cont;

        let func_new_body = quote!{

            #vis #new_sig {
                let #actor = #actor_name:: #func_new_name #args_ident #unwrapped;
                #declaration
                #vars_debut
                #spawn
                #return_statement
            }
        };

        live_mets.insert(0,(new_sig.ident.clone(),func_new_body));
    };

        
    

    // LIVE INTER METHODS AND TRAITS
    if aaa.debut.active(){
        aaa.debut.impl_debut( &mut cont, vars, &new_vis, &ty_generics, &where_clause)
    }
    
    // SCRIPT DEFINITION
    let script_def = {
        let Cont{ script_fields,..} = &mut cont;
        quote! {
            #new_vis enum #script_name #ty_generics #where_clause {
                #(#script_fields),*
            }
        }
    };        


    // DIRECT
    {
        let Cont{script_mets,direct_arms,..} = &mut cont;
        script_mets.push((direct.clone(),
        quote!{
            #new_vis #async_decl fn #direct (self, #actor: &mut #actor_type /*#actor_ty_generics*/ ) {
                match self {
                    #(#direct_arms)*
                }
            }
        }));
    }


    // PLAY
    if Model::Actor.eq(&mac) {

        let await_call  = async_decl.as_ref().map(|_| quote!{.await});
        // let recv_await    =  play_async_decl.as_ref().map(|_| quote!{.await});
        let end_of_play = error::end_of_life( &actor_name, &aaa.debut.clone() ); // <- include 
        
        let debut_pat_type = if aaa.debut.active(){quote!{,#debut: std::time::SystemTime }} else { quote!{} };

        let MpscChannel{pat_type_receiver,..}      = mpsc;
        let Cont{script_mets,..} = &mut cont;
        let play_method = {
        
            let ok_or_some = match aaa.lib {
                Lib::Tokio => quote!{Some},
                _ => quote!{Ok}
            };
            quote! {
                #new_vis #async_decl fn #play ( #pat_type_receiver mut #actor: #actor_type /*#actor_ty_generics*/ #debut_pat_type ) {
                    while let #ok_or_some (#msg) = #receiver.recv() #await_call {
                        #msg.#direct ( &mut #actor ) #await_call;
                    }
                    #end_of_play
                }
            }
        };
        script_mets.push(( play.clone(), play_method ));

    }

    // SCRIPT TRAIT (Debug)
    {   
        let Cont{ script_trts,debug_arms,..} = &mut cont;
        let str_script_name = script_name.to_string();
        let body = 
        if debug_arms.is_empty() { 
            quote!{ write!(f, #str_script_name )} 
        } else {
            quote!{ match self { #(#debug_arms)* } }
        };
        script_trts.push((format_ident!("Debug"),
        quote! {
            impl #ty_generics std::fmt::Debug for #script_name #ty_generics #where_clause {
        
                fn fmt( &self, f: &mut std::fmt::Formatter<'_> ) -> std::fmt::Result {
                    #body
                }
            }
        }));
    }


    // LIVE DEFINITION
    let live_def = {
        let MpscChannel{pat_type_sender,..} = &mpsc;
        if Model::Actor.eq(&mac) {
            let (debut_field, name_field) = if aaa.debut.active() {
                ( quote!{ pub #debut: std::sync::Arc<std::time::SystemTime>,},
                quote!{ pub #name: String,} )
            } else { (quote!{}, quote!{})};   
            
            quote!{
                #[derive(Clone)]
                #new_vis struct #live_name #ty_generics #where_clause {
                    #pat_type_sender
                    #debut_field
                    #name_field
                }
            }
        } else { 

            quote!{
                #[derive(Clone)]
                #new_vis struct #live_name #ty_generics #where_clause {
                    #pat_type_sender
                }
            }

        }

    }; 
    //----------(3)


// -------------------------------------------------------------------



        // if Model::Group.eq(&mac){
        //     //we have to extract play from the model
        //     // ???
        //     let play = format_ident!("play");
        

        // pub struct GroupModelSdpl {
        //     pub name:        Ident,
        //     pub edit:    EditGroup,
        //     pub actors: Vec<ActorModelSdpl>,
        // }



        // for i in gaa.members

        // }
        // crate::model::ActorModelSdpl {
        //     name:          cust_name,
        //     mac:         mac.clone(),
        //     edit:           aaa.edit,
        //     generics: model_generics,
        //     script: ( script_def, script_mets, script_trts ),
        //     live:   (   live_def,   live_mets,   live_trts ),
        // }
}






pub fn macro_group_generate_code(
    gaa: GroupAttributeArguments, 
    item_impl: ItemImpl ) 
    -> ( TokenStream, TokenStream ) {

        todo!()

}