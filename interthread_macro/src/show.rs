
use std::fs::OpenOptions;
use std::io::Write;

fn example_remove( path: &std::path::PathBuf) -> Result<(),String>{
    if let Err(_) = std::fs::remove_dir_all(path){
        format!(" Failed to remove directory - {:?}", path); 
    }
    Ok(())
}

fn example_create( path: &std::path::PathBuf) -> Result<(),String>{
    if let Err(_) = std::fs::create_dir(path){
        format!(" Failed to create directory - {:?}", path);
    }
    Ok(())
}

fn example_check_get() -> Result<std::path::PathBuf,String> {

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
        return Err(format!("Error: 'CARGO_MANIFEST_DIR' - Not Present"));
    }
}

pub fn example_path( file: &std::path::PathBuf ) -> Result<std::path::PathBuf,String> {

    let mut path = example_check_get()?;
    if let Some(name) = file.file_name(){
        path.push(name);
        return Ok(path)
    }
    else {
        return Err(format!("Internal Error. Function 'show::example_path' could not get file name."));
    }

}

pub fn write( val:String, path: &std::path::PathBuf ) -> Result<(), std::io::Error>{

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)?;

    write!(file, "{}", val)
}


fn write_file( file: syn::File, path: &std::path::PathBuf ) -> Result<(), std::io::Error> {

    let val = quote::quote!{ #file }.to_string();

    write(val, path )?;
    std::process::Command::new("rustfmt")
    .arg("--edition")
    .arg("2021")
    .arg(path)
    .output()?;

    Ok(())
}

pub fn example_show( file: syn::File, path: &std::path::PathBuf, lib: Option<crate::attribute::AALib> ) {

    let main_file = if lib.is_none() { None } else {  be_main(path, lib.unwrap()) };

    match example_path(path) {

        Ok( mut ex_path ) => {

            if let Err(e) = write_file( file, &ex_path ){
                proc_macro_error::abort!(proc_macro2::Span::call_site(),e);
            }
            if let Some( main_file ) = main_file {
                let file_name = crate::MAIN.to_string() + ".rs";
                ex_path.set_file_name(file_name);

                if let Err(e) = write_file( main_file, &ex_path ){
                    proc_macro_error::abort!(proc_macro2::Span::call_site(),e);
                }
            }
        },
        Err(e) => {
            proc_macro_error::abort!(proc_macro2::Span::call_site(),e);
        },
    }
}

fn be_main( path: &std::path::PathBuf ,lib: crate::attribute::AALib) ->  Option<syn::File> { //Option<( syn::File, std::path::PathBuf )>  {
    
    if let Some(stem) = path.file_stem(){
        if let Some(stem) = stem.to_str(){
            if stem == crate::MAIN {
                return None;
            }

            let main_file = crate::file::main_file(stem.clone().into(),lib);

            return Some( main_file);

        }

        let msg = "Internal Error. Inside 'show::be_main'. Could not cast OsStr to Str";
        proc_macro_error::abort!(proc_macro2::Span::call_site(),msg);

    }
    
    let msg = "Internal Error. Inside 'show::be_main'. Could not get 'file_stem' from provided path.";
    proc_macro_error::abort!(proc_macro2::Span::call_site(),msg);
    
}







