
use syn::{ parse_quote, AngleBracketedGenericArguments, Attribute, FnArg,
    GenericArgument, Generics, Ident, ImplItem, ImplItemFn, ItemImpl,
    Pat, PatType, Path, PathArguments, ReturnType, Signature, Token, Type, 
    TypePath, TypeReference, Visibility };

use proc_macro_error::{abort, abort_call_site};
use proc_macro2::{TokenStream,Span};
use quote::quote;

use crate::{ error, model::{ self, name, ModelGenerics,GenWork,
    ActorAttributeArguments, InterVars, Lib, ModelFilter, OneshotChannel}};

#[derive(Debug,Clone)]
pub enum  ERType {
    String,
    StatString,
}

impl ERType{

    pub fn get_some_to_string_call(&self) -> Option<TokenStream> {
        if let Self::String = self {
            return Some(quote!{ .to_string()});
        }
        None
    }
}

#[derive(Debug,Clone)]
 pub enum ModelOutput {
    Result(Path,Option<ERType>),
    Option(Path),
    None,
}


impl ModelOutput {

    pub fn return_ok(&self, ret_init: &TokenStream ) -> TokenStream {

        match self {
            Self::Result(path,_) =>  quote!{ #path :: Ok ( #ret_init )},
            Self::Option(path)   =>  quote!{ #path :: Some( #ret_init )},
            Self::None                  =>  quote!{ #ret_init },
        }
    }

    pub fn return_err(&self, err_msg: &'static str ) -> TokenStream {
        match self {
            Self::Result(path,r_ty) =>  { 
                if r_ty.is_none(){ abort_call_site!("InternalError 'ModelOutput::return_err' expected a type for Result::Err ") }

                let to_str_call = r_ty.as_ref().unwrap().get_some_to_string_call();
                quote!{ #path :: Err ( #err_msg #to_str_call )}
            },
            Self::Option(path)   =>  quote!{ #path :: None },
            Self::None                  =>  abort_call_site!("InternalError 'ModelOutput::return_err' expected a Option or Result variant "),
        }
    }

    pub fn would_return_if_err(&self) -> bool {
        match self {
            Self::Result(_,Some(_)) => return true,
            Self::Option(_) => return true,
            _ =>(),
        }
        false
    }

    pub fn unwrap_sign(&self) -> Option<TokenStream> {
        if self.is_some() {Some(quote!{?})} else  {None} 
    }

    pub fn is_none(&self) -> bool {
        if let Self::None = self {
            return true;
        }
        false
    }

    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    pub fn is_result(&self) -> bool{
        if let Self::Result(_,_) = self {
            return true;
        }
        false
    }

}


#[derive(Debug,Clone)]
pub struct MethodNew {
    pub _webs: Vec<Attribute>,
    pub mod_output: ModelOutput,
    pub args_idents: Vec<Box<Pat>>,
    pub turbo_gen: Option<TokenStream>,
    pub met: ImplItemFn,
}




#[derive(Debug,Clone)]
pub enum ModelMethod {
    Io  { met: ImplItemFn, await_call: Option<TokenStream>, _webs: Vec<Attribute>, turbo_gen: Option<TokenStream>, is_stat: bool, is_mut: bool, args: Vec<FnArg>, inter_vars: Option<InterVars>, output: Type },   
    I   { met: ImplItemFn, await_call: Option<TokenStream>, _webs: Vec<Attribute>, turbo_gen: Option<TokenStream>, is_stat: bool, is_mut: bool, args: Vec<FnArg>, inter_vars: Option<InterVars>, },    
    O   { met: ImplItemFn, await_call: Option<TokenStream>, _webs: Vec<Attribute>, turbo_gen: Option<TokenStream>, is_stat: bool, is_mut: bool, output: Type },    
    Void{ met: ImplItemFn, await_call: Option<TokenStream>, _webs: Vec<Attribute>, turbo_gen: Option<TokenStream>, is_stat: bool, is_mut: bool, }, 
    Slf { met: ImplItemFn, await_call: Option<TokenStream>, _webs: Vec<Attribute>, turbo_gen: Option<TokenStream>, is_stat: bool, args_idents: Vec<Box<Pat>>, mod_output: ModelOutput },
    Stat{ met: ImplItemFn, await_call: Option<TokenStream>, _webs: Vec<Attribute>, turbo_gen: Option<TokenStream>, args_idents: Vec<Box<Pat>>},
}

impl ModelMethod {

    pub fn get_met_mut(&mut self ) -> &mut ImplItemFn {
        match self {
            Self::Io  {met,..}|
            Self::I   {met,..}|
            Self::O   {met,..}|
            Self::Void{met,..}|
            Self::Slf {met,..}| 
            Self::Stat{met,..} => met,
        }
    }

    pub fn get_met(&self ) -> &ImplItemFn {
        match self {
            Self::Io  {met,..}|
            Self::I   {met,..}|
            Self::O   {met,..}|
            Self::Void{met,..}|
            Self::Slf {met,..}|
            Self::Stat{met,..} => met,
        }
    }

    pub fn is_async(&self) -> bool {
        self.get_met().sig.asyncness.is_some()
    }

    pub fn is_stat(&self) -> bool {
        if let Self::Stat{..} = self {
            return true;
        }
        false 
    }

    pub fn is_slf(&self) -> bool {
        if let Self::Slf{..} = self {
            return true;
        }
        false 
    }

    pub fn is_ref(&self) -> bool {
        match self {
            Self::Io  {..}|
            Self::I   {..}|
            Self::O   {..}|
            Self::Void{..} => true,
            _ => false 
        }
    }

    pub fn is_gen(&self) -> bool {
        match self {
            Self::Io  {turbo_gen,..}|
            Self::I   {turbo_gen,..}|
            Self::O   {turbo_gen,..}|
            Self::Void{turbo_gen,..}|
            Self::Slf {turbo_gen,..}|
            Self::Stat{turbo_gen,..} => turbo_gen.is_some(),
        }
    }

    pub fn set_await_call(mets: &mut Vec<Self>){
        for met in mets {
            
            match met {
                Self::Io  {met,await_call,..}|
                Self::I   {met,await_call,..}|
                Self::O   {met,await_call,..}|
                Self::Void{met,await_call,..}|
                Self::Slf {met,await_call,..}| 
                Self::Stat{met,await_call,..} => {
                    *await_call = met.sig.asyncness.as_ref().map(|_| quote!{.await} );
                },
            }
        }
    }


    pub fn to_async(&mut self,  lib: &Lib) {
        if !self.is_stat() && !lib.is_std(){
            self.get_met_mut().sig.asyncness = Some(Token![async](Span::call_site()));
        }
    }

    pub fn get_ident_field_name(&self) -> (Ident, Ident) {
        let ident = self.get_met().sig.ident.clone();
        let field_name = name::script_field(&ident);
        ( ident, field_name )
    }
}



#[derive(Debug)]
pub enum ModelMethod1{

    Ref (ImplItemFn,bool),
    Stat(ImplItemFn),
    Slf (ImplItemFn),
}

pub struct ImplWork<'a> {

    actor_ty: &'a TypePath,
    actor_turbo_ty: TypePath,
    aaa: &'a ActorAttributeArguments,
    filter: ModelFilter,

    gen_work: GenWork<'a>,

    pub met_new: Option<MethodNew>,
    pub met_cont: Vec<ModelMethod>,
}

impl<'a> ImplWork<'a> {

    pub fn new( actor_ty: &'a TypePath, impl_gen: &'a Generics, aaa: &'a ActorAttributeArguments ) -> Self {

        let filter = ModelFilter::new(&aaa);
        let actor_turbo_ty = crate::model::turbofish::from_type_path(&actor_ty);
        let gen_work = GenWork::new(&impl_gen);
        Self { actor_ty, actor_turbo_ty, aaa, filter, gen_work, met_new: None, met_cont: vec![] }
    }

    pub fn process_impl(&mut self, item_impl: &ItemImpl){

        for itm in &item_impl.items {
            if let ImplItem::Fn( met ) = itm { 
                match met.vis {
                    Visibility::Inherited => {continue;},
                    _ => {
                        if let Some(mm) = self.first_met_sort(met){
                            self.process_met(mm,false);
                        }
                    },
                } 
            }
        }
    }

    fn process_met(&mut self, mm: ModelMethod1, is_stat: bool) {

        match mm {
            // in methods that have local generic types 
            ModelMethod1::Ref( mut met, is_mut)  => { 
                
                if !self.filter.condition(&met.sig){ return; }
                let _webs = take_doc_web_attrs(&mut met);
                self.substitute_args_type_and_return_type(&mut met);

                let turbo_gen = crate::model::get_some_turbo(&met.sig.generics);

                if turbo_gen.is_none(){
                    // check for Context Generics and return method turbo
                    self.gen_work.retain(&met.sig);
                }

                let arg_flag = super::if_args_and_clean_pats(&mut met.sig);
                
                let mod_met = 
                    match &met.sig.output.clone() {
                        ReturnType::Type(_,output) => { 

                            if arg_flag {
                                ModelMethod::Io{ met, await_call: None, _webs, turbo_gen, is_stat, is_mut, args: vec![], inter_vars: None, output: (**output).clone() }
                            } else {
                                ModelMethod::O{ met, await_call: None, _webs, turbo_gen, is_stat, is_mut, output: (**output).clone() }
                            }
                        },
                        ReturnType::Default => {

                            if arg_flag {
                                ModelMethod::I{ met, await_call: None, _webs, turbo_gen, is_stat, is_mut, args: vec![], inter_vars: None }
                            } else {
                                ModelMethod::Void{ met, await_call: None, _webs, turbo_gen, is_stat, is_mut }
                            }
                        },
                    };
                self.met_cont.push( mod_met );
            },

            // needs to work NOO for SELF consumming 
            ModelMethod1::Slf(mut met)  => {
                // if is an actor of family that and is not static - ignore
                // we can't raise an error since there may be different macros 
                // for the same block
                if !is_stat  && !self.aaa.mod_receiver.is_slf() { return; }

                if !self.filter.condition(&met.sig){ return; }
                let _webs = take_doc_web_attrs(&mut met);
                self.aaa.mod_receiver.remove_mut(&mut met);
                self.substitute_args_type_and_return_type(&mut met);
                let args_idents = change_sig_get_args_idents(&mut met);
                let turbo_gen = crate::model::get_some_turbo(&met.sig.generics);

                match &met.sig.output {

                    ReturnType::Type(_,ty) => {
                        if let Some(opt_path)= is_some_path_of("Option",&*ty ){
                                    self.met_cont.push( ModelMethod::Slf{met, await_call: None, _webs, turbo_gen, is_stat, args_idents, mod_output: ModelOutput::Option(opt_path)} );
                                    return;
                        } 

                        if let Some(res_path)= is_some_path_of("Result",&*ty ){
                            if let syn::Type::Path(syn::TypePath{qself:None, path: Path{ segments,..},..})  = *ty.clone() { 
                                if let Some(path_seg) = segments.last() {
                                    if let PathArguments::AngleBracketed(AngleBracketedGenericArguments{args,..}) = &path_seg.arguments {
                                        if args.len() == 2 {
                                            let gen_types = args.iter().collect::<Vec<_>>();
                                            if let GenericArgument::Type(ty) = &gen_types[1]{
                                                match ty {
                                                    // expecting `String`
                                                    Type::Path(syn::TypePath{qself:None,..}) => {
                                                        if let Some(_) = is_some_path_of( "String", ty){
                                                            self.met_cont.push( ModelMethod::Slf{met, await_call: None, _webs, turbo_gen, is_stat, args_idents, mod_output: ModelOutput::Result(res_path,Some(ERType::String))} );
                                                            return;
                                                        }
                                                    },
                                                    // expecting `&'static str`
                                                    Type::Reference( TypeReference{lifetime: Some(lt),elem,..}) => {
                                                        if lt.ident == "static"{
                                                            if let Some(_) = is_some_path_of( "str", &**elem){
                                                                self.met_cont.push( ModelMethod::Slf{met, await_call: None, _webs, turbo_gen, is_stat, args_idents, mod_output: ModelOutput::Result(res_path,Some(ERType::StatString))} );
                                                                return;
                                                            }
                                                        }
                                                    },
                                                    _ => (),
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            self.met_cont.push( ModelMethod::Slf{met, await_call:None, _webs, turbo_gen, is_stat, args_idents, mod_output: ModelOutput::Result(res_path, None)} );
                            return; 
                        } 
                    },
                    _ => (),
                }
                self.met_cont.push( ModelMethod::Slf{met, await_call: None, _webs, turbo_gen, is_stat, args_idents, mod_output: ModelOutput::None} ); 
                return;
            },

            ModelMethod1::Stat(mut met) => {

                // for method new 
                if met.sig.ident == "new" || met.sig.ident == "try_new" {

                    let _webs = take_doc_web_attrs(&mut met);
                    let(new_sig,mod_output) = 
                    super::check_self_return(&met.sig,&self.actor_ty);
                    met.sig = new_sig;
                    let args_idents = change_sig_get_args_idents(&mut met);
                    let turbo_gen = crate::model::get_some_turbo(&met.sig.generics);
                    self.met_new = Some( MethodNew{ met,args_idents, mod_output, _webs,turbo_gen } );

                } else {
                    //check if the receiver is 'actor'
                    if !is_stat {

                        self.process_met( self.aaa.mod_receiver.second_sort(met,&self.actor_ty, &self.aaa.lib), true);

                    } else {

                        let _webs = take_doc_web_attrs(&mut met);
                        if !self.filter.condition(&met.sig){ return; }
                        self.substitute_args_type_and_return_type(&mut met);
                        let args_idents = change_sig_get_args_idents(&mut met);
                        let turbo_gen = crate::model::get_some_turbo(&met.sig.generics);
                        self.met_cont.push( ModelMethod::Stat{ met, await_call: None,  _webs, turbo_gen, args_idents} );

                    }
                }
            },
        }
    }


    /// this function can be called after 
    /// full first process
    pub fn get_mod_gen(&self) -> ModelGenerics {
        // check filter 
        self.filter.check();

        // each condition triggers "Script" and "Live" parts to have the same generics  
        let has_self_consm_mets = self.met_cont.iter().any(|m| m.is_slf());
        let has_loc_gen_mets = self.met_cont.iter().filter(|m| m.is_ref()).any(|m| m.is_gen());

        self.gen_work.get_mod_gen(has_self_consm_mets || has_loc_gen_mets)
    }

    pub fn get_methods(&mut self, one: &OneshotChannel ) -> ( Option<MethodNew>, Vec<ModelMethod>, Option<TokenStream> ){

        // add a check for method new here 
        if self.met_new.is_none() {

            let (note,help) = error::met_new_note_help(&self.actor_ty);
            abort_call_site!("Can not find public/restricted  method `new` or `try_new` .";note=note;help=help);

        }

        // set 'await_call' field
        ModelMethod::set_await_call(&mut self.met_cont);

        // check inter vars
        self.check_set_inter_vars(one);

        // check the asyncness for Script methods 
        let async_decl = {

            match &self.aaa.lib {
    
                Lib::Std => {
                    let methods = self.met_cont.iter().filter(|m| !m.is_slf() && !m.is_stat() ).collect::<Vec<_>>();
                    if let Some(pos) = methods.iter().position(|m| m.is_async()){
                        error::abort_async_no_lib(&methods[pos]);
                    }
                    None
                },
                _ => { Some(quote!{async}) },
            }
        };

        // for ref methods set model generic bounds if generics local
        let mut mets = vec![];
        
        for mut met in std::mem::take(&mut self.met_cont){
            if  met.is_ref() && met.is_gen() {
                let m = met.get_met_mut();
                crate::model::add_model_bounds(&mut m.sig.generics);
            }
            mets.push(met);
        }

        ( std::mem::take(&mut self.met_new),mets, async_decl )
    }

    pub fn check_set_inter_vars(&mut self, one: &OneshotChannel ){

        for met in self.met_cont.iter_mut(){
            match met {
                ModelMethod::Io{met,args,inter_vars,..}=>{
                    set_args_inter_vars(self.aaa.interact, met, args, inter_vars,None);
                },
                ModelMethod::I{met,args,inter_vars,..}=>{
                    set_args_inter_vars(self.aaa.interact, met, args, inter_vars,Some(one));
                },
                _=>(),
            }
        }
    }

    pub fn first_met_sort(&mut self, met: &ImplItemFn ) -> Option<ModelMethod1> {

        if let Some(input) = met.sig.inputs.first() {
            
            if let FnArg::Receiver(receiver) = input{
                let slf: syn::token::SelfValue = Default::default();
                
                if receiver.self_token == slf {
                    if receiver.reference.is_some(){
                        // reference 
                        return Some(ModelMethod1::Ref(met.clone(), receiver.mutability.is_some()));
                    } else {
    
                        // self consumming   
                        return Some(ModelMethod1::Slf(met.clone()));
                    }
                } 
                return None;
            } 
        }  
        // static 
        return Some(ModelMethod1::Stat(met.clone()));
         
    }

    fn substitute_args_type_and_return_type(&self, met: &mut ImplItemFn ){
        // we'll try to keep as much as it is possible from original Spans 
        let turbo_self = quote!{ Self:: };
        if crate::model::includes( &met.sig, &turbo_self  ){
            substitute(&mut met.sig, &turbo_self, &self.actor_turbo_ty );
        }

        let slf = quote!{ Self };
        if crate::model::includes( &met.sig, &slf ){
            substitute(&mut met.sig, &slf,&self.actor_ty);
        }
    }

}

fn substitute<O,N>( sig: &mut Signature, old:&O, new: &N )
where   O: quote::ToTokens,
        N: quote::ToTokens,
{   

    for arg in sig.inputs.iter_mut(){
        if let FnArg::Typed(PatType{ty,..}) = arg {
            if let Type::Reference( TypeReference { elem,.. } ) = &mut **ty {
                if crate::model::includes( &**elem , &old) {
                    **elem = crate::model::replace(&**elem,old,new);
                }
            }  else {
                if crate::model::includes( &**ty, &old ) {
                    **ty = crate::model::replace(&**ty,old,new);
                }
            }
        }
    }

    if let ReturnType::Type(_, ty) = &mut sig.output{
        if crate::model::includes( &**ty, &old ) {
            **ty = crate::model::replace(&**ty,old,new);
        }
    }
}

fn change_sig_get_args_idents( met: &mut ImplItemFn ) -> Vec<Box<Pat>> {
    
    if super::if_args_and_clean_pats(&mut met.sig){
        let (live_arguments,live_sig) = super::get_live_args_and_sig(&met.sig );
        let (args_idents,_) = super::args_to_pat_type(&live_arguments);
        met.sig  = live_sig;
        return args_idents;
    }
    vec![]
}

fn take_doc_web_attrs(met: &mut ImplItemFn) -> Vec<Attribute> {

    let attrs = std::mem::take(&mut met.attrs);

    met.attrs = 
        attrs
            .iter()
            .cloned()
            .filter(|x|  x.path().is_ident("doc"))
            .collect::<Vec<_>>();

    let web_paths: Vec<Path> = vec![ parse_quote!{ interthread::web }, parse_quote!{ web } ];

        attrs
            .into_iter()
            .filter(|x| web_paths.iter().any(|p| p.eq(x.path()) ))
            .collect::<Vec<_>>()             
}


fn set_args_inter_vars(interact: bool, met: &mut ImplItemFn, args: &mut Vec<FnArg>, inter_vars: &mut Option<InterVars>, one: Option<&OneshotChannel> ){
    let _ = super::if_args_and_clean_pats(&mut met.sig);
    let (live_arguments,live_sig) = super::get_live_args_and_sig(&met.sig );
    if let Some(i_vars) = get_some_inter_vars( interact, &met.sig, one){
        met.sig = i_vars.new_sig.clone();
        *args = live_arguments;
        *inter_vars = Some(i_vars);

    } else {
        met.sig = live_sig;
        *args = live_arguments;
        *inter_vars = None;
    }
}


fn get_some_inter_vars(interact: bool, sig: &Signature, one: Option<&OneshotChannel>) -> Option<InterVars> {
    if interact { 
        model::get_variables( sig,one )
    } else {
        // check for inter vars conflict
        if let Some((inter_var,pat)) = model::check_send_recv( &sig.inputs, None ){
            let msg = error::var_name_conflict(&inter_var,"parameter");
            abort!(pat,msg;note= error::INTER_SEND_RECV_RESTRICT_NOTE);
        }
        None
    }
}

// returns clean path
pub fn is_some_path_of( s:&'static str, ty: &Type ) -> Option<Path> {
    if let syn::Type::Path(syn::TypePath{qself:None, path,..})  = ty.clone() {
        if let Some(path_seg)  = path.segments.last() {
                                        
            if path_seg.ident == s {
                let path = crate::model::method::clean_path(&path);
                return Some(path);
            }
        }
    }
    None
}



