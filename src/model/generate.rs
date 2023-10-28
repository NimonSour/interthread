use crate::error;
use crate::model::{self,get_ident_type_generics,ModelSdpl,AttributeArguments,ActorModelSdpl,MpscChannel,Cont,Vars,method,generic,attribute::ActorAttributeArguments,argument::{Lib,Model}};
use super::{ActorMethodNew,ActorMethod};

use proc_macro_error::abort;
use syn::{Ident,Type,Signature,Generics,ItemImpl,PathSegment,Visibility };
use quote::{quote,format_ident};
use proc_macro2::TokenStream;
use proc_macro::Span;
use std::collections::BTreeMap;
/*
Expectations :

1) Function for both actor and group plus actor-group
2) One return probably as BTreeMap -- <ident, ModelSdpl>  where ident - self or field name.
3) Generate function takes as input ( AttributeArgument, ItemImpl, Option<GroupVars>).

4) New type 
GroupVars{
    vis: Visibility,
    fild: Ident,
    group_script_type: Type
    group_generics: Generics,
    /* other necessary types probably even Vec<ActorMethods> */

} 

5) let (mac,model) = 




*/

pub struct GroupVars {
    vis:         Visibility,
    fild:             Ident,
    group_script_type: Type,
    impl_vars:     ImplVars,
}


/*


    let (actor_name,
        actor_type,
        generics) = get_ident_type_generics(item_impl);
    


    let ( mut actor_methods, 
          mut met_new) =
         method::get_methods( &actor_type,item_impl.clone(),aaa.assoc ,Model::Actor.eq(&mac));


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
    
*/

fn group_direct_add_fields(cont: &mut Cont, vars: &Vars, model_sdpl: &ModelSdpl ){

    // ADD FIELDS MEMBERS
    let Cont{ direct_arms, 
            script_fields,
               debug_arms,..} = cont;

    let Vars{script_name,.. } = vars;
    
    for (field,ams) in model_sdpl.fields.iter(){

        let variant_name = crate::model::name::script_field(field);
        // probably needs the type as well 
        //  not sure if is group script may be script group
        let field_struct_variant_name = &ams.vars.script_name;// name::script_group(&members[field].name);
        // probably I have this error in Inter_send
        // let error_send = error::direct_send(&script_name,&variant_name);
        let await_call = ams.asyncness.as_ref().map(|_|quote!{.await});
        
        
        // Direct Arm
        let arm_match = quote! { 
            #variant_name ( msg )
        };
        let direct_arm = quote! {
            #script_name :: #arm_match => { msg.direct( &mut group. #field )#await_call;} //#await_call  #error_send ;}
        };
        direct_arms.push(direct_arm);


        // Script Struct
        let script_field = quote! { 
            #variant_name ( #field_struct_variant_name )
        };
        script_fields.push(script_field);

        // Debug arm
        let str_field_name = format!("{}::{}",script_name.to_string() ,field.to_string());

        let debug_arm = quote! {
            #script_name :: #variant_name (_) => write!(f, #str_field_name),
        };
        debug_arms.push(debug_arm);
    }
}

#[derive(Clone)]
pub struct ImplVars {

    pub actor_name:               Ident,
    pub actor_type:                Type,
    pub generics:              Generics,
    pub async_decl: Option<TokenStream>,
    pub actor_methods: Vec<ActorMethod>,
    pub met_new: Option<ActorMethodNew>,
    pub model_generics:        Generics,
    pub vis:         Option<Visibility>,
    pub field:            Option<Ident>,
    pub ty:                Option<Type>,
    pub def_gen:       Option<Generics>,
    pub group_script_type: Option<Type>,


}

impl ImplVars {

    pub fn get_group_script_wrapper(&self) -> Box<dyn Fn(TokenStream) -> TokenStream> {
    
        if let Some(gst)  = self.group_script_type.clone(){
            if let Some(field) = &self.field{
                let field_var = crate::model::name::script_field(field);
                return Box::new( move |ts:TokenStream| 
                    quote::quote!{ 
                        #gst :: #field_var ( #ts )
                    }
                );
            }
        }
        Box::new( |ts:TokenStream| ts)
    }
}

pub fn get_impl_vars(
    item_impl: &ItemImpl, 
          aaa: &ActorAttributeArguments,
      def_gen: Option<Generics>,
          mac: Model,
        model: Model) -> ImplVars 
        
 {
    let (actor_name,
        actor_type,
        generics) = get_ident_type_generics(item_impl);
    

    let ( mut actor_methods, 
          mut met_new) =
         method::get_methods( &actor_type,item_impl.clone(),aaa.assoc ,mac.eq(&model));


    let mut model_generics = generics.clone();

    let mut sigs = actor_methods.iter_mut().map(|m| m.get_mut_sig()).collect::<Vec<_>>();
    if met_new.is_some() {

        let mut mn = met_new.unwrap();
        sigs.push(mn.get_mut_sig());
        generic::take_generic_parts( &mut model_generics, sigs, def_gen);
        met_new = Some(mn);

    } else { generic::take_generic_parts( &mut model_generics, sigs,def_gen); }
        

    let async_decl = 

        match &aaa.lib {

            Lib::Std => {
                if let Some(pos) = actor_methods.iter().position(|x| x.is_async()){
                    error::abort_async_no_lib(&actor_name,&actor_methods[pos]);
                }
                None
            },
            _ => { Some(quote!{async}) },
        };

    ImplVars {
        actor_name,
        actor_type,
        generics,
        async_decl,
        actor_methods,
        met_new,
        model_generics,
        vis:               None,
        field:             None,
        ty:                None,
        def_gen:           None,
        group_script_type: None,
    }
}

pub fn generate_model( aa: AttributeArguments, item_impl: &ItemImpl , impl_vars: Option<ImplVars>) 

// -> BTreeMap<Ident,ActorModelSdpl> {
    -> ModelSdpl {


    /*
     Actor None   Actor Actor
     Actor Some   Group Actor
     Group None   Group Group
     */
    // mac model values 

    let mut cont = model::Cont::new();
    let mut model_sdpl = ModelSdpl::new();
    // let mut members = BTreeMap::new();
    let aaa;

    let mac = aa.get_mac();
    let model = if impl_vars.is_some(){ mac.get_invers() } else { mac };


    let impl_vars = 
    //AA
    if mac.eq(&model) {
        match &aa {
            //AA
            AttributeArguments::Actor(aaas) => {
                aaa = aaas.clone();
                get_impl_vars(item_impl, &aaa, None, mac, model)
            },
            //GG
            AttributeArguments::Group(gaas) => {


                
                // herre goes calculation for all group members
                let mut coll_impl_vars = BTreeMap::new();

                for key in gaas.members.keys(){
                
                    let aaa = gaas.get_aaa(Some(key));
                    let ( item_impl,vis,ty,def_gen ) = gaas.members[key].clone();
                
                    let mut impl_vars = get_impl_vars(&item_impl, &aaa, Some(def_gen),mac, model);

                    // add constraints to model_generics from definition generics if any
                    // error
                    let old = impl_vars.generics.clone();
                    let new = impl_vars.model_generics.clone();
                    generic::exam_rename(&old, &new);
                    // end error
                    impl_vars.vis     = Some(vis);
                    impl_vars.field   = Some(key.clone());
                    impl_vars.ty      = Some(ty.clone());
                    // impl_vars.def_gen = Some(def_gen.clone());
                    let aa = AttributeArguments::Actor(aaa);
                
                    coll_impl_vars.insert(key,(aa,item_impl,impl_vars));
                
                }
                aaa = gaas.get_aaa(None);
                let impl_vars = get_impl_vars(&item_impl, &aaa, Some(gaas.def_generics.clone()), mac, model);

                // after the calculations of total model generics (now is just generics)
                // after the generics of model is known 
                // we have to pass a  ..GroupScript  (type with generics)
                let ImplVars{ generics,actor_name,.. } = &impl_vars;
                let cust_name  = &if aaa.name.is_some(){ aaa.name.clone().unwrap() } else { actor_name.clone() };
                let group_script =  crate::model::name::group_script(cust_name);
                let ( _, ty_generics, _ ) = (*generics).split_for_impl();
                
                // create the type of groupt script enum
                let group_script_type: Type = syn::parse_quote!{ #group_script #ty_generics };


                for (key,(aa,item_impl, mut impl_vars)) in coll_impl_vars {
                
                    impl_vars.group_script_type = Some(group_script_type.clone());

                    // call gen again
                    let btm_sdpl = 
                    generate_model(AttributeArguments::Actor(aaa.clone()),&item_impl,Some(impl_vars.clone()));

                    model_sdpl.extend(btm_sdpl);
                }
                get_impl_vars(&item_impl, &aaa, Some(gaas.def_generics.clone()),mac, model)
            }    
        }
    }

    //AG
    else {
        match &aa {

            AttributeArguments::Actor(aaas) => { aaa = aaas.clone();},
            AttributeArguments::Group(_) => {
                abort!(Span::call_site(),"Internal Error 'model::actor_group::generate_model' . Unexpected configuration of AttributeArguments.");
            },
        }
        impl_vars.unwrap() 
    };

    let vars = &Vars::new(&aaa,impl_vars, mac, model);
    let (oneshot,mpsc) = &model::get_channels_one_mpsc(&aaa,vars);
    

    let Vars{
        actor, play, direct,
        debut, msg,  debut_play,
        sender,receiver,name,
        impl_vars,script_name,live_name,
        cust_name, .. } = vars;

    let ImplVars { 
        vis,model_generics,
        async_decl,actor_type,
        actor_name, met_new, .. } = impl_vars;
    
    
    let mut new_vis = vis.as_ref().map(|x| x.clone());
    let ( impl_generics,
        ty_generics,
       where_clause ) = model_generics.split_for_impl();

    method::to_raw_parts( vars,&mut cont,&aaa,oneshot,mpsc );
    

    // need a function to add 
    // for every field  variant in direct Group




    

    if mac.eq(&model) && Model::Group.eq(&mac) {
        // condition if is Group ONLY!!!
        // add some field variants in self direct
        group_direct_add_fields(&mut cont,vars,&model_sdpl);
    }
        
        
 

    // This is file_path for legend
    // this has to go somewhere 

    let ( script_legend_file, live_legend_file ) = 
    if aaa.debut.is_legend(){
    let (s,l) = crate::show::check_legend_path(&mac, &vars.cust_name, &aaa.debut.path.as_ref().unwrap());
    (Some(s),Some(l))
    } else {
    (None, None)
    };


    //-------------(2)

    if mac.eq(&model) { 

        if met_new.is_none() {

            let msg = format!("Can not find public/restricted  method `new` or `try_new` for {:?} object.",actor_name.to_string());
            let (note,help) = error::met_new_note_help(&actor_name);
            abort!(item_impl,msg;note=note;help=help);
        }
        
        // Change visibility of model methods 
        new_vis = met_new.as_ref().map(|m| m.vis.clone());

        let met_new         = met_new.clone().unwrap();
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
        // let ImplVars{ async_decl,actor_type,..} = impl_vars;
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
    if mac.eq(&model) {

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

    //-------------(3)

    // let Vars { cust_name,..} = vars;
    let Cont { script_mets, script_trts,
               live_mets,   live_trts,..} = cont;
    
    let sdpl =  
    crate::model::ActorModelSdpl {

            name:          cust_name.clone(),
            asyncness:    async_decl.clone(),
            mac:                 mac.clone(),
            edit:                   aaa.edit,
            generics: model_generics.clone(),
            vars:               vars.clone(),

            script: ( script_def, script_mets, script_trts ),
            live:   (   live_def,   live_mets,   live_trts ),
    };

    

    let ImplVars{ field,..} = impl_vars;

    if let Some(field)  = field {
        model_sdpl.insert(field.clone(), sdpl);
    } else {
        let Vars{self_,..} = vars;
        model_sdpl.insert(self_.clone(), sdpl);
    }

    // error 


    model_sdpl

}


/*
1) Sort the example so that it can take the input 
and do the right imports on main file .
2) -  Add fields in GG .
3) Add a wrapper function in methods generating code parts.
4) Add iter_vars in methods generating code parts.
5) Sort interact for Group macro.

----
6) Look for ways to integrate legend.


*/