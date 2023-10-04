// use crate::attribute;

use crate::model::argument::Lib ;
use std::path::PathBuf;
use std::fs::OpenOptions;
use std::io::{self,Read,Write};


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







