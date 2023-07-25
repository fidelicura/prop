#![allow(unused)]


mod informer;
mod parser;

use crate::parser::get_args;
use crate::informer::File;



fn main() {
    let args = get_args();
    args
        .iter()
        .for_each(|arg| println!("{}", File::new(arg)));
}
