fn main() {
    if let Err(e) = enki_lib::commands::run() {
        eprintln!("ENKI failed to start: {e}");
        std::process::exit(1);
    }
}
