
pub struct UseMacro {

    mod_name:  syn::Ident,
    mac_name:  syn::Ident,
    
    mac_path:  syn::Path,
    imp_path:  Option<syn::Path>,

}

impl UseMacro {

    pub fn new( mac_name: &str ) -> Self {

        let mod_name  = quote::format_ident!("{}",crate::INTERTHREAD);
        let mac_name  = quote::format_ident!("{}",mac_name); 
        
        let mac_path   = Self::create_path(Some(mod_name.clone()), mac_name.clone());
        
        Self { mod_name,mac_name, mac_path, imp_path: None }
    }

    pub fn exclude_self_macro(&self, item: &mut syn::Item){

        match item {
        
            syn::Item::Const(syn::ItemConst { attrs, .. })              => { let _ = std::mem::replace(attrs, self.exclude(&attrs)); },
            syn::Item::Enum(syn::ItemEnum {  attrs, .. })               => { let _ = std::mem::replace(attrs, self.exclude(&attrs)); },
            syn::Item::ExternCrate(syn::ItemExternCrate {  attrs, .. }) => { let _ = std::mem::replace(attrs, self.exclude(&attrs)); },
            syn::Item::Fn(syn::ItemFn {  attrs, .. })                   => { let _ = std::mem::replace(attrs, self.exclude(&attrs)); },
            syn::Item::ForeignMod(syn::ItemForeignMod {  attrs, .. })   => { let _ = std::mem::replace(attrs, self.exclude(&attrs)); },
            syn::Item::Impl(syn::ItemImpl {  attrs, .. })               => { let _ = std::mem::replace(attrs, self.exclude(&attrs)); },
            syn::Item::Macro(syn::ItemMacro {  attrs, .. })             => { let _ = std::mem::replace(attrs, self.exclude(&attrs)); },
            syn::Item::Mod(syn::ItemMod {  attrs, .. })                 => { let _ = std::mem::replace(attrs, self.exclude(&attrs)); },
            syn::Item::Static(syn::ItemStatic {  attrs, .. })           => { let _ = std::mem::replace(attrs, self.exclude(&attrs)); },
            syn::Item::Struct(syn::ItemStruct {  attrs, .. })           => { let _ = std::mem::replace(attrs, self.exclude(&attrs)); },
            syn::Item::Trait(syn::ItemTrait {  attrs, .. })             => { let _ = std::mem::replace(attrs, self.exclude(&attrs)); },
            syn::Item::TraitAlias(syn::ItemTraitAlias {  attrs, .. })   => { let _ = std::mem::replace(attrs, self.exclude(&attrs)); },
            syn::Item::Type(syn::ItemType {  attrs, .. })               => { let _ = std::mem::replace(attrs, self.exclude(&attrs)); },
            syn::Item::Union(syn::ItemUnion {  attrs, .. })             => { let _ = std::mem::replace(attrs, self.exclude(&attrs)); },
            syn::Item::Use(syn::ItemUse {  attrs, .. })                 => { let _ = std::mem::replace(attrs, self.exclude(&attrs)); },
            syn::Item::Verbatim(_)                                                           => {()},
            _=> (),
        };
    } 
    pub fn exclude(&self, attrs: &Vec<syn::Attribute>  ) -> Vec<syn::Attribute> {

        attrs.into_iter().filter_map(|x|
            {
                if self.is(x) { None } else { Some(x.clone()) }
            }
            ).collect::<Vec<_>>()
    }

    pub fn is(&self, attr: &syn::Attribute) -> bool { 
        if let Some(imp_path) = &self.imp_path {
            if self.mac_path.eq( attr.path() ) || imp_path.eq( attr.path() ) {
                return true;
            }
            return false;
        } else {
            if self.mac_path.eq(attr.path()){
                return true;
            }
            return false;
        }  
    }

    pub fn update(&mut self, mut item_use: syn::ItemUse ) -> Option<syn::ItemUse>{

        match self.file_self_use(&item_use.tree) {

            (Some(p), Some(t)) => {
                self.imp_path = Some(p);
                
                item_use.tree = t;
                Some(item_use)
            },

            (Some(p), None)             => {
                self.imp_path = Some(p);
                None
            },

            (None, Some(t))          => {
                item_use.tree = t;
                Some(item_use)
            },

            (None,None)                       => None,
        }
    }


    pub fn file_self_use(&self, item_use: &syn::UseTree ) -> (Option<syn::Path>, Option<syn::UseTree>) {

        match item_use.clone() {
    
            // A path prefix of imports in a `use` item: `std::...`.
            syn::UseTree::Path(mut use_path) => {
                if use_path.ident.eq(&self.mod_name){

                    match self.file_self_use( &*use_path.tree ){
                        // (Some, Some)
                        (Some(p), Some(t)) => { 
                            use_path.tree = Box::new(t);
                            return (Some(p), Some(syn::UseTree::Path(use_path)));
                        },
                        // (None, Some)
                        ( None, Some(t) ) => {
                            use_path.tree = Box::new(t);
                            return (None, Some(syn::UseTree::Path(use_path)));
                        },
                        // (Some, None)
                        ( Some(p), None ) => { 
                            return (Some(p), None) 
                        },
                        // (None, None)
                        (None,None) => { return (None, None) },
                    }
                }
                else { 
                    return (None, Some(item_use.clone()));
                }
            },
    
            // An identifier imported by a `use` item: `HashMap`.
            syn::UseTree::Name(use_name) => {
                if use_name.ident.eq(&self.mac_name){
                    return (Some(Self::create_path(None, self.mac_name.clone())), None )
                }
                else {

                    return ( None, Some(item_use.clone()) )
                }
            },
            
            // An renamed identifier imported by a `use` item: `HashMap as Map`.
            syn::UseTree::Rename(use_rename) => {
                if use_rename.ident.eq(&self.mac_name){
                    return (Some(Self::create_path(None, use_rename.rename)), None )
                }
                else {
                    return ( None, Some(item_use.clone()) )
                }
            },
            
            // A glob import in a `use` item: `*`.
            syn::UseTree::Glob(_) => {
                return (Some(Self::create_path(None, self.mac_name.clone())), None )
            },
            
            // A braced group of imports in a `use` item: `{A, B, C}`.
            syn::UseTree::Group(mut use_group) => {
    
                for _ in 0..use_group.items.len(){
                    if let Some(tree) = use_group.items.pop(){
    
                        let (p,t) = self.file_self_use( &tree.into_value());
                        if p.is_some(){
                            if let Some(use_tree) = t {
                                use_group.items.insert(0,use_tree);
                            } 
                            if use_group.items.len() > 0 {
                                return ( p, Some(syn::UseTree::Group(use_group)) )
                            }
                            else { return (p, None) }
                        }
                        else {
                            use_group.items.insert(0,t.unwrap());
                        }
                    }
                }
                return (None,Some(item_use.clone()));
            },
        }
    }

    pub fn create_path( mod_name: Option<syn::Ident>, mac_name: syn::Ident ) -> syn::Path {

        let tokens = if mod_name.is_some() { 
            let mod_name = mod_name.unwrap();
            quote::quote! { #mod_name :: #mac_name }
        }
        else {
            quote::quote! { #mac_name }
        };
    
        if let Ok(p) = syn::parse2(tokens.into()){
            return p
        }
        else {
            proc_macro_error::abort!( proc_macro2::Span::call_site(),"Internal Error. 'file::create_path' Could not parse path ."); 
        }
    }

}