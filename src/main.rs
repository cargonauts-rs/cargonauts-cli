extern crate rustc_serialize;
extern crate docopt;

use std::fs;
use std::env;
use std::io::{self, Write};
use std::process::{Command, Stdio};

use docopt::Docopt;

const USAGE: &'static str = "
cargonauts command line tool.

Usage:
    cargonauts new <name>
    cargonauts (-h | --help)
    cargonauts (-v | --version)

Options:
    -h --help       Print this message.
    -v --version    Print the version number.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_name: String,
    flag_version: bool,
    cmd_new: bool,
}

fn main() {
    let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());
    if args.flag_version {
        return println!("{}", env!("CARGO_PKG_VERSION"));
    }
    if args.cmd_new {
        build_cargonauts_app(&args.arg_name).unwrap_or_else(|e| panic!("{}", e));
    }
}

const CARGONAUTS_VERSION: &str = "{ git = \"https://github.com/withoutboats/cargonauts\" }";

const DIRS: &[(&str, Option<&str>)] = &[
        ("bin", None),
        ("assets", None),
        ("clients", Some(CLIENTS)),
        ("formats", Some(FORMATS)),
        ("methods", Some(METHODS)),
        ("middleware", Some(MIDDLEWARE)),
        ("resources", Some(RESOURCES)),
        ("templates", None),
    ];

const CLIENTS: &'static str = "\
// The clients module is for defining your server's API clients.
// 
// When your server establishes a connection to another service, it is good
// practice to wrap that connection in a higher-level API. cargonauts provides
// a pair of trait - `Client` and `ConnectClient` which you can use to write
// these wrappers.
//
// These traits are also designed to support easily mocking the other service
// when testing your client, which you can do with the MockConnection type.";

const FORMATS: &'static str = "\
// The formats module is for defining custom formats for displaying your
// resources.
//
// A Format, which implements the `Format` trait, encapsulates the logic for
// displaying responses from that resource's methods as HTTP responses, and of
// translating the body of an HTTP request into the semantic type supported
// by a particular method.
//
// Usually, you will not need to define your own formats; cargonauts comes with
// several formats built-in that should be satisfactory for most use cases.";

const METHODS: &'static str = "\
// The methods module is for defining custom methods you want your resources to
// support.
//
// A method must implement the `Method` trait and either `ResourceMethod` or
// `CollectionMethod` (but not both!). Methods are themselves traits, and can
// be a bit tricky to implement correctly. Read the docs for more info.
//
// Usually, you will not need to define your own methods; cargonauts comes with
// several methods built-in that should be satisfactory for most use cases.";

const MIDDLEWARE: &'static str ="\
// The middleware module is for defining middleware you need in your server.
//
// Middleware let you wrap your endpoint in arbitrary code that can manipulate
// an HTTP service. A middleware must implement the `Middleware` trait.
";

const RESOURCES: &'static str = "\
// The resources module is for defining your application's resources. Every
// app will have many resources.
//
// Each resource must implement the `Resource` trait. For every method you've
// associated with the resource in your routing file, you must also implement
// that method for your resource.
";

const ROUTING: &'static str = "\
// This is your routing file, which contains the `routes!` macro that defines
// the surface API for your application.
//
// Every time you add a new endpoint, you will need to modify this file to
// declare how it will be used.
use cargonauts::methods::*;
use cargonauts::formats::*;

use methods::*;
use formats::*;
use resources::*;

routes! {
}
";

const LIB: &'static str = "\
#![feature(associated_consts)]

#[macro_use] extern crate cargonauts;

mod clients;
mod formats;
mod methods;
mod middleware;
mod resources;
mod routing;

pub use routing::routes;
";

fn build_cargonauts_app(name: &str) -> io::Result<()> {

    // Create a new app with cargo.
    Command::new("cargo").arg("new").arg(name)
            .stdout(Stdio::inherit()).stderr(Stdio::inherit())
            .output()?;

    let path = env::current_dir()?.join(name);
    let src_path = path.join("src");

    // Create subdirectories
    for &(dir, mod_file) in DIRS {
        let dir = src_path.join(dir);
        fs::create_dir(&dir)?;
        if let Some(content) = mod_file  {
            write!(fs::File::create(dir.join("mod.rs"))?, "{}", content)?;
        }
    }

    // Create routing file
    write!(fs::File::create(src_path.join("routing.rs"))?, "{}", ROUTING)?;

    // Create server file
    write!(fs::File::create(src_path.join("bin/server.rs"))?, "{}", server(name))?;

    // Rewrite lib.rs file
    write!(fs::File::create(src_path.join("lib.rs"))?, "{}", LIB)?;

    // Rewrite Cargo.toml
    let mut file = fs::OpenOptions::new().append(true).open(path.join("Cargo.toml"))?;
    write!(file, "cargonauts = {}", CARGONAUTS_VERSION)?;

    Ok(())
}

fn server(name: &str) -> String {
    format!("\
// This is your actual server application. By default it just runs the app you
// created with the `routes!` macro. You can edit it to have it do additional
// set-up or tear-down as necesary.

extern crate cargonauts;
extern crate {};

fn main() {{
    cargonauts::serve({}::routes).unwrap();
}}", name, name)
}

