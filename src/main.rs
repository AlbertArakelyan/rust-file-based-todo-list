use std::env;
use std::fs;
use std::io;
use std::io::ErrorKind::Other;
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
                done: false,
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
fn parse_args(mut args: env::Args) -> Result<Command, String> {
    args.next(); // first arg is the binary path, drop it

    // No verb at all -> plain `todo` means list.
    let verb = match args.next() {
        Some(v) => v,
        None => return Ok(Command::List),
    };

    // Matching on &str, so `verb` itself stays alive and usable in the arms.
    match verb.as_str() {
        "list" => Ok(Command::List),

        "add" => {
            // Everything after `add` is the title, so qouting is optional.
            let words: Vec<String> = args.collect();
            if words.is_empty() {
                return Err("add needs a title".into());
            }
            Ok(Command::Add(words.join(" ")))
        }

        // One arm can cover several patterns with `|`.
        "done" | "rm" => {
            let n: usize = args
                .next()
                .ok_or_else(|| format!("{verb} needs a task number"))?
                // parse() knows the target type from the annotation on `n`
                .parse()
                .map_err(|_| "task number must be a number".to_string())?;

            if n == 0 {
                return Err("task number start at 1".into());
            }

            // Humans count from 1, Vec counts from 0.
            if verb == done {
                Ok(Command::Done(n - 1))
            } else {
                Ok(Command::Remove(n - 1))
            }
        }

        other => Err(format!("unknown command: {other}")),
    }
}

fn main() {
    println!("Hello, world!");
}
