//! Prints what Saber would import on this machine.
//!
//! ```text
//! cargo run -p saber-core --example scan
//! ```

use saber_core::scan;

fn main() {
    match scan::steam::steam_root() {
        Some(root) => println!("steam: {}", root.display()),
        None => println!("steam: not found"),
    }
    for game in scan::steam::scan() {
        let art = if game.cover.is_some() {
            "cover"
        } else {
            "no art"
        };
        println!("  {:<40} {:<16} {art}", game.title, game.id);
    }

    match scan::epic::manifests_dir() {
        Some(dir) => println!("epic: {}", dir.display()),
        None => println!("epic: not found"),
    }
    for game in scan::epic::scan() {
        println!("  {:<40} {}", game.title, game.id);
    }
}
