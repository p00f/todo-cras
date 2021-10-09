#![warn(clippy::all, clippy::pedantic)]

use std::cmp::Ordering;
use std::collections::HashSet;
use std::env::Args;
use std::ffi::OsString;
use std::fmt::Display;
use std::fs::read_to_string;
use std::fs::File;
use std::process::exit;
use std::{io::prelude::*, str};

use chrono::NaiveDateTime;
use home::home_dir;
use read_input::prelude::*;
use regex::Regex;
use termcolor::{Color, ColorChoice::Auto, ColorSpec, StandardStream, WriteColor};
use termion::screen::AlternateScreen;

const COLORS: [&str; 8] = [
    "Black", "Blue", "Green", "Red", "Cyan", "Magenta", "Yellow", "White",
];
const FMT: &str = "%Y-%m-%d %H:%M";

struct Category {
    name: String,
    probability: f32,
    color: Color,
}

impl Category {
    fn parse(line: &str) -> Result<Self, String> {
        let regex = Regex::new(
        r"^Category name: (?P<name>[^\t]+)\tcolor: (?P<color>[^\t]+)\tprobability: (?P<probability>\d\.\d{2})$"
        )
        .unwrap();

        let name = regex
            .captures(line)
            .unwrap()
            .name("name")
            .unwrap()
            .as_str()
            .to_owned();

        let color = parse_color(
            regex
                .captures(line)
                .unwrap()
                .name("color")
                .unwrap()
                .as_str(),
        )?;

        let probability = regex
            .captures(line)
            .unwrap()
            .name("probability")
            .unwrap()
            .as_str()
            .parse::<f32>()
            .map_err(|err| err.to_string())?;

        assert!(
            (0.0..=1.0).contains(&probability),
            "Probability {} outside 0..=1",
            probability
        );

        Ok(Self {
            name,
            probability,
            color,
        })
    }

    fn edit(&mut self) {
        let operation = get_choices(&["Change name", "Change probability", "Change color"]);
        match operation {
            1 => {
                //clear();
                self.name = input().msg("New name: ").get();
            }
            2 => {
                //clear();
                self.probability = input::<f32>()
                    .msg("New probability: ")
                    .add_err_test(|x| (&0.0..=&1.0).contains(&x), "Invalid probability")
                    .get();
            }
            3 => {
                //clear();
                println!("New color: ");
                self.color = parse_color(COLORS[get_choices(&COLORS.to_vec()) - 1]).unwrap();
                //clear();
            }
            _ => unreachable!(),
        }
    }
}

struct Task {
    task: String,
    deadline: Option<NaiveDateTime>,
    category: String,
}

impl Task {
    fn parse(line: &str, category: &str) -> Result<Self, String> {
        let regex = Regex::new(
        r#"^    Task name: (?P<name>[^\t]+)\tdeadline: "(?P<deadline>(\d{4}-\d{2}-\d{2} \d{2}:\d{2})|none)"$"#
        )
        .unwrap();

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
                    .map_err(|err| err.to_string())?,
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
        let operation = get_choices(&["Change task name", "Change deadline", "Change category"]);
        match operation {
            1 => {
                //clear();
                self.task = input().msg("New task name: ").get();
            }
            2 => {
                //clear();
                self.deadline = get_deadline("New deadline: ");
            }
            3 => {
                //clear();
                let category_names = get_category_names(categories);
                let category_index = get_choices(&category_names);
                //clear();
                self.category = category_names[category_index - 1].to_string();
            }
            _ => unreachable!(),
        }
    }
}

fn edit(categories: &mut Vec<Category>, tasks: &mut Vec<Task>) {
    match get_choices(&["Category", "Task"]) {
        1 => {
            //clear();
            edit_category(categories, tasks);
        }
        2 => {
            //clear();
            edit_task(tasks, categories);
        }
        _ => unreachable!(),
    };
}

/// # Panics
/// Will panic if
/// 1) `$TODO_FILE` is not set and `home::home_dir()` fails
/// 2) Screen cannot be flushed
pub fn run(args: &mut Args) {
    args.next();
    let file = std::env::var_os("TODO_FILE").map_or_else(
        || home_dir().unwrap().join("todo.txt").into_os_string(),
        |var| var,
    );
    if let Some(arg) = args.next() {
        match arg.as_str() {
            "-e" => {
                let mut screen = AlternateScreen::from(std::io::stdout());
                //clear();
                let (mut tasks, mut categories) = handle_err(read(&file));
                loop {
                    edit(&mut categories, &mut tasks);
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
                screen.flush().unwrap();
            }
            "-p" => {
                let (tasks, categories) = handle_err(read(&file));
                display(&categories, tasks, true);
            }
            _ => help(),
        };
    } else {
        let (tasks, categories) = handle_err(read(&file));
        display(&categories, tasks, false);
    }
}

fn save(categories: &[Category], tasks: &[Task], file: OsString) {
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
                    "    Task name: {}\tdeadline: {:?}",
                    task.task,
                    task.deadline.map_or_else(
                        || "none".to_string(),
                        |deadline| deadline.format(FMT).to_string()
                    )
                )
                .ok();
            }
        }
    }
}

fn help() {
    let help = r"Usage:
    todo_cras <no arguments>: Display tasks
              -e:             Edit tasks and categories
              -h:             Display this help";
    println!("{}", help);
}

fn display(categories: &[Category], mut tasks: Vec<Task>, probability: bool) {
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
                writeln!(
                    color_stream,
                    "    {}: {}",
                    task.deadline
                        .map_or("No deadline".to_string(), |deadline| deadline
                            .format(FMT)
                            .to_string()),
                    task.task
                )
                .ok();
            }
        }
    }
}

fn read(file: &OsString) -> Result<(Vec<Task>, Vec<Category>), String> {
    let mut categories = vec![];
    let mut tasks = vec![];
    let text = read_to_string(file).expect("Could not read file");

    let category_regex =
        Regex::new(r"^Category name: [^\t]+\tcolor: [^\t]+\tprobability: \d\.\d{2}+$").unwrap();
    let task_regex = Regex::new(
        r#"^    Task name: [^\t]+\tdeadline: "(\d{4}-\d{2}-\d{2} \d{2}:\d{2})"|"none"$"#,
    )
    .unwrap();

    for line in text.lines() {
        if category_regex.is_match(line) {
            categories.push(Category::parse(line)?);
        } else if task_regex.is_match(line) {
            tasks.push(Task::parse(line, &categories.last().unwrap().name)?);
        }
    }

    if !categories.iter().any(|c| c.name == "Unclassified") {
        categories.push(Category {
            name: "Unclassified".to_string(),
            probability: 1.00,
            color: Color::White,
        });
    }

    Ok((tasks, categories))
}

fn edit_category(categories: &mut Vec<Category>, tasks: &mut [Task]) {
    let category_names = get_category_names(categories);
    match get_choices(&["Add category", "Edit category", "Delete category"]) {
        1 => {
            //clear();
            let name = input::<String>().msg("Name: ").get();
            let probability = input::<f32>()
                .msg("Probability: ")
                .add_err_test(|x| (&0.0..=&1.0).contains(&x), "Invalid probability")
                .get();
            println!("Color: ");
            let color = parse_color(COLORS[get_choices(&COLORS.to_vec()) - 1]).unwrap();
            //clear();
            categories.push(Category {
                name,
                probability,
                color,
            });
        }
        2 => {
            //clear();
            let category = get_choices(&category_names);
            //clear();
            categories[category - 1].edit();
        }
        3 => {
            //clear();
            let category_index = get_choices(&category_names) - 1;
            if category_names[category_index] == "Unclassified" {
                let mut color_stream = StandardStream::stdout(Auto);
                color_stream
                    .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
                    .ok();
                writeln!(color_stream, "Cannot delete special category Unclassified").ok();
                exit(1);
            }
            //clear();
            for task in tasks.iter_mut() {
                if task.category == category_names[category_index] {
                    task.category = String::from("Unclassified");
                }
            }
            if !category_names.iter().any(|v| v == &"Unclassified") {
                categories.push(Category {
                    name: "Unclassified".to_string(),
                    probability: 1.00,
                    color: Color::White,
                });
            }
            categories.remove(category_index);
        }
        _ => unreachable!(),
    };
}

fn edit_task(tasks: &mut Vec<Task>, categories: &[Category]) {
    match get_choices(&["Add task", "Edit task", "Delete task"]) {
        1 => {
            //clear();
            let category_number = get_choices(&get_category_names(categories));
            //clear();
            let category = get_category_names(categories)[category_number - 1].to_string();
            let task = input::<String>().msg("Task name: ").get();
            let deadline = get_deadline("Deadline: ");
            tasks.push(Task {
                task,
                deadline,
                category,
            });
        }
        2 => {
            //clear();
            let task_names = get_task_names(tasks);
            let task_index = get_choices(&task_names) - 1;
            //clear();
            tasks[task_index].edit(categories);
        }
        3 => {
            //clear();
            let task_names = get_task_names(tasks);
            let task_index = get_choices(&task_names) - 1;
            //clear();
            tasks.remove(task_index);
        }
        _ => unreachable!(),
    };
}

fn get_choices(choices: &[&str]) -> usize {
    for (iteration, choice) in choices.iter().enumerate() {
        println!("{}: {}", iteration + 1, choice);
    }
    input::<usize>()
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
        _ => Err("Invalid color".to_string()),
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

fn handle_err<T, E: Display>(result: Result<T, E>) -> T {
    let mut red_stream = StandardStream::stdout(Auto);
    red_stream
        .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
        .ok();
    match result {
        Ok(ok) => ok,
        Err(e) => {
            writeln!(red_stream, "{}", e).ok();
            exit(1);
        }
    }
}

#[allow(dead_code)]
fn clear() {
    assert!(std::process::Command::new("cls")
        .status()
        .or_else(|_| std::process::Command::new("clear").status())
        .unwrap()
        .success());
}
