extern crate clap;
use clap::*;

fn build_clap_app() -> ArgMatches<'static> {
    /* APP INFO */
    // App info
    let app = App::new("qdb-cli")
        .version("1.0.1")
        .about("Qdb CLI client")
        .author("VaskillerDev Vaskoooo9241@gmail.com");

    /* COMMANDS */
    // Debug mode
    let debug = Arg::with_name("debug")
        .short("D")
        .long("debug")
        .help("(bool) Set debug mode.\n Example: qdb --debug true \n qdb -D true")
        .takes_value(true)
        .value_name("BOOL")
        .default_value("false");

    // URI address
    let uri = Arg::with_name("uri")
        .short("U")
        .long("uri")
        .help("(string) Set uri for connection. \n Example: qdb --uri \"example.com:6060/mynode\" \n qdb -U \"example.com:6060/mynode\"")
        .takes_value(true)
        .value_name("URI")
        .default_value("localhost:6060");

    app.arg(debug).arg(uri).get_matches()
}

lazy_static! {
    static ref ARG_MATCHES: ArgMatches<'static> = build_clap_app();
}

pub fn get_app_config() -> &'static ArgMatches<'static> {
    &*ARG_MATCHES
}
