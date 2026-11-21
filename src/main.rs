//! `mmp-rs`: checks minimap icon files and lists their icons.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let (list, path) = match args.as_slice() {
        ["--list", p] => (true, *p),
        [p] if !p.starts_with('-') => (false, *p),
        _ => {
            eprintln!("usage: mmp-rs <war3map.mmp>          check: read and write back");
            eprintln!("       mmp-rs --list <war3map.mmp>   the icons, tab-separated");
            return ExitCode::FAILURE;
        }
    };
    let result = std::fs::read(path).map_err(|e| e.to_string()).and_then(|b| mmp_rs::parse(&b).map(|f| (b, f)).map_err(|e| e.to_string()));
    let (bytes, file) = match result {
        Ok(x) => x,
        Err(e) => {
            eprintln!("{path}: {e}");
            return ExitCode::FAILURE;
        }
    };
    if list {
        for i in &file.icons {
            let kind = mmp_rs::kind_name(i.kind).unwrap_or("unknown");
            println!("{kind}\t{}\t{}\t{},{},{},{}", i.x, i.y, i.red, i.green, i.blue, i.alpha);
        }
    } else if file.to_bytes().ok() != Some(bytes) {
        eprintln!("{path}: differs after writing");
        return ExitCode::FAILURE;
    } else {
        println!("{path}: {} icons", file.icons.len());
    }
    ExitCode::SUCCESS
}
