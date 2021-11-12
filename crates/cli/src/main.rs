mod ext;
pub mod stub_symbols;

use std::{borrow::Cow, path::PathBuf};

use anyhow::Context;
use clap::Parser;

use ext::Extension;
use ext_php_rs_describe::stub::ToStub;

/// Options given as arguments when calling the CLI application.
#[derive(Parser)]
#[clap(
    about = "Generates stub files for PHP extensions generated with `ext-php-rs`.",
    author = "David Cole <david.cole1340@gmail.com>",
    version = env!("CARGO_PKG_VERSION"),
)]
struct Opts {
    /// Path to extension to generate stubs for.
    ext: Extension,
    /// Path used to store generated stub file. Defaults to writing to
    /// `<ext-name>.stubs.php` in the current directory.
    #[clap(short, long)]
    out: Option<PathBuf>,
    /// Print stubs to stdout rather than write to file. Cannot be used with `out`.
    #[clap(long, conflicts_with = "out")]
    stdout: bool,
}

fn main() -> anyhow::Result<()> {
    stub_symbols::link();

    let opts = Opts::parse();

    let result = opts.ext.ext_php_rs_describe_module();
    let stubs = result
        .to_stub()
        .with_context(|| "Failed to generate stubs.")?;

    if opts.stdout {
        print!("{}", stubs);
    } else {
        let out_path = if let Some(out_path) = &opts.out {
            Cow::Borrowed(out_path)
        } else {
            let mut cwd =
                std::env::current_dir().with_context(|| "Failed to get current working directory")?;
            cwd.push(format!("{}.stubs.php", result.name));
            Cow::Owned(cwd)
        };

        std::fs::write(out_path.as_ref(), &stubs).with_context(|| "Failed to write stubs to file")?;
    }

    Ok(())
}
