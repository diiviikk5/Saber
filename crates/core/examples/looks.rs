//! Builds looks for every game in the real library and prints their colors.
//! `cargo run -p saber-core --example looks`

use saber_core::{library::Library, look, storage};

fn main() {
    let lib: Library = storage::load_or_recover(&storage::library_path());
    let dir = storage::looks_dir();
    for g in &lib.games {
        let start = std::time::Instant::now();
        match look::build(g, &dir) {
            Some(l) => println!(
                "{:<32} tint #{:06x}  shade #{:06x}  {:>4}ms",
                g.title,
                l.tint,
                l.shade,
                start.elapsed().as_millis()
            ),
            None => println!("{:<32} (no art)", g.title),
        }
    }
}
