use std::env::Args;
use std::fs::read_to_string;
use std::{
    io::{self, prelude::*},
    str,
};

use chrono::{DateTime, FixedOffset};
use read_input::prelude::*;
use regex::Regex;
use termcolor::Color;

const COLORS: [&str; 8] = [
    "Black", "Blue", "Green", "Red", "Cyan", "Magenta", "Yellow", "White",
];
const FMT: &str = "%Y %b %d %H:%M +0000";

struct Category {
    name: String,
    probability: f32,
    color: Color,
}

impl Category {
    fn parse(line: &str) -> Self {
        let category_regex = Regex::new(
            r"^Category name: (?P<name>).+ color: (?P<color>).+ probability: (?P<probability>)\d{1,3}$",
        ).unwrap();
        let caps = category_regex.captures(line).unwrap();

        let mut color;
        let color_result = parse_color(caps.name("color").unwrap().as_str());
        if let Ok(parsed_color) = color_result {
            color = parsed_color;
        } else {
            panic!("{} at {}", color_result.err().unwrap(), line);
        }

        Self {
            name: caps.name("name").unwrap().as_str().to_string(),
            probability: caps
                .name("probability")
                .unwrap()
                .as_str()
                .parse::<f32>()
                .unwrap(),
            color,
        }
    }

    fn edit(&mut self) {
        let operation = get_choices(&vec!["Change name", "Change probability", "Change color"]);
        match operation {
            1 => {
                self.name = input().msg("New name: ").get();
            }
            2 => {
                self.probability = input::<f32>()
                    .msg("New probability: ")
                    .add_err_test(|x| x >= &0.0 && x <= &1.0, "Invalid probability")
                    .get();
            }
            3 => {
                println!("New color: ");
                self.color = parse_color(COLORS[get_choices(&COLORS.to_vec()) - 1]).unwrap();
            }
            _ => unreachable!(),
        }
    }
}

struct Task {
    task: String,
    deadline: DateTime<FixedOffset>,
    category: String,
}

impl Task {
    fn parse(line: &str, category: String) -> Self {
        let task_regex = Regex::new(
            r"^    Task name: (?P<name>).+ deadline: (?P<deadline>)\d{4}-\d{2}-\d{2} \d{2}:\d{2}$",
        )
        .unwrap();

        let caps = task_regex.captures(line).unwrap();
        Task {
            task: caps.name("name").unwrap().as_str().to_string(),
            deadline: DateTime::parse_from_str(caps.name("deadline").unwrap().as_str(), FMT)
                .unwrap(),
            category,
        }
    }

    fn edit(&mut self) {
        let operation = get_choices(&vec!["Change task name", "Change deadline"]);
        match operation {
            1 => {
                self.task = input().msg("New task name: ").get();
            }
            2 => {
                self.deadline = DateTime::parse_from_str(
                    input::<String>()
                        .msg("New deadline")
                        .add_err_test(
                            |x| DateTime::parse_from_str(x.as_str(), FMT).is_ok(),
                            "Invalid deadline",
                        )
                        .get()
                        .as_str(),
                    FMT,
                )
                .unwrap();
            }
            _ => unreachable!(),
        }
    }
}

fn edit(categories: &mut Vec<Category>, tasks: &mut Vec<Task>) {
    match get_choices(&vec!["Category", "Task"]) {
        1 => category(categories, tasks),
        2 => task(tasks),
        _ => unreachable!(),
    };
}

pub fn run(args: &mut Args) {
    if let Some(arg) = args.next() {
        match arg.as_str() {
            "-e" => {
                let (mut tasks, mut categories) = read("~/todo.txt".to_string());
                edit(&mut categories, &mut tasks);
            }
            _ => help(),
        };
    } else {
        let (tasks, categories) = read("~/todo.txt".to_string());
        display(categories, tasks);
    }
}

fn help() {
    let help = r"Usage:
    todo_cras <no arguments>: Display tasks
              -e:             Edit tasks and categories
              -h:             Display this help";
    println!("{}", help);
}

fn display(categories: Vec<Category>, mut tasks: Vec<Task>) {
    tasks.sort_by(|t1, t2| t1.category.cmp(&t2.category));
    let rand = fastrand::f32();
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    for category in categories
        .iter()
        .filter(|category| category.probability >= rand)
    {
        writeln!(out, "Category: {}", category.name).ok();
        for task in &tasks {
            if task.category == category.name {
                // O(tasks*categories) is fine lol
                writeln!(out, "    {}: {}", task.deadline, task.task).ok();
            }
        }
    }
}

fn read(file: String) -> (Vec<Task>, Vec<Category>) {
    let mut categories: Vec<Category> = vec![];
    let mut tasks = vec![];
    let lines = read_to_string(file).expect("Could not read file");
    let lines = lines.lines();

    let category_regex = Regex::new(
        r"^Category name: (?P<name>).+ color: (?P<color>).+ probability: (?P<probability>)\d{1,3}$",
    )
    .unwrap();
    let task_regex = Regex::new(
        r"^    Task name: (?P<name>).+ deadline: (?P<deadline>)\d{4}-\d{2}-\d{2} \d{2}:\d{2}$",
    )
    .unwrap();
    let empty_regex = Regex::new(r"^$").unwrap();

    for line in lines {
        if category_regex.is_match(line) {
            categories.push(Category::parse(line));
        } else if task_regex.is_match(line) {
            let category = categories[categories.len() - 1].name.to_owned();
            tasks.push(Task::parse(line, category));
        } else if !empty_regex.is_match(line) {
            panic!("Malformed file at {}", line);
        }
    }
    (tasks, categories)
}

fn category(categories: &mut Vec<Category>, tasks: &mut Vec<Task>) {
    let category_names = {
        let mut v: Vec<&str> = Vec::with_capacity(categories.len());
        for category in categories.iter() {
            v.push(category.name.as_str());
        }
        v
    };
    match get_choices(&vec!["Add category", "Edit category", "Delete category"]) {
        0 => {
            let name = input::<String>().msg("Name: ").get();
            let probability = input::<f32>()
                .msg("Probability: ")
                .add_err_test(|x| x >= &0.0 && x <= &1.0, "Invalid probability")
                .get();
            println!("Color: ");
            let color = parse_color(COLORS[get_choices(&COLORS.to_vec()) - 1]).unwrap();
            categories.push(Category {
                name,
                probability,
                color,
            });
        }
        1 => {
            let category = get_choices(&category_names);
            categories[category - 1].edit();
        }
        2 => {
            let category_index = get_choices(&category_names) - 1;
            for task in tasks.into_iter() {
                if task.category == category_names[category_index] {
                    task.category = String::from("Unclassified");
                }
            }
            categories.remove(category_index);
        }
        _ => unreachable!(),
    };
}

fn task(tasks: &mut Vec<Task>) {
    match get_choices(&vec!["Add task", "Edit task", "Delete task"]) {
        0 => {}
        1 => {}
        2 => {}
        _ => unreachable!(),
    };
}

fn get_choices(choices: &Vec<&str>) -> usize {
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
