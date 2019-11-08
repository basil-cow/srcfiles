use ::srcfiles::crate_srcfiles;

use std::env;
use std::process;

fn main() {
    let mut args = env::args();
    let _ = args.next(); // executable name

    let filename = match (args.next(), args.next()) {
        (Some(filename), None) => filename,
        _ => {
            eprintln!("Usage: srcfiles path/to/main/or/lib.rs");
            process::exit(1);
        }
    };

    let (srcfiles, errors) = match crate_srcfiles(filename.into()) {
        Ok(srcfiles) => (srcfiles, vec![]),
        Err(x) => x,
    };

    for i in srcfiles {
        println!("{:?}", i);
    }

    for i in errors {
        println!("{:?}", i);
    }
}
