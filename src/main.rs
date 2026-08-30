use core::task;
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

fn store_path() -> PathBuf {
    // Env override first, useful for testing against a throwaway file.
    if let Ok(costum) = env::var("TODO_FILE") {
        return PathBuf::from(costum);
    }

    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".todo.txt")
}

fn load(path: &Path) -> io::Result<Vec<Task>> {
    match fs::read_to_string(path) {
        // filter_map drops every None that parse returned, in one pass.
        Ok(text) => Ok(text.lines().filter_map(Task::parse).collect()),
        // Match guard: a missing file is the first run, not a failure.
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e),
    }
}

// &[Task] instead of Vec<Task>: we only read it, so borrow a slice.
fn save(path: &Path, tasks: &[Task]) -> io::Result<()> {
    let text: String = tasks.iter().map(|t| t.to_line() + "\n").collect();
    fs::write(path, text)
}

fn print_tasks(tasks: &[Task]) {
    if tasks.is_empty() {
        println!("nothing to do");
        return;
    }

    // enumerate pairs each item with its index.
    for (i, task) in tasks.iter().enumerate() {
        println!("{:>3}. {}", i + 1, task.to_line());
    }
}

fn run() -> Result<(), String> {
    let command = parse_args(env::args())?;
    let path = store_path();

    // `mut` is opt-in: without it the Vec could not be pushed to or removed from.
    let mut tasks = load(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;

    // A match is an expression, so it can produce a value.
    let changed = match command {
        Command::List => false,

        // `title` is moved out of the Command and into the new Task.
        Command::Add(title) => {
            tasks.push(Task { done: false, title });
            true
        }

        Command::Done(i) => {
            // get_mut returns Option, so an out-of-range number is handled,
            // not a panic. ok_or turns None into our error type.
            let task = tasks.get_mut(i).ok_or("no task with that number")?;
            task.done = true;
            true
        }

        Command::Remove(i) => {
            if i >= tasks.len() {
                return Err("no task with that number".into());
            }
            tasks.remove(i);
            true
        }
    };

    // Only touch the file when something actually changed.
    if changed {
        save(&path, &tasks).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    }

    print_tasks(&tasks);
    Ok(())
}

fn main() {
    if let Err(msg) = run() {
        eprintln!("todo: {msg}");
        process::exit(1);
    }
}
