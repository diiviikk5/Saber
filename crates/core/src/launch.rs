//! Starting games.

use crate::game::{Game, Launch};
use std::io;
use std::path::Path;
use std::process::{Child, Command};

/// What happened when we pressed play.
pub enum Started {
    /// We own the process and can time the session by waiting on it.
    Tracked(Child),
    /// Handed off to another launcher or the shell; we can't see the game.
    Handed,
}

pub fn launch(game: &Game) -> io::Result<Started> {
    match &game.launch {
        Launch::Uri { uri } => open(uri).map(|_| Started::Handed),
        Launch::Exe {
            path,
            args,
            working_dir,
        } => {
            if !is_executable(path) {
                // Shortcuts, .url files, scripts: let the shell decide.
                return open(&path.to_string_lossy()).map(|_| Started::Handed);
            }
            let dir = working_dir
                .clone()
                .or_else(|| path.parent().map(Path::to_path_buf));
            let mut cmd = Command::new(path);
            cmd.args(args);
            if let Some(dir) = dir {
                cmd.current_dir(dir);
            }
            cmd.spawn().map(Started::Tracked)
        }
    }
}

fn is_executable(path: &Path) -> bool {
    if cfg!(windows) {
        path.extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("exe"))
    } else {
        // On unix anything that isn't an app bundle or desktop file is run directly.
        !path
            .extension()
            .is_some_and(|e| e == "app" || e == "desktop")
    }
}

/// Opens a URI or file with whatever the OS associates with it.
pub fn open(target: &str) -> io::Result<()> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        // `cmd /c start` would split URIs on `&`; the URL handler doesn't.
        Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", target])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map(|_| ())
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(target).spawn().map(|_| ())
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open").arg(target).spawn().map(|_| ())
    }
}
