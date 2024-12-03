
use std::collections::{HashMap, HashSet};
use syn::{ 
    parse_quote,Generics,WherePredicate,GenericParam,GenericArgument,
    Type,ItemImpl,TypePath,Signature,TypeParamBound,
    Lifetime, LifetimeParam, PredicateLifetime, PredicateType, TypeParam 
};
use proc_macro2::TokenStream;
use proc_macro_error::abort_call_site;
use super::ModelPhantomData;
use quote::quote;


pub struct GenWork<'a> {
    pub impl_gen: &'a Generics,
    actor_gen_set: HashMap<GenericArgument,GenericParam>,
}

impl<'a> GenWork<'a> {

    pub fn new(impl_gen: &'a Generics) -> Self {
        let actor_gen_set = 
        impl_gen.params
            .iter()
            .cloned()
            .filter_map(|item| 
            if gen_params::is_const(&item){ 
                None 
            } else {
                Some((gen_params::as_arg(&item), item))
            }
            )
            .collect();

        Self { impl_gen, actor_gen_set }
    }

    pub fn retain(&mut self, sig : &Signature ){
        self.actor_gen_set.retain(|k,_| !crate::model::includes(&sig, k ));
    }

    pub fn get_mod_gen(&self, full: bool) -> ModelGenerics {

        if full { 
            let mut new_gen =self.impl_gen.clone();
            add_model_bounds(&mut new_gen);
            return ModelGenerics::new(new_gen);
        }

        let mut script_gen:  Generics = Default::default();
        let mut private_gen: Generics = Default::default();
        let mut live_gen = self.impl_gen.clone();

        let param_set = self.actor_gen_set.keys().collect::<HashSet<_>>();

        // calc params for private
        let params = 
            self.impl_gen.params.iter().filter_map(|p| {
                if param_set.contains(&gen_params::as_arg(p)) {
                    Some(p.clone())
                } else { None }
        }).collect::<Vec<_>>();
        private_gen.params.extend(params.iter().cloned());

        // calc params for script
        let params = 
            self.impl_gen.params.iter().filter_map(|p| {
                if param_set.contains(&gen_params::as_arg(p)) {
                    None
                } else { Some(p.clone()) }
        }).collect::<Vec<_>>();

        // add params
        script_gen.params.extend(params.iter().cloned());

        // calc preds for script
        let preds = 
        live_gen.make_where_clause().predicates.iter()
            .filter_map(|wp| {
                if param_set.iter().any(|p| crate::model::includes(wp,p)){
                    None
                } else { Some( wp.clone())}
            });

        // add preds 
        script_gen.make_where_clause().predicates.extend(preds);
        
        let phantom_data =  ModelPhantomData::from( &self.actor_gen_set);


        // add model bounds
        add_model_bounds(&mut live_gen);
        add_model_bounds(&mut script_gen);
        ModelGenerics{
            script_gen, 
            private_gen,
            live_gen, 
            phantom_data,
        }

    }

}




#[derive(Clone)]
pub struct ModelGenerics {

    pub script_gen:  Generics,
    pub live_gen:    Generics,
    pub private_gen: Generics,
    pub phantom_data: ModelPhantomData,
}


impl ModelGenerics {

    pub fn new(gen:Generics) -> Self {
        Self {
            script_gen: gen.clone(),
            live_gen: gen,
            private_gen: Generics::default(),
            phantom_data: ModelPhantomData::default(),
        }
    }

    pub fn get_script_live_impl_block(&self, script_type: &TypePath, live_type: &TypePath) -> (ItemImpl,ItemImpl) {

        let mut script: ItemImpl =  parse_quote!( impl #script_type {} );
        let mut live: ItemImpl =  parse_quote!( impl #live_type {} );

        script.generics = self.script_gen.clone();
        live.generics = self.live_gen.clone();

        (script,live)
    }
}


/// add Generic restrictions to existing
/// parameters if not present
pub fn add_model_bounds(gen: &mut Generics ){

    let params = gen.params.clone();

    for param in params.iter() {

        if !gen_params::is_const(param){
            add_bounds(param, gen);
        }
    }
}

pub fn add_bounds( param: &GenericParam, gen: &mut Generics ){
    // it will not work for lifetimes but we do it anyway just to keep it consistent
    if let GenericParam::Lifetime(LifetimeParam{lifetime: param_lifetime,bounds: param_bounds,..}) = param {
        let static_lifetime: &Lifetime = &parse_quote!{ 'static };
        if param_bounds.iter().any(|pblt| *pblt == *static_lifetime ){ return; }

        for pred in gen.make_where_clause().predicates.iter_mut() {
            if let WherePredicate::Lifetime(PredicateLifetime{ lifetime,bounds,..}) = pred {

                if param_lifetime == lifetime {
                    if bounds.iter().any(|pblt| *pblt == *static_lifetime ){ return; }
                    bounds.push(static_lifetime.clone());
                    return;
                }
            }
        }
        // if isn't present add lifetime
        gen.make_where_clause().predicates.push( parse_quote!( #param_lifetime : #static_lifetime));
        return;
    } 

    
    if let GenericParam::Type(TypeParam{ident,bounds: param_bounds,.. }) = param{
        let param_ty: Type = parse_quote!{ #ident };
        let mut model_bounds: Vec<TypeParamBound> = 
        vec![
            parse_quote!{ Send },
            parse_quote!{ Sync },
            parse_quote!{ 'static }, 
        ]; 

        // check param bounds
        if let Some(pos) = model_bounds.iter().position(|mb| param_bounds.iter().any(|pb| *mb == *pb )){
            model_bounds.remove(pos);
        }
        if model_bounds.is_empty(){ return; }

        for pred in gen.make_where_clause().predicates.iter_mut() {
            if let WherePredicate::Type(PredicateType{bounded_ty,bounds,..}) = pred {
                    
                if param_ty.eq(bounded_ty){
                    // check existing bounds
                    for mb in model_bounds.clone(){
                        if !bounds.iter().any(|b| mb.eq(b)){
                            bounds.push(mb);
                        }
                    }
                    return;
                }
            }
        }
        // if not present add type 
        gen.make_where_clause().predicates.push( parse_quote!( #param_ty : #(#model_bounds)+* ));
        return;

    }
}

pub fn is_empty(gen: &Generics) -> bool {
    let wcb =         
    if let Some(where_clause) = &gen.where_clause{
        where_clause.predicates.is_empty()
    } else { true };
    gen.params.is_empty() && wcb
}

pub fn get_some_turbo( gen: &Generics ) -> Option<TokenStream>{
    if !is_empty(gen){
        let (_,gen_ty,_) = gen.split_for_impl();
        let turbo_gen = gen_ty.as_turbofish();
        Some(quote!{ #turbo_gen })
    } else { None }
}


pub mod turbofish {

    use proc_macro2::Span;
    use super::{TypePath,abort_call_site};
    use syn::{ Path,Type, PathArguments};

    /// this function will cover cases when the 'TypeGenerics::turbofish' fails
    /// if the type has a concrete type prdefined 
    /// ```compile_fail
    ///  struct Bla<A,B>{..}
    ///  // B is String
    ///  impl<A> Bla <A,String> {..}
    ///           
    /// ```
    /// ```TypeGenerics::turbofish``` will return ```Bla::<A>```

    pub fn from_type_path(type_path: &TypePath) -> TypePath {
        let mut segments = type_path.path.segments.clone();

        if let Some(mut pair) = segments.pop() {
            if let PathArguments::AngleBracketed(ref mut angle_bracketed) = pair.value_mut().arguments {
                angle_bracketed.colon2_token = Some(syn::Token![::]([Span::call_site(), Span::call_site()]));
                segments.push(pair.value().clone());
    
                let modified_path = Path {
                    leading_colon: type_path.path.leading_colon,
                    segments,
                };
                return TypePath {
                    qself: type_path.qself.clone(),
                    path: modified_path,
                };       
            } else {
                return type_path.clone();
            }
        }
        abort_call_site!("Internal Error. 'generic::actor_turbofish::from_type_path'.'syn::Type::Path' is empty ");
    }

    pub fn from_type(ty: &Type) -> TypePath {
        if let Type::Path(type_path) = ty {
            return from_type_path(type_path);
        } else {
            abort_call_site!("Internal Error. 'generic::actor_turbofish::from_type'.expected 'syn::Type::Path' variant ");
        }
    }
}    



pub mod gen_params {

    use syn::{GenericParam,GenericArgument,Ident,parse_quote};


    pub fn get_ident(param: &GenericParam) -> Ident {
        match param {
            GenericParam::Const(ct) => ct.ident.clone(),
            GenericParam::Lifetime(lt) => lt.lifetime.ident.clone(),
            GenericParam::Type(ty) => ty.ident.clone(),
        }
    }

    pub fn as_arg(param: &GenericParam ) -> GenericArgument {
        
        match param {
            GenericParam::Lifetime(lt) => {
                let lt = &lt.lifetime;
                parse_quote!{ #lt }
            },
            _ => { 
                let ident = get_ident(&param);
                parse_quote!{ #ident } 
            },
        } 
    }

    pub fn is_const(param: &GenericParam) -> bool {
        if let GenericParam::Const(_) = param {
            return true;
        }
        false
    }
 
}
