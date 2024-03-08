use crate::error;
use crate::model::{
    self,get_ident_type_generics,ModelSdpl,AttributeArguments,
    MpscChannel,Cont,Vars,method,generic,
    ActorAttributeArguments,Lib,Model};
use super::{ActorMethodNew,ActorMethod};

use proc_macro_error::abort;
use syn::{
    Ident,Type,Signature,WhereClause,Generics,
    ItemImpl,Visibility, ImplGenerics, TypeGenerics };
use quote::{quote,format_ident};
use proc_macro2::TokenStream;
use proc_macro::Span;
use std::collections::BTreeMap;

fn group_direct_add_fields(cont: &mut Cont, vars: &Vars, model_sdpl: &ModelSdpl ){

    // ADD FIELDS MEMBERS
    let Cont{ direct_arms, 
            script_fields,
               debug_arms,..} = cont;

    let Vars{script_name,.. } = vars;
    
    for (field,ams) in model_sdpl.fields.iter(){

        let variant_name = crate::model::name::script_field(field);
        let field_struct_variant_name = &ams.vars.script_name;
        let mem_model_generics = &ams.vars.impl_vars.model_generics;
        let await_call = ams.asyncness.as_ref().map(|_|quote!{.await});
        
        // Direct Arm
        let arm_match = quote! { 
            #variant_name ( msg )
        };
        let direct_arm = quote! {
            #script_name :: #arm_match => { msg.direct( &mut group. #field )#await_call;} 
        };
        direct_arms.push(direct_arm);


        // Script Struct
        let script_field = quote! { 
            #variant_name ( #field_struct_variant_name #mem_model_generics )
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
    pub group_model_generics: Option<Generics>,
    pub vis:         Option<Visibility>,
    pub field:            Option<Ident>,
    pub ty:                Option<Type>,
    pub def_gen:       Option<Generics>,
    pub group_script_type: Option<Type>,
    pub group_script_name: Option<Ident>,


}

impl ImplVars {


    pub fn get_mut_sigs(&mut self) -> Vec<&mut Signature>{

        let mut sigs = self.actor_methods.iter_mut().map(|m| m.get_mut_sig()).collect::<Vec<_>>();
        if let Some(met_new_sig) = self.met_new.as_mut().map(|x| x.get_mut_sig()){
            sigs.push(met_new_sig);
        }
        sigs
    }

    pub fn get_split_model_generics(&self) 
        -> ((ImplGenerics,TypeGenerics,Option<&WhereClause>),
            (ImplGenerics,TypeGenerics,Option<&WhereClause>)){

        if let Some(gmg) = &self.group_model_generics{
            (self.model_generics.split_for_impl(), gmg.split_for_impl())
        } else {
            (self.model_generics.split_for_impl(), self.model_generics.split_for_impl())
        }
    }

    pub fn get_group_script_wrapper(&self) -> Box<dyn Fn(TokenStream) -> TokenStream> {
    
        if let Some(gs_name)  = self.group_script_name.clone(){
            if let Some(field) = &self.field{
                let field_var = crate::model::name::script_field(field);
                return Box::new( move |ts:TokenStream| 
                    quote::quote!{ 
                        #gs_name :: #field_var ( #ts )
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
         method::get_methods( &actor_type,item_impl.clone(),&aaa ,mac.eq(&model));


    let mut model_generics = generics.clone();

    let mut sigs = actor_methods.iter_mut().map(|m| m.get_mut_sig()).collect::<Vec<_>>();
    if let Some(met_new_sig) = met_new.as_mut().map(|x| x.get_mut_sig()){
        sigs.push(met_new_sig);
    }
             
    generic::take_generic_parts( &mut model_generics, sigs,def_gen);

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
        group_model_generics: None,
        vis:                  None,
        field:                None,
        ty:                   None,
        def_gen:              None,
        group_script_type:    None,
        group_script_name:    None,
    }
}

pub fn generate_model( aa: AttributeArguments, item_impl: &ItemImpl , impl_vars: Option<ImplVars>) 
    -> ModelSdpl {

    // mac model values 
    let mut cont = model::Cont::new();
    let mut model_sdpl = ModelSdpl::new();
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

                    impl_vars.vis     = Some(vis);
                    impl_vars.field   = Some(key.clone());
                    impl_vars.ty      = Some(ty.clone());

                    let aa = AttributeArguments::Actor(aaa);
                
                    coll_impl_vars.insert(key,(aa,item_impl,impl_vars));
                }
 
                aaa = gaas.get_aaa(None);
                let mut impl_vars = get_impl_vars(&item_impl, &aaa, Some(gaas.def_generics.clone()), mac, model);

                crate::model::generic::group_generics(&mut impl_vars,&mut coll_impl_vars );

                let ImplVars{ actor_name,.. } = &impl_vars;
                let cust_name    = &if aaa.name.is_some(){ aaa.name.clone().unwrap() } else { actor_name.clone() };
                let group_script = &crate::model::name::group_script(cust_name);
                let (_,( _, ty_generics, _ )) = impl_vars.get_split_model_generics();

                // create the type of groupt script enum
                let group_script_type: Type = syn::parse_quote!{ #group_script #ty_generics };

                let field_names = aaa.get_inter_field_names();

                for (key,(aa,item_impl, mut impl_vars)) in coll_impl_vars {
                    
                    if field_names.contains(key){
                        let msg = error::var_name_conflict(key,"field");
                        abort!(Span::call_site(),msg);
                    }
                    impl_vars.group_script_type = Some(group_script_type.clone());
                    impl_vars.group_script_name = Some(group_script.clone());

                    let btm_sdpl = 
                    generate_model(aa,&item_impl,Some(impl_vars.clone()));
                    model_sdpl.extend(btm_sdpl);
                }

                impl_vars
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
        actor_name, met_new, .. } = &impl_vars;
    
    
    let mut new_vis = vis.as_ref().map(|x| x.clone());

    let (( _s_impl_generics,
             s_ty_generics,
            s_where_clause ),
         ( _l_impl_generics,
             l_ty_generics,
            l_where_clause )) = impl_vars.get_split_model_generics();

    // generate raw parts of model   
    method::to_raw_parts( vars,&mut cont,&aaa,oneshot,mpsc );
    

    // condition if is Group ONLY!!!
    if mac.eq(&model) && Model::Group.eq(&mac) {
        group_direct_add_fields(&mut cont,vars,&model_sdpl);
    }
        
    if mac.eq(&model) { 

        if met_new.is_none() {

            let msg = format!("Can not find public/restricted  method `new` or `try_new` for {:?} object.",actor_name.to_string());
            let (note,help) = error::met_new_note_help(&actor_name);
            abort!(item_impl,msg;note=note;help=help);
        }

        
        // Change visibility for `model methods` 
        new_vis = met_new.as_ref().map(|m| m.vis.clone());

        let met_new         = met_new.clone().unwrap();
        let new_sig             = &met_new.new_sig;
        let func_new_name           = &new_sig.ident;
        let (args_ident, _ )   = method::arguments_pat_type(&met_new.get_arguments());
        let unwrapped          = met_new.unwrap_sign();
        let doc_attrs      = &met_new.doc_attrs;
        let vis                = &met_new.vis.clone();
        let group_fields_init = model_sdpl.get_fields_init();
        let (init_live, play_args) = {
            if aaa.debut.active() {
                (quote!{ Self { #group_fields_init #debut: std::sync::Arc::clone(&#debut), #name : format!("{:?}",* #debut),#sender  }} ,
                    quote!{ #receiver, #actor, #debut_play})
            } else {

                (quote!{ Self{ #group_fields_init #sender  } }, 
                    quote!{ #receiver, #actor } )
            }
        };

        let spawn = aaa.lib.method_new_spawn(&play_args,script_name);
        let turbofish = s_ty_generics.as_turbofish();

        let vars_debut = 
        if aaa.debut.active() {
            quote!{let #debut =  #script_name #turbofish ::#debut();
                   let #debut_play = *std::sync::Arc::clone(&#debut); }
        } else {quote!{}};

        let return_statement   = met_new.live_ret_statement(&init_live);
        
        let MpscChannel{declaration, ..} = mpsc;
        let Cont{live_mets,..} = &mut cont;
        
        let func_new_body = quote!{

            #(#doc_attrs)*
            #vis #new_sig {
                let #actor = #actor_name:: #func_new_name #args_ident #unwrapped;
                #declaration
                #vars_debut
                #spawn
                #return_statement
            }
        };

        live_mets.insert(0,(new_sig.ident.clone(),func_new_body));


        // LIVE INTER METHODS AND TRAITS
        if aaa.debut.active(){
            aaa.debut.impl_debut( &mut cont, vars, &new_vis, &l_ty_generics, &l_where_clause);

            if aaa.debut.is_legend(){
                aaa.debut.impl_legend( 
                    &mut cont, vars, &new_vis,mpsc, 
                    model_sdpl.fields.keys().collect::<Vec<_>>(), 
                    &l_ty_generics, &l_where_clause,
                    &spawn
                );
            }
        }
    };


    // SCRIPT DEFINITION
    let script_def = {
        let Cont{ script_fields,..} = &mut cont;
        quote! {
            #new_vis enum #script_name #s_ty_generics #s_where_clause {
                #(#script_fields),*
            }
        }
    };        


    // DIRECT
    {
        let Cont{script_mets,direct_arms,..} = &mut cont;
        script_mets.push((direct.clone(),
        quote!{
            #new_vis #async_decl fn #direct (self, #actor: &mut #actor_type ) {
                match self {
                    #(#direct_arms)*
                }
            }
        }));
    }


    // PLAY
    if mac.eq(&model) {

        let await_call  = async_decl.as_ref().map(|_| quote!{.await});
        let end_of_play = error::end_of_life( &actor_name, &aaa.debut.clone() );  
        
        let debut_pat_type = if aaa.debut.active(){quote!{,#debut: std::time::SystemTime }} else { quote!{} };

        let MpscChannel{pat_type_receiver,..}      = mpsc;
        let Cont{script_mets,..} = &mut cont;
        let Vars{ actor_legend,..} = &vars;

        let legend_call = 
        if aaa.debut.is_legend(){
            quote!{ let _ = #script_name :: #actor_legend ( #debut, std::option::Option::Some( #actor )); }
        } else { quote!{} };

        let play_method = {
        
            let ok_or_some = match aaa.lib {
                Lib::Tokio => quote!{std::option::Option::Some},
                _ => quote!{std::result::Result::Ok}, 
            };
            quote! {
                #new_vis #async_decl fn #play ( #pat_type_receiver mut #actor: #actor_type #debut_pat_type ) {
                    while let #ok_or_some (#msg) = #receiver.recv() #await_call {
                        #msg.#direct ( &mut #actor ) #await_call;
                    }
                    #legend_call
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
            impl #s_ty_generics std::fmt::Debug for #script_name #s_ty_generics #s_where_clause {
            
                fn fmt( &self, f: &mut std::fmt::Formatter<'_> ) -> std::fmt::Result {
                    #body
                }
            }
        }));
    }


    // LIVE DEFINITION
    let live_def = {

    let MpscChannel{pat_type_sender,..} = &mpsc;
    let group_pat_type_fields = model_sdpl.get_pat_type_fields();
        if mac.eq(&model) {
            let (debut_field, name_field) = if aaa.debut.active() {
                ( quote!{ pub #debut: std::sync::Arc<std::time::SystemTime>,},
                  quote!{ pub #name: String,} )
            } else { (quote!{}, quote!{})};   

            quote!{
                #[derive(Clone)]
                #new_vis struct #live_name #l_ty_generics #l_where_clause {
                    #pat_type_sender
                    #debut_field
                    #name_field
                    #group_pat_type_fields
                }
            }
        } else { 

            quote!{
                #[derive(Clone)]
                #new_vis struct #live_name #l_ty_generics #l_where_clause {
                    #pat_type_sender
                }
            }
        }
    };

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

    model_sdpl

}
