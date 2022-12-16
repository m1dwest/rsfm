fn main() {
    if let Err(e) = rsfm::run() {
        eprintln!("Application error: {e}");
        std::process::exit(1);
    }
}
