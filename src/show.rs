// use crate::attribute;

// use crate::model::Model;
use crate::model::{ Model, Lib };
use std::path::PathBuf;
use std::fs::{self,OpenOptions};
use std::io::{self,Read,Write};
use proc_macro_error::abort;
use proc_macro::Span;


pub fn get_text (path: &PathBuf) -> io::Result<String>{
    let mut file = std::fs::File::open(path)?;
    let mut contents = Vec::new();

    file.read_to_end(&mut contents)?;
    // Convert the raw bytes to a string
    let contents_string = String::from_utf8_lossy(&contents).into_owned();
    Ok(contents_string)
}

fn example_remove( path: &PathBuf) -> Result<(),String>{
    if let Err(_) = std::fs::remove_dir_all(path){
        format!("Internal Error.'show::example_remove'. Failed to remove directory - {:?}!", path); 
    }
    Ok(())
}

fn example_create( path: &PathBuf) -> Result<(),String>{
    if let Err(_) = std::fs::create_dir(path){
        format!("Internal Error.'show::example_create'. Failed to create directory - {:?}!", path);
    }
    Ok(())
}

fn example_check_get() -> Result<PathBuf,String> {

    if let Ok(mut curr_dir) = std::env::current_dir(){

        curr_dir.push(crate::EXAMPLES);

        if !curr_dir.exists() {
            example_create( &curr_dir )?;
        }
        match std::env::var(crate::INTER_EXAMPLE_DIR_NAME){
            Ok(inter) => { curr_dir.push(inter); },
            Err(_) => {
                curr_dir.push(crate::INTER);
            },
        }
        if curr_dir.exists() {
            example_remove(&curr_dir)?;
        }
        example_create( &curr_dir )?;
        return Ok(curr_dir);
    }
    else {
        return Err(format!("Internal Error.'show::example_check_get'. 'CARGO_MANIFEST_DIR' - Not Present"));
    }
}

pub fn example_path( file_path: &PathBuf ) -> Result<PathBuf,String> {

    let mut path = example_check_get()?;
    if let Some(name) = file_path.file_name(){
        path.push(name);
        return Ok(path)
    }
    else {
        return Err(format!("Internal Error.'show::example_path'. Could not get file name!"));
    }
}

pub fn write( val:String, path: &PathBuf ) -> Result<(), std::io::Error>{

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)?;

    write!(file, "{}", val)
}

fn write_file( file: syn::File, path: &PathBuf ) -> Result<(), std::io::Error> {

    let code =  prettyplease::unparse(&file);
    write(code, path )?;
    Ok(())
}

pub fn example_show( file: syn::File, path: &PathBuf, lib: Option<Lib > ) -> PathBuf {

    let main_file = if lib.is_none() { None } else {  be_main(path, lib.unwrap()) };

    match example_path(path) {

        Ok( mut ex_path ) => {

            let rltv_path = ex_path.clone().components().rev().take(3).collect::<Vec<_>>()
                                            .into_iter().rev().collect::<PathBuf>();
            
            if let Err(e) = write_file( file, &ex_path ){
                proc_macro_error::abort!(proc_macro2::Span::call_site(),e);
            }
            if let Some( main_file ) = main_file {
                ex_path.set_file_name("main.rs");

                if let Err(e) = write_file( main_file, &ex_path ){
                    proc_macro_error::abort!(proc_macro2::Span::call_site(),e);
                }
            }
            return rltv_path;
        },
        Err(e) => {
            proc_macro_error::abort!(proc_macro2::Span::call_site(),e);
        },
    }
}

fn be_main( path: &PathBuf ,lib: Lib ) ->  Option<syn::File> { 
    
    if let Some(stem) = path.file_stem(){
        if let Some(stem) = stem.to_str(){
            if stem == "main" {
                return None;
            }
            let main_file = crate::file::main_file(stem.clone().into(),lib);
            return Some( main_file);
        }
        let msg = "Internal Error.'show::be_main'. Could not cast OsStr to Str!";
        proc_macro_error::abort!(proc_macro2::Span::call_site(),msg);
    }
    let msg = "Internal Error.'show::be_main'. Could not get 'file_stem' from provided path!";
    proc_macro_error::abort!(proc_macro2::Span::call_site(),msg);
}

    /*
        1) debut takes a directory path
        2) check if path contains:
                    a) dir 'legends'
                        `model` = actor/group.
                        `name = full_actor_cust_name  
                        `model-part` = script/live


                    b) check if 
                        legends/`model`_`name`_script.rs  
                        legends/`model`_`name`_live.rs  

                    c) if not create some 
    */


pub fn check_legend_path( model: &Model, name: &syn::Ident, path: &PathBuf ) -> (PathBuf, PathBuf) {


    // let legends_dir = path.join("legends");

    let legends_dir = if path.ends_with("legends") {
        path.to_path_buf()
    } else {
        path.join("legends")
    };

    let script_file = legends_dir.join(format!("{model}_{name}_script.txt"));
    let live_file = legends_dir.join(format!("{model}_{name}_live.txt"));

    // Create "legends" directory if it doesn't exist
    if !legends_dir.exists() {
        if let Err(_) = fs::create_dir(&legends_dir){
            abort!(Span::call_site(),"Internal Error. Failed to create `legends` directory.")
        }
    }
    
    // Check if script and live files exist, if not, create them
    if !script_file.exists() {
        if let Err(_) = fs::File::create(&script_file){
            abort!(Span::call_site(),"Internal Error. Failed to create `script` file.")
        }
    }

    if !live_file.exists() {
        if let Err(_) = fs::File::create(&live_file){
            abort!(Span::call_site(),"Internal Error. Failed to create `live` file.")
        }
    }

    (script_file, live_file)
}




// pub fn check_legend_path ( model: &str, name: &str, path: &std::path::PathBuf  ) -> (std::path::PathBuf,std::path::PathBuf) {
//     let (script_file,live_file) = 
//     ( std::path::PathBuf::from(format!("{model}_{name}_script.txt")),
//       std::path::PathBuf::from(format!("{model}_{name}_live.txt"))   );



//     // check if path exists

//     // check if contains a direcory "legends"
//     //      false ) crate a dir "legends" 
//     //      true  ) check if "legends" contains  `script_file` and `live_file`
//     //                false ) create `script_file` and `live_file`

//     // return full paths as 
//     // ( `path`/legends/`script_file`, `path`/legends/`live_file` )


// }








