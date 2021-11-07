mod validators;

use std::path::PathBuf;

use clap::Parser;

use dlopen::wrapper::{Container, WrapperApi};
use dlopen_derive::WrapperApi;
use ext_php_rs_cli::link;
use ext_php_rs_describe::{stub::ToStub, Module};

#[derive(Debug, Parser)]
struct Opts {
    #[clap(validator = validators::is_file)]
    ext: PathBuf,
}

#[derive(WrapperApi)]
struct Extension {
    ext_php_rs_describe_module: fn() -> Module,
}

fn main() {
    link();

    let ops = Opts::parse();
    let ext: Container<Extension> = unsafe { Container::load(&ops.ext) }.unwrap();
    let result = ext.ext_php_rs_describe_module();
    dbg!(&result);
    let stubs = result.to_stub().unwrap();
    println!("{}", stubs);
}
