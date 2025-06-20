
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


pub struct GenWork {
    pub impl_gen: Generics,
    org_arg_param_map: HashMap<GenericArgument,GenericParam>,
    pub arg_param_map: HashMap<GenericArgument,GenericParam>,
    is_send: bool,
}

impl GenWork {

    pub fn new(impl_gen: &Generics,is_send: bool) -> Self {

        let arg_param_map: HashMap<GenericArgument,GenericParam> = 
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
        let org_arg_param_map = arg_param_map.clone();
        Self { impl_gen: impl_gen.clone(), org_arg_param_map, arg_param_map, is_send  }
    }

    pub fn retain_io(&mut self, sig : &Signature ){
        self.arg_param_map.retain(|k,_| !crate::model::includes(&sig, k ));
    }

    /// should be used for method 'new' when !Sync
    pub fn retain_i(&mut self, sig : &Signature ){
        self.arg_param_map.retain(|k,_| !crate::model::includes(&sig.inputs, k ));
    }

    // we do not attempt to extract 'where_predicates'from both 'where_clause's 
    // and add (extend) the bounds if the type (generic) will match.
    // later we'll try to take for 'script_gen' as many predicates as it is possible;
    // the condition being for those : 'if none of params from `private_gen` are included in';
    // this way more predicates will be lifted to 'script_gen'  

    // TODO: Maybe in 'method::vars::ImplVars::new' just before the initiation to call a method 
    //       which will clear the possible mess.

    pub fn take_bounds(&mut self, sig : &mut Signature ){
        if let Some(w_c ) = &sig.generics.where_clause.take(){
            self.impl_gen.make_where_clause().predicates.extend(
                w_c.predicates.iter().cloned()
            );
        } 
    }

    pub fn difference(&self) -> Vec<GenericParam> {
        let mut loc = Vec::new();
        for (arg,param) in self.org_arg_param_map.iter() {
            if !self.arg_param_map.contains_key(arg){
                loc.push(param.clone());
            }
        } 
        loc
    }

    pub fn get_mod_gen(&mut self, full: bool) -> ModelGenerics {

        let param_set = self.arg_param_map.keys().collect::<HashSet<_>>();
        
        if full || param_set.is_empty(){ 
            add_model_bounds(&mut self.impl_gen);
            return ModelGenerics::new(self.impl_gen.clone());
        }

        let mut script_gen:  Generics = Default::default();
        let mut private_gen: Generics = Default::default();
        let mut live_gen = self.impl_gen.clone();

        // split params between script and private
        for p in self.impl_gen.params.iter() {
            // !!! keep this order, so all constants will be included in 'script_gen'
            if param_set.contains(&gen_params::as_arg(p)) {
                private_gen.params.push(p.clone());
            } else {
                script_gen.params.push(p.clone());
            }
        }

        // split preds between script and private
        for wp in self.impl_gen.make_where_clause().predicates.iter() {
            if param_set.iter().any(|p| crate::model::includes(wp,p)){
                private_gen.make_where_clause().predicates.push(wp.clone());
            } else { 
                script_gen.make_where_clause().predicates.push(wp.clone());
            }
        }

        let phantom_data =  ModelPhantomData::from( &self.arg_param_map);


        // add model bounds
        if self.is_send {
            add_model_bounds(&mut live_gen);
        } else {
            // model bounds for params present in script only
            for param in script_gen.params.iter(){
                if !gen_params::is_const(param){
                    add_bounds(param, &mut live_gen);
                }
            }
        }
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
pub fn add_model_bounds(gen: &mut Generics){

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


pub fn get_some_turbo( gen: &Generics ) -> Option<TokenStream>{
    if !gen.params.is_empty(){
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
