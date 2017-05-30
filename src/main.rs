extern crate docopt;
extern crate heck;
extern crate rustc_serialize;

mod generator;
mod new;

use docopt::Docopt;

const USAGE: &'static str = "
cargonauts command line tool.

Usage:
    cargonauts new <name>
    cargonauts (g | generate) <generator> <name>
    cargonauts (-h | --help)
    cargonauts (-v | --version)

Options:
    -h --help       Print this message.
    -v --version    Print the version number.

Generators:
    resource        Generate a new resource.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_name: String,
    arg_generator: Option<String>,
    flag_version: bool,
    cmd_new: bool,
    cmd_generate: bool,
}

fn main() {
    let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());
    if args.flag_version {
        return println!("{}", env!("CARGO_PKG_VERSION"));
    }
    let result = if args.cmd_new {
        new::build_cargonauts_app(&args.arg_name)
    } else if args.cmd_generate {
        match &*args.arg_generator.unwrap() {
            "resource"  => generator::generate_resource(&args.arg_name),
            generator   => panic!("Unknown generator: `{}`", generator),
        }
    } else { panic!("Unknown command") };
    result.unwrap_or_else(|e| panic!("{}", e))
}
