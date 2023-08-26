
use syn::{Generics,WherePredicate,Ident,Token,WhereClause,ImplGenerics,TypeGenerics, TypeParamBound,punctuated::Punctuated};
use proc_macro_error::abort;
use quote::quote;


fn get_pred_bounds( pred: &mut WherePredicate ) -> Option<(&Ident, &mut Punctuated<TypeParamBound,Token![+]>)>{
    match pred {
        syn::WherePredicate::Type(pred_type) => {
            let ident = match &pred_type.bounded_ty {
                syn::Type::Path(type_path) => {
                    match type_path.path.get_ident(){
                       Some(idnt) => idnt,
                       None => abort!(type_path,"Internal Error.'generics::get_pred_bounds'.The path is not an ident !"),  
                    }
                },
                _ => { abort!(proc_macro2::Span::call_site(), "Internal Error.'generics::get_pred_bounds'. Expected a path !"); },   
            };
            let bounds = &mut pred_type.bounds;
            Some((ident,bounds))
        },
        _ => None,
    }
}

fn generate_where_clause(ident_list: &Vec<Ident>) -> WhereClause {
    match syn::parse2::<syn::WhereClause>(quote!{
        where #( #ident_list : Send + Sync + 'static ),*
    }) {
        Ok( v ) => v,
        Err(e)        => {
            let msg = format!("Internal Error.'generics::generate_where_clause'. {}!",e);
            abort!(proc_macro2::Span::call_site(),msg );
        },
    }
}

fn  get_ty_param_idents( gen: &Generics ) -> Vec<Ident> {

    gen.params.iter().filter_map(|x| 
        match x {
            syn::GenericParam::Type(type_param) => {
                Some(type_param.ident.clone())
            },
            _ => None,
        }
    ).collect::<Vec<_>>()

}


pub fn get_parts(generics: &Option<Generics>) -> (Option<ImplGenerics>,Option<TypeGenerics>,Option<WhereClause>) {

    if let Some(gen) = generics{

        let (impl_generics, ty_generics, mut where_clause) = gen.split_for_impl();
        
        if !gen.params.is_empty(){
    
            let idents = get_ty_param_idents(gen);
    
            if !idents.is_empty() {
    
                let mut inter_where_clause = generate_where_clause(&idents);
            
                let mut live_pred = 
                inter_where_clause.predicates.iter_mut().filter_map(|x| get_pred_bounds(x)).collect::<Vec<_>>();
            
                if let Some(actor_where_clause) = where_clause {
                    let mut new_where_clause = actor_where_clause.clone();
            
                    for pred in new_where_clause.predicates.iter_mut(){
            
                        if let Some((name,bounds)) = get_pred_bounds(pred){
    
                            // position name in live_pred
                            if let Some(pos) = live_pred.iter().position(|x| x.0.eq(name)){
                                let (_,live_bounds) = live_pred.remove(pos);
                                for bnd in live_bounds {
                                    if !bounds.iter().any(|b| b.eq(&bnd)){
                                        bounds.push(bnd.clone());
                                    }
                                }
                            }
                        }
                    }
                    // push leftover predicates if any
                    for (type_param_name,bounds) in live_pred{
    
                        new_where_clause.predicates.push(syn::parse_quote! {
                            #type_param_name: #bounds
                        });
                    }
                    return (Some(impl_generics), Some(ty_generics), Some(new_where_clause));
    
                } else {
                    return (Some(impl_generics), Some(ty_generics), Some(inter_where_clause));
                }
            }
        }
        let where_clause = where_clause.as_mut().map(|x| x.clone());
        return (Some(impl_generics), Some(ty_generics), where_clause);

    } else {
        return (None,None,None);
    }

}



