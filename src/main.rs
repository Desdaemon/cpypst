use std::{
    path::Path,
    process::{exit, Command},
};

enum Errors {
    NoArgs = 1,
    WrongBinary,
    CpyNoPath,
    CpyNotAPath,
    PstInvalidPath,
}

use Errors::*;

fn main() {
    let mut args = std::env::args();
    match args.next() {
        Some(bin) => {
            let path = Path::new(&bin);
            let path = path
                .file_stem()
                .expect("Invalid path")
                .to_str()
                .expect("Non utf-8 string");
            match path {
                "cpy" => cpy(args),
                "pst" => pst(args),
                _ => print_usage(WrongBinary as _),
            }
        }
        _ => print_usage(NoArgs as _),
    }
}

fn print_usage(code: i32) -> ! {
    eprintln!(
        "
cpy FILE
    Adds file/folder to the clipboard.

pst [DIR]
    Pastes file to the specified folder, or current folder."
    );
    exit(code)
}

fn cpy(mut args: std::env::Args) {
    match args.next() {
        Some(path) => {
            let path = Path::new(&path);
            if !path.exists() {
                eprintln!("error: provided file/folder does not exist.");
                exit(CpyNotAPath as _);
            }
            let canon = path.canonicalize();
            let path = canon.as_deref().unwrap_or(path);
            if let Err(err) =
                cli_clipboard::set_contents(path.to_str().expect("Non utf-8 path").to_owned())
            {
                eprintln!("warn: failed to set clipboard ({err})");
            }
        }
        _ => {
            eprintln!("error: cpy requires a path to copy.");
            exit(CpyNoPath as _)
        }
    }
}

fn copy_file(from: impl AsRef<Path>, to: impl AsRef<Path>) {
    std::fs::copy(from, to).expect("Failed to copy file");
}

#[cfg(unix)]
fn copy_dir(from: &Path, to: &Path) {
    _ = Command::new("sh")
        .arg("-c")
        .arg(&format!("cp -r {from:?} {to:?}"))
        .status();
}

#[cfg(windows)]
fn copy_dir(from: &Path, to: &Path) {
    _ = Command::new("cmd.exe")
        .arg("/C")
        .arg(&format!("robocopy /z /e {from:?} {to:?}"))
        .status();
}

fn pst(mut args: std::env::Args) {
    let dir = match args.next() {
        Some(dest) => {
            let dest = Path::new(&dest);
            if !dest.is_dir() {
                eprintln!("error: the provided path does not exist, or is not a folder.");
                exit(PstInvalidPath as _);
            }
            dest.to_owned()
        }
        _ => std::env::current_dir().expect("Failed to get current directory"),
    };

    let contents = cli_clipboard::get_contents().expect("Failed to retrieve clipboard content");
    let src = Path::new(&contents);
    if let Ok(meta) = src.metadata() {
        let dest = dir.join(src.file_name().expect("Invalid filename"));
        if meta.is_dir() {
            copy_dir(src, &dest)
        } else if meta.is_file() {
            copy_file(src, dest)
        }
    }
}
