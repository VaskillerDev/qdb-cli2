#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate regex;
extern crate simple_logger;

mod environment;
mod text_processing;

//use crate::text_processing::m;

fn main() {
    simple_logger::init();
    //m();
}