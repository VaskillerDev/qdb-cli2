#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate qdb_ast;
extern crate simple_logger;

mod environment;

use crate::environment::about::build_clap_app;
use qdb_ast::parser::states::DefaultParser;
use std::io;
use std::io::BufRead;

fn main() {
    simple_logger::init();
    let args = build_clap_app();
    let run = args
        .value_of("run")
        .unwrap_or("false")
        .to_lowercase()
        .parse::<bool>().unwrap();
    // todo: continue creation after written qdb-core
    if run {
        println!("{} {} \n >",env!("CARGO_PKG_NAME"),env!("CARGO_PKG_VERSION"));
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let a = DefaultParser::parse_from_string(line.unwrap());
            println!("{:?}",a);
        }
    }

}
