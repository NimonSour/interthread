use crate::{attribute::{AAEdit, AAExpand}, name::get_actor_names};

use quote::quote;
use syn::{Ident,Generics};
use proc_macro2::TokenStream;
use proc_macro_error::abort;



pub struct ActorModelSdpl {
    pub name:        Ident,
    pub mac:      AAExpand,
    pub edit:       AAEdit,
    pub generics: Generics,
    pub script: (  TokenStream,  Vec<(Ident,TokenStream)>,  Vec<(Ident,TokenStream)> ),
    pub live:   (  TokenStream,  Vec<(Ident,TokenStream)>,  Vec<(Ident,TokenStream)> ),
}


impl ActorModelSdpl {

    pub fn split_edit(&mut self) -> (TokenStream,TokenStream){

        let mut edit_script_def  = None;
        let mut edit_script_mets = None;
        let mut edit_script_trts = None;
    
        let mut edit_live_def  = None;
        let mut edit_live_mets = None;
        let mut edit_live_trts = None;



        let (script,live) = 
        match &self.edit { crate::attribute::AAEdit{ script, live,..  } => {(script.clone(),live.clone())}};
        
        let diff = 
        | ((def,scope_def),mets,trts): ( (bool,bool), (Option<Vec<(syn::Ident,bool)>>,bool), (Option<Vec<(syn::Ident,bool)>>,bool) ),
          model_def:  &mut TokenStream,
          model_mets: &mut Vec<(Ident,TokenStream)>,
          model_trts: &mut Vec<(Ident,TokenStream)>,
          edit_def:   &mut Option<TokenStream>,
          edit_mets:  &mut Option<Vec<TokenStream>>,
          edit_trts:  &mut Option<Vec<TokenStream>>
        |{
            if def {
                let temp_def = Some(model_def.clone());
                *model_def  = quote!{}; 

                if scope_def{
                    *edit_def = temp_def;
                }
            }
            *edit_mets = Some(edit_select(mets,model_mets));
            *edit_trts = Some(edit_select(trts,model_trts));

        };

        diff(
            script,
             &mut self.script.0,
            &mut self.script.1,
            &mut self.script.2,
              &mut edit_script_def,
             &mut edit_script_mets,
             &mut edit_script_trts 
        );

        diff(
            live,
             &mut self.live.0,
            &mut self.live.1,
            &mut self.live.2,
              &mut edit_live_def,
             &mut edit_live_mets,
             &mut edit_live_trts 
        );
        
    
        // Prepare Token Stream Vecs
        let script_def         = &self.script.0;
        let script_methods = self.script.1.iter().map(|x| x.1.clone()).collect::<Vec<_>>();
        let script_traits  = self.script.2.iter().map(|x| x.1.clone()).collect::<Vec<_>>();

        let live_def           = &self.live.0;
        let live_methods   = self.live.1.iter().map(|x| x.1.clone()).collect::<Vec<_>>();
        let live_traits    = self.live.2.iter().map(|x| x.1.clone()).collect::<Vec<_>>();
        
        let(impl_generics,ty_generics ,where_clause) = self.generics.split_for_impl();
        let (script_name,live_name) = get_actor_names(&self.name, &self.mac);


        let res_code = quote! {
    
            #script_def
            impl #impl_generics #script_name #ty_generics #where_clause {
                #(#script_methods)*
            }
            #(#script_traits)*
    
            #live_def
            impl #impl_generics #live_name #ty_generics #where_clause {
                #(#live_methods)*
            }
            #(#live_traits)*
    
        };
    
    
        let res_edit_script_mets =  
            edit_script_mets.as_ref().map(|mets| 
                quote!{ 
                    impl #impl_generics #script_name #ty_generics #where_clause {
                        #(#mets)* 
                    }
                }
            );

        let res_edit_script_trts = 
            edit_script_trts.as_ref().map(|trts| 
                quote!{ #(#trts)* }
            );
    
        let res_edit_live_mets = 

            edit_live_mets.as_ref().map(|mets| 
                quote!{ 
                    impl #impl_generics #live_name #ty_generics #where_clause {
                        #(#mets)* 
                    }
                }
            ); 

        let res_edit_live_trts = 
        edit_live_trts.as_ref().map(|trts| 
            quote!{ #(#trts)* }
        );

        let res_edit = quote!{
    
            #edit_script_def
            #res_edit_script_mets
            #res_edit_script_trts
    
            #edit_live_def
            #res_edit_live_mets
            #res_edit_live_trts
        };
    
        (res_code, res_edit)
    
    
    }

}    

// OLD EDIT
/*

    // Create and Select Edit Parts

    let mut edit_script_def   = quote!{};
    let edit_script_mets ;
    let edit_script_trts ;

    let mut edit_live_def  = quote!{};
    let edit_live_mets ;
    let edit_live_trts ;


    match aaa.edit {

        crate::attribute::AAEdit  { live, script } => {
            match script {

                ( def , mets, trts) => {
                    if def {
                        edit_script_def = script_def.clone();
                        script_def      = quote!{}; 
                    }
                    edit_script_mets = edit_select(mets,&mut script_mets);
                    edit_script_trts = edit_select(trts,&mut script_trts);
                },
            }

            match live {

                ( def , mets, trts) => {
                    if def {
                        edit_live_def = live_def.clone();
                        live_def      = quote!{}; 
                    }
                    edit_live_mets = edit_select(mets,&mut live_mets);
                    edit_live_trts = edit_select(trts,&mut live_trts);
                },
            }
        }
    }

    // Prepare Token Stream Vecs
    let script_methods = script_mets.iter().map(|x| x.1.clone()).collect::<Vec<_>>();
    let script_traits  = script_trts.iter().map(|x| x.1.clone()).collect::<Vec<_>>();
    let live_methods   = live_mets.iter().map(|x| x.1.clone()).collect::<Vec<_>>();
    let live_traits    = live_trts.iter().map(|x| x.1.clone()).collect::<Vec<_>>();
    

    let res_code = quote! {

        

        #script_def
        impl #impl_generics #script_name #ty_generics #where_clause {
            #(#script_methods)*
        }
        #(#script_traits)*

        #live_def
        impl #impl_generics #live_name #ty_generics #where_clause {
            #(#live_methods)*
        }
        #(#live_traits)*

    };


    let res_edit_script_mets =  
    if  edit_script_mets.is_empty() { quote!{} }
    else { quote!{ 
        impl #impl_generics #script_name #ty_generics #where_clause {
            #(#edit_script_mets)* 
        }
    }};

    let res_edit_script_trts =  
    if  edit_script_trts.is_empty() { quote!{} }
    else { quote!{ #(#edit_script_trts)* }};

    let res_edit_live_mets =  
    if  edit_live_mets.is_empty() { quote!{} }
    else { quote!{ 
        impl #impl_generics #live_name #ty_generics #where_clause { 
            #(#edit_live_mets)* 
        }
    }};

    let res_edit_live_trts =  
    if  edit_live_trts.is_empty() { quote!{} }
    else { quote!{ #(#edit_live_trts)* }};


    let res_edit = quote!{

        #edit_script_def
        #res_edit_script_mets
        #res_edit_script_trts

        #edit_live_def
        #res_edit_live_mets
        #res_edit_live_trts
    };

    (res_code, res_edit)


}

*/


pub fn edit_select((edit_idents,scope): (Option<Vec<(Ident,bool)>>,bool), 
    ident_mets: &mut Vec<(Ident,TokenStream)> ) -> Vec<TokenStream> {

    let mut res = Vec::new();

    if let Some(idents) = edit_idents { 

        if idents.is_empty() {
            // let temp_ident_mets = ident_mets.clone();
            let temp_ident_mets = std::mem::replace(ident_mets,Vec::new());
            if scope {
                res = temp_ident_mets.into_iter().map(|x| x.1).collect::<Vec<_>>();
            }
            // ident_mets.clear();
        }



        for (ident,scp) in idents {
            if let Some(pos) = ident_mets.iter().position(|x| x.0 == ident){
                let (_,met)  = ident_mets.remove(pos);
                if scope || scp {
                    res.push(met);
                }
            } else {
                let msg = format!("No method named `{}` in Actor's methods.",ident.to_string());
                abort!(ident,msg);
            }
        }
    } 
    res
}

/*
V 1) Find  macros with active file 
2) Edit sort active
V 3) Clean all  active 'file' idents from macro.
4) Write to the file .


*/
