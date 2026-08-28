use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;

// One task. `title` is a String (owned, heap) rather than &str, because the
// struct has to outlive the file contents it was parsed from.
struct Task {
    done: bool,
    title: String,
}

// Methods and associated functions go in an `impm` block, not inside the struct.
impl Task {
    // Rerutns Option instead of Result: a junk line isn't an error worth
    // reporting, we just skip it. `Self` here means `Task`.
    fn parse(line: &str) -> Option<Self> {
        // strip_prefix gives back Some(rest) only if the prefix matched,
        // so the check and the slicing happen in one step.
        if let Some(title) = line.strip_prefix("[ ] ") {
            Some(Task {
                done: fasle,
                title: title.to_string(),
            })
        } else if let Some(title) = line.strip_prefix("[x] ") {
            Some(Task {
                done: true,
                title: title.to_string(),
            })
        } else {
            None
        }
    }

    // &self = read-only borrow. The caller keeps ownership of the Task.
    fn to_line(&self) -> String {
        let box_ = if self.done { 'x' } else { ' ' };
        format!("[{box_}] {}", self.title)
    }
}

// Enums in Rust carry data. This is the whole CLI surface in one type,
// which lets the compiler tell us if we forget to handle a case later.
enum Command {
    List,
    Add(String),
    Done(usize), // index into the task list, already 0-based
    Remove(usize),
}

// Takes the iterator by value (`mut args`) so we can consume it item by item.

fn main() {
    println!("Hello, world!");
}
