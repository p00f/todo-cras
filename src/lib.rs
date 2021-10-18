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

#![warn(clippy::all, clippy::pedantic)]

use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt::Display;
use std::fs::read_to_string;
use std::fs::File;
use std::path::Path;
use std::process::exit;
use std::{io::prelude::*, str};

use chrono::NaiveDateTime;
use read_input::prelude::*;
use regex::Regex;
use termcolor::{Color, ColorChoice::Auto, ColorSpec, StandardStream, WriteColor};
use termion::screen::AlternateScreen;

const COLORS: [&str; 8] = [
    "Black", "Blue", "Green", "Red", "Cyan", "Magenta", "Yellow", "White",
];
const FMT: &str = "%Y-%m-%d %H:%M";

pub struct Category {
    name: String,
    probability: f32,
    color: Color,
}

impl Category {
    fn parse(line: &str) -> Result<Self, String> {
        let regex = Regex::new(
        r"^Category name: (?P<name>[^\t]+)\tcolor: (?P<color>[^\t]+)\tprobability: (?P<probability>.+)$"
        )
        .unwrap();

        let name = regex.captures(line).unwrap().name("name").unwrap().as_str();

        let color = parse_color(
            regex
                .captures(line)
                .unwrap()
                .name("color")
                .unwrap()
                .as_str(),
        )
        .map_err(|err| format!("{} at\n{}", err, line))?;

        let probability = regex
            .captures(line)
            .unwrap()
            .name("probability")
            .unwrap()
            .as_str()
            .parse::<f32>()
            .map_err(|err| format!("Could not parse probability: {} at\n{}", err, line))?;

        if !(0.0..=1.0).contains(&probability) {
            return Err(format!("Probability {} outside 0..=1", probability));
        }

        Ok(Self {
            name: String::from(name),
            probability,
            color,
        })
    }

    fn edit(&mut self) {
        color_print(Color::Yellow, &format!("Editing category '{}'", &self.name));

        let operation = get_choices(&["Change name", "Change probability", "Change color"]);
        match operation {
            1 => {
                clear();
                self.name = input().msg("New name: ").get();
            }
            2 => {
                clear();
                self.probability = input::<f32>()
                    .msg("New probability: ")
                    .add_err_test(|x| (&0.0..=&1.0).contains(&x), "Invalid probability")
                    .get();
            }
            3 => {
                clear();
                println!("New color: ");
                self.color = parse_color(COLORS[get_choices(&COLORS.to_vec()) - 1]).unwrap();
                clear();
            }
            _ => unreachable!(),
        }
    }
}

pub struct Task {
    task: String,
    deadline: Option<NaiveDateTime>,
    category: String,
}

impl Task {
    fn parse(line: &str, category: &str) -> Result<Self, String> {
        let regex =
            Regex::new(r"^    Task name: (?P<name>[^\t]+)\tdeadline: (?P<deadline>.+)$").unwrap();

        let captured_deadline = regex
            .captures(line)
            .unwrap()
            .name("deadline")
            .unwrap()
            .as_str()
            .trim();

        let deadline = match captured_deadline {
            "none" => None,
            _ => Some(
                NaiveDateTime::parse_from_str(captured_deadline, FMT)
                    .map_err(|err| format!("Could not parse deadline: {} at\n{}", err, line))?,
            ),
        };

        Ok(Self {
            task: regex
                .captures(line)
                .unwrap()
                .name("name")
                .unwrap()
                .as_str()
                .to_string(),
            deadline,
            category: String::from(category),
        })
    }

    fn edit(&mut self, categories: &[Category]) {
        color_print(Color::Yellow, &format!("Editing task '{}'", &self.task));
        let operation = get_choices(&["Change task name", "Change deadline", "Change category"]);
        match operation {
            1 => {
                clear();
                self.task = input().msg("New task name: ").get();
            }
            2 => {
                clear();
                self.deadline = get_deadline("New deadline: ");
            }
            3 => {
                clear();
                let category_names = get_category_names(categories);
                let category_index = get_choices(&category_names);
                clear();
                self.category = String::from(category_names[category_index - 1]);
            }
            _ => unreachable!(),
        }
    }
}

/// # Errors
/// Returns an error when the file has invalid syntax
#[allow(clippy::missing_panics_doc)]
pub fn read(file: &Path) -> Result<(Vec<Task>, Vec<Category>), String> {
    let mut categories = vec![];
    let mut tasks = vec![];
    let text = read_to_string(file).expect("Could not read file");

    let category_regex =
        Regex::new(r"^Category name: [^\t]+\tcolor: [^\t]+\tprobability: .+$").unwrap();
    let task_regex = Regex::new(r"^    Task name: [^\t]+\tdeadline: .+$").unwrap();

    for line in text.lines() {
        if category_regex.is_match(line) {
            categories.push(Category::parse(line)?);
        } else if task_regex.is_match(line) {
            tasks.push(Task::parse(line, &categories.last().unwrap().name)?);
        } else {
            let mut color_stream = StandardStream::stdout(Auto);
            color_stream
                .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
                .ok();
            writeln!(color_stream, "Invalid format at {}", line).ok();
        }
    }

    if !categories.iter().any(|c| c.name == "Unclassified") {
        categories.push(Category {
            name: String::from("Unclassified"),
            probability: 1.00,
            color: Color::White,
        });
    }

    Ok((tasks, categories))
}

pub fn display(categories: &[Category], mut tasks: Vec<Task>, probability: bool) {
    tasks.sort_by(|t1, t2| match (t1.deadline, t2.deadline) {
        (Some(d1), Some(d2)) => d1.cmp(&d2),
        (Some(_d1), None) => Ordering::Less,
        (None, Some(_d2)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    });

    let rand = {
        if probability {
            fastrand::f32()
        } else {
            0.00
        }
    };

    let mut color_stream = StandardStream::stdout(Auto);

    let mut has_task = HashSet::new();
    for task in &tasks {
        has_task.insert(&task.category);
    }

    for category in categories
        .iter()
        .filter(|category| category.probability >= rand && has_task.contains(&category.name))
    {
        color_stream
            .set_color(ColorSpec::new().set_fg(Some(category.color)))
            .ok();
        writeln!(color_stream, "{}", category.name).ok();

        for task in &tasks {
            if task.category == category.name {
                let mut task_name_str = task.task.clone();
                if let Some(deadline) = task.deadline {
                    if chrono::Local::now().naive_local() > deadline {
                        task_name_str.push_str(" [BACKLOG]");
                    }
                }

                writeln!(
                    color_stream,
                    "    {}: {}",
                    task.deadline
                        .map_or(String::from("No deadline"), |deadline| deadline
                            .format(FMT)
                            .to_string()),
                    task_name_str
                )
                .ok();
            }
        }
    }
}

/// # Errors
/// Returns errors when
/// 1) File has invalid syntax
/// 2) Alternate buffer can't be flushed
pub fn edit_mode(file: &Path) -> Result<(), String> {
    let mut screen = AlternateScreen::from(std::io::stdout());
    clear();
    let (mut tasks, mut categories) = read(file)?;
    loop {
        match get_choices(&["Category", "Task"]) {
            1 => {
                clear();
                edit_categories(&mut categories, &mut tasks);
            }
            2 => {
                clear();
                edit_tasks(&mut tasks, &categories);
            }
            _ => unreachable!(),
        }

        let cont = input::<String>()
            .msg("Continue editing? [y/n] ")
            .add_err_test(
                |str| str.as_str() == "y" || str.as_str() == "n",
                "Please enter y or n",
            )
            .get();
        if cont == "n" {
            break;
        }
    }
    save(&categories, &tasks, file);
    screen.flush().map_err(|err| err.to_string())?;
    Ok(())
}

fn edit_categories(categories: &mut Vec<Category>, tasks: &mut [Task]) {
    let category_names = get_category_names(categories);
    match get_choices(&["Add category", "Edit category", "Delete category"]) {
        1 => {
            clear();
            color_print(Color::Green, "Adding category");
            let name = input::<String>().msg("Name: ").get();
            let probability = input::<f32>()
                .msg("Probability: ")
                .add_err_test(|x| (&0.0..=&1.0).contains(&x), "Invalid probability")
                .get();
            println!("Color: ");
            let color = parse_color(COLORS[get_choices(&COLORS.to_vec()) - 1]).unwrap();
            clear();
            categories.push(Category {
                name,
                probability,
                color,
            });
        }

        2 => {
            clear();
            let category = get_choices(&category_names);
            clear();
            categories[category - 1].edit();
        }

        3 => {
            clear();
            let category_index = get_choices(&category_names) - 1;
            let category_name = category_names[category_index];

            if category_name == "Unclassified" {
                let mut red_stream = StandardStream::stdout(Auto);
                red_stream
                    .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
                    .ok();
                writeln!(red_stream, "Cannot delete special category Unclassified").ok();
                exit(1);
            }

            clear();
            color_print(Color::Red, &format!("Removing category '{}'. All tasks in this category will be moved to 'Unclassified'", category_name));

            for task in tasks.iter_mut() {
                if task.category == category_name {
                    task.category = String::from("Unclassified");
                }
            }

            if !category_names.iter().any(|v| v == &"Unclassified") {
                categories.push(Category {
                    name: String::from("Unclassified"),
                    probability: 1.00,
                    color: Color::White,
                });
            }
            categories.remove(category_index);
        }
        _ => unreachable!(),
    };
}

fn edit_tasks(tasks: &mut Vec<Task>, categories: &[Category]) {
    match get_choices(&["Add task", "Edit task", "Delete task"]) {
        1 => {
            clear();
            let category_number = get_choices(&get_category_names(categories));
            clear();
            let category = String::from(get_category_names(categories)[category_number - 1]);
            color_print(
                Color::Green,
                &format!("Adding task to category '{}'", category),
            );
            let task = input::<String>().msg("Task name: ").get();
            let deadline = get_deadline("Deadline: ");
            tasks.push(Task {
                task,
                deadline,
                category,
            });
        }

        2 => {
            clear();
            let task_names = get_task_names(tasks);
            let task_index = get_choices(&task_names) - 1;
            clear();
            tasks[task_index].edit(categories);
        }

        3 => {
            clear();
            color_print(Color::Red, "Deleting the task you choose");
            let task_names = get_task_names(tasks);
            let task_index = get_choices(&task_names) - 1;
            clear();
            tasks.remove(task_index);
        }

        _ => unreachable!(),
    };
}

/// # Panics
/// Panics when the file cannot be opened in write mode
fn save(categories: &[Category], tasks: &[Task], file: &Path) {
    let mut out = File::create(file).unwrap();
    for category in categories {
        writeln!(
            out,
            "Category name: {}\tcolor: {:?}\tprobability: {:.2}",
            category.name, category.color, category.probability
        )
        .ok();
        for task in tasks {
            if task.category == category.name {
                writeln!(
                    out,
                    "    Task name: {}\tdeadline: {}",
                    task.task,
                    task.deadline.map_or_else(
                        || String::from("none"),
                        |deadline| deadline.format(FMT).to_string()
                    )
                )
                .ok();
            }
        }
    }
}

pub fn help() {
    let help = r"Usage:
    todo_cras <no arguments>: Display all tasks
              -p:             Display tasks according to probability
              -e:             Edit tasks and categories
              -h:             Display this help";
    println!("{}", help);
}

// Helpers

fn get_choices(choices: &[&str]) -> usize {
    for (iteration, choice) in choices.iter().enumerate() {
        println!("{}: {}", iteration + 1, choice);
    }
    input::<usize>()
        .msg(format!("Your choice [{}-{}]: ", 1, choices.len()))
        .inside_err(1..=choices.len(), "Invalid choice")
        .get()
}

fn parse_color(color: &str) -> Result<Color, String> {
    match color.to_lowercase().as_str() {
        "black" => Ok(Color::Black),
        "blue" => Ok(Color::Blue),
        "green" => Ok(Color::Green),
        "red" => Ok(Color::Red),
        "cyan" => Ok(Color::Cyan),
        "magenta" => Ok(Color::Magenta),
        "yellow" => Ok(Color::Yellow),
        "white" => Ok(Color::White),
        _ => Err(format!("Invalid color {}", color)),
    }
}

fn get_category_names(categories: &[Category]) -> Vec<&str> {
    let mut v: Vec<&str> = Vec::with_capacity(categories.len());
    for category in categories.iter() {
        v.push(category.name.as_str());
    }
    v
}

fn get_task_names(tasks: &[Task]) -> Vec<&str> {
    let mut v: Vec<&str> = Vec::with_capacity(tasks.len());
    for category in tasks.iter() {
        v.push(category.task.as_str());
    }
    v
}

fn get_deadline(msg: &str) -> Option<NaiveDateTime> {
    let input = input::<String>()
        .msg(msg)
        .add_err_test(
            |x| NaiveDateTime::parse_from_str(x.as_str().trim(), FMT).is_ok() || x.trim() == "",
            "Invalid deadline",
        )
        .get();

    if input.as_str().trim() == "" {
        None
    } else {
        Some(NaiveDateTime::parse_from_str(input.as_str(), FMT).unwrap())
    }
}

fn clear() {
    assert!(std::process::Command::new("cls")
        .status()
        .or_else(|_| std::process::Command::new("clear").status())
        .unwrap()
        .success());
}

pub trait HandleErr {
    type Inner;
    fn ok_or_exit(self) -> Self::Inner;
}

impl<T, E: Display> HandleErr for Result<T, E> {
    type Inner = T;
    fn ok_or_exit(self) -> T {
        self.unwrap_or_else(|err| {
            color_print(Color::Red, &err.to_string());
            exit(1);
        })
    }
}

fn color_print(color: Color, text: &str) {
    let mut color_stream = StandardStream::stdout(Auto);
    color_stream
        .set_color(ColorSpec::new().set_fg(Some(color)))
        .ok();
    writeln!(color_stream, "{}", text).ok();
    color_stream
        .set_color(ColorSpec::new().set_fg(Some(Color::White)))
        .ok();
}
