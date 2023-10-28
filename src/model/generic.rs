
use std::collections::{BTreeMap,HashMap};

use syn::{Generics,WherePredicate,GenericParam,Type,TypePath,PathArguments,PathSegment,Ident,Signature,Token,TypeParamBound,punctuated::Punctuated};
use proc_macro::Span;
use proc_macro_error::abort;
use quote::quote;


/*

Find the generics of field types 

the model generics contains:
1) Impl generics
2) Impl method sig generics
3) Struct def generics.

we need to find the difference 
between original generics and the 
additional bounds.



*/


pub fn add_constraints( this: &mut Generics, other: &Generics ){

}


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

// pub fn get_diff_idents( old: &Generics, new: &Generics ) -> Vec<Ident> { 
//     let new_idents = get_type_idents(&new);
//     let old_idents = get_type_idents(&old);
// }


pub fn contains<V>(a: V, b: V, contains: bool) -> Vec<V::Item>
where
    V: IntoIterator + Clone,
    V::Item: Eq + Clone,
{
    let b_clone = b.clone().into_iter().collect::<Vec<_>>();
   
    a.clone()
    .into_iter()
    .filter(|aa| contains && b_clone.contains(aa))
    .collect()
}


pub fn exam_rename( old: &Generics, new: &Generics ){
    let new_idents = get_type_idents(&new);
    let old_idents = get_type_idents(&old);

    let idents = contains(&old_idents, &new_idents,true);
    let mut my_gen = new.clone();

    // create an impl block 
    let mut item_impl = {
        if let Ok(ty) = syn::parse_str::<syn::ItemImpl>("impl InterFoo {}") {
            ty
        } else {
            
            let msg = format!("Internal Error. 'method::replace'. Could not parse &str to ItemImpl!");
            abort!(Span::call_site(), msg);
        }
    };

    item_impl.generics = new.clone();
    
    for i in &idents{

        let new_i = quote::format_ident!("{}my",i.to_string());

        if let Some(new_wc) = new.where_clause.clone(){
            let wc = crate::model::method::replace(&new_wc,&i,&new_i);
            my_gen.where_clause = Some(wc);
        }

        item_impl = crate::model::method::replace(&item_impl,i,&new_i);

    }


    let my_where = item_impl.generics.where_clause.clone();
    let init_where = new.where_clause.clone();
    let my = quote!{ #my_where}.to_string();
    let nw = quote!{ #init_where}.to_string();
    let msg = format!("Initial - {} where {} , renamed - {} where {} ", quote!{#new}.to_string(),nw, quote!{#my_gen}.to_string(),my);
   
   
    proc_macro_error::abort!( proc_macro::Span::call_site(),msg);
}



fn get_where_pred_bounds( pred: WherePredicate ) -> Option<(Type, Punctuated<TypeParamBound,Token![+]>)>{
    match pred {
        syn::WherePredicate::Type(pred_type) => {

            return Some( (pred_type.bounded_ty.clone(), pred_type.bounds.clone()));
            // match &pred_type.bounded_ty {
            //     syn::Type::Path(type_path) => {
            //         if let Some(ident) = type_path.path.get_ident(){
            //             return Some((ident.clone(),pred_type.bounds.clone()));
            //         }
            //         return None;  
            //     },
            //     _ => return None,   
            // };
        },
        _ => return None,
    }
}

fn model_bounds() -> Punctuated<TypeParamBound, Token![+]>{
    let where_clause = match syn::parse2::<syn::WhereClause>(quote!{
        where T : Send + Sync + 'static,
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


/*
    pub struct Generics {
        pub lt_token: Option<Token![<]>,
        pub params: Punctuated<GenericParam, Token![,]>,
        pub gt_token: Option<Token![>]>,
        pub where_clause: Option<WhereClause>,
        }

    pub enum GenericParam {
        /// A lifetime parameter: `'a: 'b + 'c + 'd`.
        Lifetime(LifetimeParam),

        /// A generic type parameter: `T: Into<String>`.
        Type(TypeParam),

        /// A const generic parameter: `const LENGTH: usize`.
        Const(ConstParam),
        }

    pub struct TypeParam {
        pub attrs: Vec<Attribute>,
        pub ident: Ident,
        pub colon_token: Option<Token![:]>,
        pub bounds: Punctuated<TypeParamBound, Token![+]>,
        pub eq_token: Option<Token![=]>,
        pub default: Option<Type>,
        }


*/
fn take_gen_param_ident_bounds( gen: &mut Generics ) -> Option<Vec<(Type,Punctuated<TypeParamBound, Token![+]>)>>{//-> Option<Vec<(Ident,Punctuated<TypeParamBound, Token![+]>)>>{
    
    let mut coll = Vec::new();

    if gen.params.is_empty(){ 
        return None; 
    } else {

        // let gen_params = gen.params.clone();
        // gen.params.clear();

        for gen_param in gen.params.iter_mut() {

            match &gen_param {
                syn::GenericParam::Type(type_param) => {
                    let bounds= type_param.bounds.clone();
                    type_param.bounds.clear();

                    // coll.push(to_type((type_param.ident.clone()),type_param.bounds.clone()));
                    coll.push((to_type(type_param.ident.clone()),bounds));
                },
                _ => (),//{ gen.params.push(gen_param) },
            }
        }

        if coll.is_empty() {

            return None;

        } else { 

            if  let Some(predicates) = gen.where_clause.as_ref().map(|w| w.predicates.clone()){
                gen.where_clause.as_mut().map(|w| w.predicates.clear());

                for pred in predicates { 
                    if let Some((ty_path, bounds)) = &get_where_pred_bounds(pred.clone()){

                        if let Some(pos) = coll.iter().position(|x| x.0.eq(ty_path)){
                            include_set( &mut coll[pos].1, bounds);
                            continue;
                        }
                        // if let Some(t) = coll.get_mut(&ty_path){
                        //     include_set( t, &bounds);
                        //     continue;
                        // }
                    } 
                    gen.where_clause.as_mut().map(|w| w.predicates.push(pred));
                }
            }
            return Some(coll);
        }
    }
}


pub fn take_generics_from_sig( sigs: Vec<&mut Signature>) -> HashMap<Type,Punctuated<TypeParamBound, Token![+]>>{//Vec<(Ident,Punctuated<TypeParamBound, Token![+]>)> {

    let mut coll : Vec<(Type,Punctuated<_,_>)> = HashMap::new();

    for sig in sigs {

        if let Some( params) = 
            take_gen_param_ident_bounds(&mut sig.generics){           
            push_include( &mut coll,params );
        }
    }
    return coll
}


fn push_include( this: &mut HashMap<Type,Punctuated<TypeParamBound, Token![+]>>,// &mut Vec<(Ident,Punctuated<TypeParamBound, Token![+]>)> , 
                other: HashMap<Type,Punctuated<TypeParamBound, Token![+]>>){//Vec<(Ident,Punctuated<TypeParamBound, Token![+]>)> ) {

    for (ident,bounds) in other {

        if let Some(mut t) = this.get_mut(&ident){
            include_set( &mut t, &bounds);
        } else {
            this.insert(ident,bounds);
        }
    }
}


// pub fn get_parts(gen: &mut Generics, methods: Vec<&mut Signature>) {//-> (Option<ImplGenerics<'a>>,Option<TypeGenerics<'a>>,Option<WhereClause>) {
pub fn include_bounds(gen: &mut Generics, 
             mut other_bounds: HashMap<Type,Punctuated<TypeParamBound, Token![+]>>) {//-> (Option<ImplGenerics<'a>>,Option<TypeGenerics<'a>>,Option<WhereClause>) {
     
    // let mut other_bounds = take_generics_from_sig(methods);
    if let Some(actor_bounds) = take_gen_param_ident_bounds(gen){
        push_include( &mut other_bounds, actor_bounds);
    } 

    // push_include( &mut other_bounds, actor_bounds);
    if !other_bounds.is_empty(){
        let mut where_clause = match &gen.where_clause {
            Some(w) => w.clone(),
            None => {
                syn::WhereClause {
                    where_token: <syn::Token![where]>::default(),
                    predicates: syn::punctuated::Punctuated::new(),
                }
            }
        };
        let model_bounds = model_bounds();
        for (ident, mut bounds) in other_bounds {
            include_set(&mut bounds,&model_bounds);
            where_clause.predicates.push(syn::parse_quote! {
                #ident: #bounds
            });
            gen.params.push(syn::parse_quote! {#ident} );
        }
        gen.where_clause = Some(where_clause);
    } 
}



pub fn take_generic_parts( gen: &mut Generics, 
                       methods: Vec<&mut Signature>, 
                     def_gen: Option<Generics> )
{
    let methods_bounds = take_generics_from_sig(methods);
    include_bounds(gen, methods_bounds);

    if let Some(mut def_gen) = def_gen {
        if let Some(strct_def_bounds) =  take_gen_param_ident_bounds(&mut def_gen){
            include_bounds(gen, strct_def_bounds); 
        }
    }
}


