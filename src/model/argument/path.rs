
use std::path::PathBuf;
use syn::{Ident,Type};
/*




*/

pub struct ActorGroup{
    ident: Ident,
    typ:    Type, 
    path: Option<PathBuf>,

}
pub struct GroupPath {
    path:  PathBuf

}
