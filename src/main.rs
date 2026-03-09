use inotify::{EventMask, Inotify, WatchMask};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

fn get_trash_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    PathBuf::from(home).join(".local/share/Trash/files")
}

fn count_items(trash_dir: &PathBuf) -> usize {
    if !trash_dir.is_dir() {
        return 0;
    }
    fs::read_dir(trash_dir)
        .map(|entries| entries.filter_map(Result::ok).count())
        .unwrap_or(0)
}

fn print_status(count: usize) {
    let (icon, tooltip, class) = if count > 0 {
        (
            format!("󰩹 {}", count),
            format!("{} items in trash", count),
            "not-empty",
        )
    } else {
        ("󰩺".to_string(), "Empty".to_string(), "empty")
    };

    let json = serde_json::json!({
        "text": icon,
        "tooltip": tooltip,
        "class": class,
    });

    println!("{}", json);
    io::stdout().flush().ok();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let trash_dir = get_trash_dir();

    // Print initial state
    print_status(count_items(&trash_dir));

    // Ensure the trash dir exists before watching
    fs::create_dir_all(&trash_dir)?;

    let mut inotify = Inotify::init()?;

    inotify.watches().add(
        &trash_dir,
        WatchMask::CREATE
            | WatchMask::DELETE
            | WatchMask::DELETE_SELF
            | WatchMask::MOVED_FROM
            | WatchMask::MOVED_TO,
    )?;

    let mut buffer = [0u8; 4096];

    loop {
        let events = inotify.read_events_blocking(&mut buffer)?;

        let mut changed = false;
        for event in events {
            if event.mask.intersects(
                EventMask::CREATE
                    | EventMask::DELETE
                    | EventMask::DELETE_SELF
                    | EventMask::MOVED_FROM
                    | EventMask::MOVED_TO,
            ) {
                changed = true;
            }
        }

        if changed {
            print_status(count_items(&trash_dir));
        }
    }
}
