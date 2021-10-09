use todo_cras::{display, edit_mode, help, ok_or_exit, read};

use home::home_dir;

fn main() {
    let mut args = std::env::args();
    args.next();

    let file = std::env::var_os("TODO_FILE").map_or_else(
        || home_dir().unwrap().join("todo.txt").into_os_string(),
        |var| var,
    );

    if let Some(arg) = args.next() {
        match arg.as_str() {
            // Edit mode.
            "-e" => {
                ok_or_exit(edit_mode(file));
            }

            // Display mode, with probability. Useful as shell greeting.
            "-p" => {
                let (tasks, categories) = ok_or_exit(read(&file));
                display(&categories, tasks, true);
            }

            // Display help if unrecognised arguments are given.
            _ => help(),
        };
    } else {
        // Display mode, without probability. Useful as command.

        let (tasks, categories) = ok_or_exit(read(&file));
        display(&categories, tasks, false);
    }
}
