
pub mod argument;
pub mod attribute;
pub mod generics;
pub mod method;
pub mod name;
pub mod generate;

pub use argument::*;
pub use attribute::*;
pub use generics::*;
pub use method::*;
pub use generate::*;



use proc_macro_error::{abort_call_site, abort};
use syn::{File, Generics, Ident, ImplItem, ImplItemFn, Item, ItemImpl, Pat, Type, TypePath};
use quote::quote;

#[derive(Clone)]
pub struct ModelPart{

    pub def:  Option<Item>,
    pub mets: Vec<(Ident,ImplItemFn)>,
    pub trts: Vec<(Ident,Item)>, 
    pub impl_block: ItemImpl,
    edit: bool,
}


impl ModelPart {

    /// this method should take the right 'live and 'script' edits and definitions
    pub fn new(
        def:  Option<Item>,
        mets: Vec<(Ident,ImplItemFn)>,
        trts: Vec<(Ident,Item)>, 
        impl_block: ItemImpl,
        ) -> Self { 
        Self { def,mets,trts,impl_block, edit:false }
    }

    fn new_empty(&self) -> Self {
        Self { def: None, mets: vec![], trts: vec![], impl_block: self.impl_block.clone(), edit: true } 
    }

    pub fn split_edit(&mut self,((def,scope_def),mets,trts): &( (bool,bool), (Option<Vec<(syn::Ident,bool)>>,bool), (Option<Vec<(syn::Ident,bool)>>,bool) )) -> Self {
        let mut other = self.new_empty();

        if *def {
            let temp_def = self.def.take();
            if *scope_def {
                other.def = temp_def;
            }
        }
        // original 
        other.mets = select(mets,&mut self.mets);
        other.trts = select(trts,&mut self.trts);

        other
    }

    pub fn is_live(&self) -> bool {
        if let Some(Item::Struct(_)) = self.def {
            return true;
        }
        false
    }

    pub fn get_met_new_args(&self) -> Vec<Box<Pat>> {

        if let Some(pos) = self.mets.iter().position(|x| x.0 == "new" || x.0 == "try_new"){
            args_to_pat_type(&ident_arguments_output(&self.mets[pos].1.sig).1).0
        } else {
            abort_call_site!(" InternalError `ModelPart` expected method `new` to be present ")
        }
    }
}

impl From<ModelPart> for Vec<Item> {
    fn from(value: ModelPart) -> Self {
        let mut items = vec![];
        let ModelPart{def,mets,trts,mut impl_block, edit } = value;

        if let Some(def) = def { items.push(def); }

        if !mets.is_empty() {
            for (_,met) in mets {
                impl_block.items.push(ImplItem::Fn(met));
            }
            items.push(Item::Impl(impl_block));
        }

        items.extend( trts.into_iter().map(|(_,t)|t));

        if edit == true {
            for item in items.iter_mut() { remove_doc_comment(item)}
        }
        items
    }
} 

#[derive(Clone)]
pub enum ModelSdpl {
    Actor(ActorModelSdpl),
    Family(FamilyModelSdpl),
}

impl ModelSdpl {

    pub fn get_code_edit(&mut self) -> (File, File){
        match self {
            Self::Actor(ams) => ams.get_code_edit(),
            Self::Family(fms) => fms.get_code_edit(),
        }
    }
}


#[derive(Clone)]
pub struct FamilyModelSdpl {

    pub aaa: ActorAttributeArguments,
    pub live:              ModelPart,
    pub actors:  Vec<ActorModelSdpl>,
}

impl FamilyModelSdpl {

    pub fn get_code_edit(&mut self) -> (File, File){
        let mut code_file = File { shebang: None,attrs: vec![],items: vec![] };
        let mut edit_file = code_file.clone();

        let EditActor{ live,..  } = &self.aaa.edit;
        edit_file.items.extend(<Vec<Item>>::from(self.live.split_edit(live)).into_iter());
        code_file.items.extend(<Vec<Item>>::from(self.live.clone()).into_iter());

        for act in &mut self.actors {
            let(c,e) = act.get_code_edit();
            code_file.items.extend(c.items.into_iter());
            edit_file.items.extend(e.items.into_iter());
        }

        (code_file,edit_file)
    }
}

#[derive(Clone)]
pub struct ActorModelSdpl {
    pub aaa: ActorAttributeArguments,
    pub met_new:  Option<MethodNew>,

    pub script:  ModelPart,
    pub live:    ModelPart,

}

impl ActorModelSdpl {

    pub fn get_script_live_type(&self) -> (TypePath,TypePath) {

        let (_,script,_) = get_ident_type_generics(&self.script.impl_block);
        let (_,live,_) = get_ident_type_generics(&self.live.impl_block);

        (script,live)
    }

    pub fn get_code_edit(&mut self) -> (File, File){

        let mut code_file = File { shebang: None,attrs: vec![],items: vec![] };
        let mut edit_file = code_file.clone();
        let EditActor{ script, live, ..  } = &self.aaa.edit;

        edit_file.items.extend(<Vec<Item>>::from(self.script.split_edit(script)).into_iter());
        edit_file.items.extend(<Vec<Item>>::from(self.live.split_edit(live)).into_iter());

        code_file.items.extend(<Vec<Item>>::from(self.script.clone()).into_iter());
        code_file.items.extend(<Vec<Item>>::from(self.live.clone()).into_iter());

        (code_file,edit_file)
    }
}

fn remove_doc_comment( item: &mut Item ){

    let attrs = 
    match item {

        Item::Enum(item_enum) => vec![ &mut item_enum.attrs],
        Item::Struct(item_struct) =>  vec![ &mut item_struct.attrs],
        Item::Impl(item_impl)  => {
            let mut col = vec![ &mut item_impl.attrs];
            for impl_item in item_impl.items.iter_mut(){
                if let ImplItem::Fn(ImplItemFn{ attrs,..}) = impl_item{
                    col.push(attrs);
                }
            }
            col 
        },
        _ =>{ return ();}
    };
    for attr in attrs {
        let new_attr = 
            attr.iter()
                .cloned()
                .filter(|x|  !x.path().is_ident("doc"))
                .collect::<Vec<_>>();
        *attr = new_attr;
    }
}


pub fn select<T>(
    (   edit_idents,scope): &(Option<Vec<(Ident,bool)>>,bool), 
        ident_mets: &mut Vec<(Ident,T)> 
    ) -> Vec<(Ident,T)> {

    let mut res = Vec::new();

    if let Some(idents) = edit_idents { 

        if idents.is_empty() {

            let temp_ident_mets = std::mem::replace(ident_mets,Vec::new());
            if *scope {
                res = temp_ident_mets;
            }
        }
        for (ident,scp) in idents {
            if let Some(pos) = ident_mets.iter().position(|x| x.0 == *ident){
                let value  = ident_mets.remove(pos);
                if *scope || *scp {
                    res.push(value);
                }
            } else {
                abort!(ident,"Unknown ident.");
            }
        }
    } 
    res

}


pub const CHAR_SET: [char;7] = ['}','{',']','[',')','(',','];

pub fn space_around_chars( mut s: String, char_set: &[char]) -> String {
    for c in char_set{
        s = s.replace(*c, &format!(" {c} "));
    }
    s
}

pub fn to_string_wide<T>(ty: &T) -> String 
where T: quote::ToTokens,
{
    let mut type_str = quote! {#ty}.to_string();
    type_str = space_around_chars(type_str, &CHAR_SET);
    format!(" {type_str} ")
}

pub fn replace<T, O, N>(ty: &T, old: &O, new: &N) -> T
where
    T: syn::parse::Parse + quote::ToTokens,
    O: quote::ToTokens,
    N: quote::ToTokens,
{   
    let type_str = to_string_wide(&ty);
    let old = to_string_wide(&old);
    let new = format!(" {} ",quote!{#new}.to_string());
    let str_type = type_str.replace(&old, &new);
    if let Ok(ty) = syn::parse_str::<T>(&str_type) {
        return ty;
    }
    let msg = format!("Internal Error. 'model::replace'. Could not parse &str to provided type! str_type - '{}'",str_type);
    abort_call_site!(msg);
}


pub fn includes<T,Y>(ty: &T, item: &Y) -> bool
where
    T: quote::ToTokens,
    Y: quote::ToTokens,
{   
    let type_str = to_string_wide(ty);
    let item = to_string_wide(item);
    type_str.contains(&item)
}

pub fn get_ident_type_generics(item_impl: &ItemImpl) -> (Ident,TypePath,Generics) {

    match &*item_impl.self_ty {
        Type::Path(tp) => {
            let ident = tp.path.segments.last().unwrap().ident.clone();
            let generics = item_impl.generics.clone();
            (ident,tp.clone(),generics)
        },
        _ => {
            let msg ="Internal Error.'model::mod::impl_get_ident_type_generics'. expected a path!";
            abort!(item_impl,msg);
        } 
    }

}



