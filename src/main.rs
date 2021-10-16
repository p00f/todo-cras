/*
  Copyright (C) 2021 Chinmay Dalal

  This file is part of todo-cras.

  todo-cras is free software: you can redistribute it and/or modify
  it under the terms of the GNU General Public License as published by
  the Free Software Foundation, either version 3 of the License, or
  (at your option) any later version.

  todo-cras is distributed in the hope that it will be useful,
  but WITHOUT ANY WARRANTY; without even the implied warranty of
  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
  GNU General Public License for more details.

  You should have received a copy of the GNU General Public License
  along with todo-cras.  If not, see <https://www.gnu.org/licenses/>.
*/

use std::path::PathBuf;
use todo_cras::{display, edit_mode, help, read, HandleErr};

use home::home_dir;

fn main() {
    let mut args = std::env::args();
    args.next();

    let file = std::env::var_os("TODO_FILE")
        .map_or_else(|| home_dir().unwrap().join("todo.txt"), PathBuf::from);

    if let Some(arg) = args.next() {
        match arg.as_str() {
            // Edit mode.
            "-e" => {
                edit_mode(&file).ok_or_exit();
            }

            // Display mode, with probability. Useful as shell greeting.
            "-p" => {
                let (tasks, categories) = read(&file).ok_or_exit();
                display(&categories, tasks, true);
            }

            // Display help if unrecognised arguments are given.
            _ => help(),
        };
    } else {
        // Display mode, without probability. Useful as command.

        let (tasks, categories) = read(&file).ok_or_exit();
        display(&categories, tasks, false);
    }
}
