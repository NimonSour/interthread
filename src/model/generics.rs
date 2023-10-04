
use syn::{Generics,WherePredicate,Ident,Signature,Token,TypeParamBound,punctuated::Punctuated};
use proc_macro_error::abort;
use quote::quote;

fn get_where_pred_bounds( pred: WherePredicate ) -> Option<(Ident, Punctuated<TypeParamBound,Token![+]>)>{
    match pred {
        syn::WherePredicate::Type(pred_type) => {
            match &pred_type.bounded_ty {
                syn::Type::Path(type_path) => {
                    if let Some(ident) = type_path.path.get_ident(){
                        return Some((ident.clone(),pred_type.bounds.clone()));
                    }
                    return None;  
                },
                _ => return None,   
            };
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


fn take_gen_param_ident_bounds( gen: &mut Generics ) -> Option<Vec<(Ident,Punctuated<TypeParamBound, Token![+]>)>>{
    
    let mut coll = Vec::new();

    if gen.params.is_empty(){ 
        return None; 
    } else {

        let gen_params = gen.params.clone();
        gen.params.clear();

        for gen_param in gen_params {

            match &gen_param {
                syn::GenericParam::Type(type_param) => {
                    coll.push((type_param.ident.clone(),type_param.bounds.clone()));
                },
                _ => { gen.params.push(gen_param) },
            }
        }

        if coll.is_empty() {

            return None;

        } else { 

            if  let Some(predicates) = gen.where_clause.as_ref().map(|w| w.predicates.clone()){
                gen.where_clause.as_mut().map(|w| w.predicates.clear());

                for pred in predicates { 
                    if let Some((ident, bounds)) = get_where_pred_bounds(pred.clone()){

                        if let Some(pos) = coll.iter().position(|x| x.0.eq(&ident)){
                            include_set( &mut coll[pos].1, &bounds);
                            continue;
                        } 
                        // the types which are declared at a block level
                        // are pusheed back to WhereClause
                        // ? are there other cases ?
                    } 
                    gen.where_clause.as_mut().map(|w| w.predicates.push(pred));
                }
            }
            return Some(coll);
        }
    }
}


pub fn take_generics( sigs: Vec<&mut Signature>) -> Vec<(Ident,Punctuated<TypeParamBound, Token![+]>)> {

    let mut coll : Vec<(Ident,Punctuated<_,_>)> = Vec::new();

    for sig in sigs {

        if let Some( params) = take_gen_param_ident_bounds(&mut sig.generics){           
            push_include( &mut coll,params );
        }
    }
    return coll
}


fn push_include( this: &mut Vec<(Ident,Punctuated<TypeParamBound, Token![+]>)> , 
                other: Vec<(Ident,Punctuated<TypeParamBound, Token![+]>)> ) {

    for (ident,bounds) in other {
        if let Some(pos) = this.iter().position(|x| x.0.eq(&ident)){
            include_set( &mut this[pos].1, &bounds);
        } else {
            this.push((ident,bounds))
        }
    }
}


pub fn get_parts(gen: &mut Generics, methods: Vec<&mut Signature>) {//-> (Option<ImplGenerics<'a>>,Option<TypeGenerics<'a>>,Option<WhereClause>) {
    
    // we don't need this condition 
    // if returning generics from all syn::Item s

    // if let Some(gen) = generics {
        
        let mut meth_bounds = take_generics(methods);

        if let Some(actor_bounds) = take_gen_param_ident_bounds(gen){
            push_include( &mut meth_bounds, actor_bounds);
        } 

        if !meth_bounds.is_empty(){

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
            for (ident, mut bounds) in meth_bounds {
                include_set(&mut bounds,&model_bounds);

                where_clause.predicates.push(syn::parse_quote! {
                    #ident: #bounds
                });
                gen.params.push(syn::parse_quote! {#ident} );
            }

            gen.where_clause = Some(where_clause);
        } 

        // gen
        // let (impl_generics, ty_generics, mut where_clause) = gen.split_for_impl();
        
        // return (Some(impl_generics), Some(ty_generics), where_clause.as_mut().map(|x| x.clone()));
        
        
    // } else {
    //     return (None,None,None);
    // }
}

