use std::{
    collections::HashMap,
    error::Error,
    fmt::Display,
    ops::Deref,
    path::{Path, PathBuf, absolute},
    process,
    sync::{LazyLock, Mutex},
    time::{Duration, Instant},
};

#[derive(Debug)]
pub struct StringError(String);
impl Display for StringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl From<&str> for StringError {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}
impl From<&str> for Box<StringError> {
    fn from(value: &str) -> Self {
        Box::new(StringError(value.to_string()))
    }
}
impl Error for StringError {}

use clap::Parser;
use notify::{EventKind, RecursiveMode};
use notify_debouncer_full::{DebounceEventResult, DebouncedEvent, new_debouncer};

use crate::args::Args;

mod args;

static ARGS: LazyLock<Args> = LazyLock::new(|| {
    let args = Args::parse();
    if !args.folder.exists() || !args.folder.is_dir() {
        panic!("Invalid folder")
    }
    args
});
#[derive(Hash)]
struct AbsolutePath(PathBuf);
impl Deref for AbsolutePath {
    type Target = PathBuf;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl PartialEq for AbsolutePath {
    fn eq(&self, other: &Self) -> bool {
        if let Ok(path) = absolute(&self.0)
            && let Ok(other) = absolute(&other.0)
        {
            path == other
        } else {
            self.0 == other.0
        }
    }
}
impl Eq for AbsolutePath {}

fn main() {
    let mut watcher = new_debouncer(
        Duration::from_millis(5),
        None,
        |res: DebounceEventResult| {
            if let Ok(events) = res
                && let Some(event) = events.into_iter().next()
            {
                if event.kind.is_remove()
                    && let Some(path) = event.paths.first()
                    && absolute(path).unwrap() == absolute(&ARGS.folder).unwrap()
                {
                    eprintln!("Folder deleted. Exiting");
                    std::process::exit(1)
                }
                handle(event).unwrap_or_else(|err| eprintln!("Event handler failed! {err:#?}"));
            }
        },
    )
    .unwrap();
    watcher
        .watch(&ARGS.folder, RecursiveMode::NonRecursive)
        .unwrap();

    // Sleep forever so the watcher stays alive
    std::thread::park();
    drop(watcher);
}

fn handle(event: DebouncedEvent) -> Result<(), Box<dyn Error>> {
    let path = event
        .paths
        .first()
        .ok_or(Box::new(StringError("not found".into())))?;
    if !(event.kind.is_create()
        || event.kind.is_remove()
        || event.kind
            == EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Any,
            )))
        || event.kind
            == EventKind::Modify(notify::event::ModifyKind::Name(
                notify::event::RenameMode::Any,
            ))
    {
        return Ok(());
    }
    {
        if let Some(command) = &ARGS.on_add
            && event.kind.is_create()
        {
            println!("Executing {command} {path:?}");
            let _ = process::Command::new(command).arg(path).spawn()?.wait()?;
        } else if let Some(command) = &ARGS.on_modify
            && event.kind.is_modify()
        {
            println!("Executing {command} {path:?}");
            let _ = process::Command::new(command).arg(path).spawn()?.wait()?;
        } else if let Some(command) = &ARGS.on_remove
            && event.kind.is_remove()
        {
            println!("Executing {command} {path:?}");
            let _ = process::Command::new(command).arg(path).spawn()?.wait()?;
        } else {
            println!("{:?} @ {:?}", event.kind, path);
        }
    }
    Ok(())
}
