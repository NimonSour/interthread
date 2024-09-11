
use std::collections::BTreeMap;

use syn::{
    Generics,WherePredicate,GenericParam,
    Type,ItemImpl,TypePath,Ident,Signature,
    Token,TypeParamBound,punctuated::Punctuated };
use proc_macro::Span;
use proc_macro_error::abort;
use quote::quote;

use crate::model::{method,gen_temp_inter,gen_add_field,AttributeArguments,ImplVars};

pub fn get_type_idents( g: &Generics ) -> Vec<Ident> {
    g.params.iter().filter_map(|x| {
        match x {
            GenericParam::Type(type_param) => {
                Some(type_param.ident.clone())
            },
            _ => None,
        }
    }).collect::<Vec<_>>()
}

pub fn contains<V>(a: V, b: V, contains: bool) -> Vec<V::Item>
where
    V: IntoIterator + Clone,
    V::Item: Eq + Clone,
{
    let a_clone = a.clone().into_iter().collect::<Vec<_>>();
   
    b.clone()
    .into_iter()
    .filter(|bb| if contains { a_clone.contains(bb)} else { !a_clone.contains(bb) })
    .collect()
}



fn get_where_pred_bounds( pred: WherePredicate ) -> Option<(Type, Punctuated<TypeParamBound,Token![+]>)>{
    match pred {
        syn::WherePredicate::Type(pred_type) => {
            return Some( (pred_type.bounded_ty.clone(), pred_type.bounds.clone()));
        },
        _ => return None,
    }
}

fn model_bounds(is_legend: bool) -> Punctuated<TypeParamBound, Token![+]>{
    let  clone = if is_legend { Some( quote!{ Clone + } )} else {None};
    let where_clause = match syn::parse2::<syn::WhereClause>(quote!{
        where T : #clone  Send + Sync + 'static,
    }) {
        Ok( v ) => v,
        Err(e)        => {
            let msg = format!("Internal Error.'generics::generate_where_clause'. {}!",e);
            abort!(proc_macro2::Span::call_site(),msg );
        },
    };

    let pred = where_clause.predicates.first().unwrap().clone();
    let (_, bounds) = get_where_pred_bounds(pred).unwrap();
    bounds
}

fn include_set<T,P>(this: &mut Punctuated<T,P>, other: &Punctuated<T,P> )
    where 
    T: PartialEq + Clone,
    P: Default,
    {
    for typ in other{
        if !this.iter().any(|t| t.eq(typ)){
            this.push(typ.clone());
        }
    }
} 

fn to_type( ident: Ident) -> Type {
    Type::Path(TypePath { qself: None, path: ident.into() })
}

fn take_gen_param_ident_bounds( gen: &mut Generics ) -> Option<Vec<(Type,Punctuated<TypeParamBound, Token![+]>)>>{
    
    let mut coll = Vec::new();
    
    if gen.params.is_empty() && gen.where_clause.is_none(){ 
        return None; 
    } else {

        let gen_params = gen.params.clone();
        gen.params.clear();

        for gen_param in gen_params {

            match &gen_param {
                syn::GenericParam::Type(type_param) => {
                    coll.push((to_type(type_param.ident.clone()),type_param.bounds.clone()));

                },
                _ => { gen.params.push(gen_param) },
            }
        }

        if  let Some(predicates) = gen.where_clause.as_ref().map(|w| w.predicates.clone()){
            gen.where_clause.as_mut().map(|w| w.predicates.clear());

            for pred in predicates { 
                if let Some((ident, bounds)) = get_where_pred_bounds(pred.clone()){

                    if let Some(pos) = coll.iter().position(|x| x.0.eq(&ident)){
                        include_set( &mut coll[pos].1, &bounds);
                        continue;
                    } else {
                        coll.push((ident,bounds));
                    }
                    // the types which are declared at a block level
                    // are pusheed back to WhereClause
                    // ? are there other cases ?
                } 
            }
        }
        return Some(coll);
    }
}

pub fn take_generics_from_sig( sigs: Vec<&mut Signature>) -> Vec<(Type,Punctuated<TypeParamBound, Token![+]>)> {

    let mut coll : Vec<(Type,Punctuated<_,_>)> = Vec::new();
    for sig in sigs {
        if let Some( params) = take_gen_param_ident_bounds(&mut sig.generics){           
            push_include( &mut coll,params );
        }
    }
    return coll
}

fn push_include( this: &mut Vec<(Type,Punctuated<TypeParamBound, Token![+]>)> , 
                other: Vec<(Type,Punctuated<TypeParamBound, Token![+]>)> ) {

    for (ty,bounds) in other {
        if let Some(pos) = this.iter().position(|x| x.0.eq(&ty)){
            include_set( &mut this[pos].1, &bounds);
        } else {
            this.push((ty,bounds))
        }
    }
}

pub fn include_bounds(gen: &mut Generics, 
             other_bounds: Vec<(Type,Punctuated<TypeParamBound, Token![+]>)>,
                is_legend: bool ) {
    let this_bounds =  
    if let Some(mut bounds) = take_gen_param_ident_bounds(gen){
        push_include( &mut bounds, other_bounds);
        bounds
    } else {  other_bounds };


    let mut where_clause = match &gen.where_clause {
        Some(w) => w.clone(),
        None => {
            syn::WhereClause {
                where_token: <syn::Token![where]>::default(),
                predicates: syn::punctuated::Punctuated::new(),
            }
        }
    };

    let model_bounds = model_bounds(is_legend);
    for (ty, mut bounds) in this_bounds {
        include_set(&mut bounds,&model_bounds);
        where_clause.predicates.push(syn::parse_quote! {
            #ty: #bounds
        });
        // if is ident push into parameters
        if let Some(ident) = type_path_some_ident(&ty){
            gen.params.push(syn::parse_quote! {#ident} );
        }
    }
    gen.where_clause = Some(where_clause);

}



pub fn take_generic_parts( gen: &mut Generics, 
                       methods: Vec<&mut Signature>, 
                       def_gen: Option<Generics>,
                     is_legend: bool )
{

    let methods_bounds = take_generics_from_sig(methods);
    include_bounds(gen, methods_bounds,is_legend);

    if let Some(mut def_gen) = def_gen {
        if let Some(strct_def_bounds) =  take_gen_param_ident_bounds(&mut def_gen){
            include_bounds(gen, strct_def_bounds,is_legend); 
        }
    }
}

pub fn type_path_some_ident( ty: &Type) -> Option<Ident>{
    match ty {
        Type::Path(ty_path) => {
            if let Some(p) = ty_path.path.get_ident(){
                Some(p.clone())
            } else {None}
        },
        _ => None,
    }
}

pub fn idents_from_ty(ty: &Type)-> Option<Vec<Ident>>{

    match ty {
        syn::Type::Path(type_path) => {
            if let Some(path_seg) = type_path.path.segments.last(){

                match &path_seg.arguments {
                    syn::PathArguments::AngleBracketed(ang_brack) => {
                        let coll = ang_brack.args.iter().filter_map(|x|
                            match x {
                                syn::GenericArgument::Type(ty) => {
                                    type_path_some_ident(ty)
                                },
                                _ => None,
                            }
                        ).collect::<Vec<_>>();
                        if coll.is_empty() { None } else { Some(coll)}
                    },
                    _ => None,
                }
            } else { None }
        },
        _ => None,
    }
}

pub fn gen_rename( gen: &Generics, ident_old: &Ident, ident_new: &Ident) -> Generics {
    let mut item_impl = {
        if let Ok(ty) = syn::parse_str::<syn::ItemImpl>("impl Inter {}") {
            ty
        } else {
            let msg = format!("Internal Error. 'generic::gen_rename'. Could not parse &str to ItemImpl!");
            abort!(Span::call_site(), msg);
        }
    };
    item_impl.generics = gen.clone();
    let new_item_impl = crate::model::method::replace(&item_impl,ident_old,ident_new);
    new_item_impl.generics.clone()
}

pub fn sig_rename( sig: &Signature, gen_old_new: &BTreeMap<String,BTreeMap<Ident,Ident>> ) -> Signature {


    let mut new_sig = sig.clone();
    if let Some(add) =  gen_old_new.get("add"){

        for (ident_old, ident_new) in add {
            new_sig = method::replace(&new_sig,ident_old,ident_new);
        }
    }

    if let Some(org) =  gen_old_new.get("org"){
        for (ident_old, _ ) in org {
            let ident_temp = &gen_temp_inter(ident_old);
            new_sig = method::replace(&new_sig,ident_old,ident_temp);
        }
        for (ident_old, ident_new ) in org { 
            let ident_temp = &gen_temp_inter(ident_old);
            new_sig = method::replace(&new_sig,ident_temp,ident_new);
        }
    }
    new_sig
}

pub fn field_gen_rename( impl_vars: &mut ImplVars ) {
    
    let old = &impl_vars.generics; 
    let new = &impl_vars.model_generics; 

    let org_gen_idents = get_type_idents(&old);
    let add_gen_idents = get_type_idents(&new);

    let intersect  = contains(&org_gen_idents,&add_gen_idents,true);
    let difference = contains(&org_gen_idents,&add_gen_idents,false);

    // keep track of ol new names 
    let mut gen_old_new = 
    BTreeMap::from([(String::from("org"),BTreeMap::new()),(String::from("add"),BTreeMap::new())]);
    let mut new_model_gen = new.clone();
    

    if let Some(field)  = impl_vars.field.clone(){
        let add = gen_old_new.get_mut("add").unwrap();

        for ident in difference{
            let ident_new = &gen_add_field(&field,ident);
            new_model_gen = gen_rename(&new_model_gen,ident,&ident_new);
            add.insert( ident.clone(),ident_new.clone());
        }

    } else {
        let msg = format!("Internal Error. 'generic::gen_rename'. Expected a some `field` in ImplVars!");
        abort!(Span::call_site(), msg);
    }

    let org = gen_old_new.get_mut("org").unwrap();

    for ident in intersect {

        let ident_temp = &gen_temp_inter(ident);
        new_model_gen = gen_rename(&new_model_gen,ident,ident_temp);
        org.insert(ident.clone(),ident_temp.clone());
    }

    // from type get new model type names
    if let Some(group_idents) = idents_from_ty(&impl_vars.actor_type){

        for (index,ident_new) in group_idents.iter().enumerate(){
            
            if let Some(ident_temp) = org.get_mut(&org_gen_idents[index]){

                new_model_gen = gen_rename(&new_model_gen,ident_temp,&ident_new);
                *ident_temp = ident_new.clone();

            }
        }
    }

    // new_model_generics
    impl_vars.model_generics = new_model_gen.clone();

    // rename sigs
    for sig in impl_vars.get_mut_sigs(){
        *sig = sig_rename(sig,&gen_old_new);
    }

}


pub fn group_generics( 
     slf: &mut ImplVars,
    mems: &mut BTreeMap<&Ident,(AttributeArguments,ItemImpl,ImplVars)> ,
    is_legend: bool )
{   


    let mut slf_gen_model = slf.model_generics.clone();
    let mut total_generics = Vec::new();

    for (_, (_,_,impl_vars) ) in  mems.iter_mut(){
        
        // change actor type to the one found in group definition
        if let Some( mem_ty ) = impl_vars.ty.clone(){
            impl_vars.actor_type = mem_ty;
        }
        // rename generics to names used by user and convetional for 'add's
        // rename method sigs
        field_gen_rename(impl_vars);
        total_generics.push(impl_vars.model_generics.clone())

    }

    for gen in total_generics.iter_mut() {

        if let Some(mem_gen_bounds) =  
            take_gen_param_ident_bounds(gen){
            include_bounds(&mut slf_gen_model, mem_gen_bounds,is_legend);  
        }
    }

    slf.group_model_generics = None; 
    slf.model_generics = slf_gen_model.clone();
   
    for (_, (_,_,impl_vars) ) in  mems.iter_mut(){

        // give group_model_generics value for group members
        impl_vars.group_model_generics = Some(slf_gen_model.clone());
    }


}

pub fn is_generic(generics: &Generics) -> bool {
    if generics.params.is_empty(){
        if let Some(where_clause) = &generics.where_clause{
            if where_clause.predicates.is_empty(){
                false
            } else { true }
        } else { false }
    } else { true }
}

