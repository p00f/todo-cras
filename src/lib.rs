#![warn(clippy::all, clippy::pedantic)]

use std::cmp::Ordering;
use std::collections::HashSet;
use std::env::Args;
use std::ffi::OsString;
use std::fs::read_to_string;
use std::fs::File;
use std::{io::prelude::*, str};

use chrono::NaiveDateTime;
use home::home_dir;
use read_input::prelude::*;
use regex::Regex;
use termcolor::{Color, ColorSpec, WriteColor};
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
    fn parse(lines: &[&str]) -> Self {
        let regex = [
            Regex::new(r"Category name: (.+)").unwrap(),
            Regex::new(r"color: (.+)").unwrap(),
            Regex::new(r"probability: (.+)").unwrap(),
        ];

        let name = regex[0]
            .captures(lines[0])
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
            .to_owned();

        let color;
        let color_result = parse_color(
            regex[1]
                .captures(lines[1])
                .unwrap()
                .get(1)
                .unwrap()
                .as_str(),
        );

        if let Ok(parsed_color) = color_result {
            color = parsed_color;
        } else {
            println!("{} at {}", color_result.err().unwrap(), lines[1]);
            std::process::exit(1);
        }

        let probability = regex[2]
            .captures(lines[2])
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
            .parse::<f32>()
            .unwrap();
        Self {
            name,
            probability,
            color,
        }
    }

    fn edit(&mut self) {
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

struct Task {
    task: String,
    deadline: Option<NaiveDateTime>,
    category: String,
}

impl Task {
    fn parse(lines: &[&str], category: &str) -> Self {
        let regex = [
            Regex::new(r"^    Task name: (.+)$").unwrap(),
            Regex::new(r#"^         deadline: "(.+)"$"#).unwrap(),
        ];

        let captured_deadline = regex[1]
            .captures(lines[1])
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
            .trim();
        let mut deadline = None;
        if captured_deadline != "none" {
            deadline = Some(
                NaiveDateTime::parse_from_str(
                    regex[1]
                        .captures(lines[1])
                        .unwrap()
                        .get(1)
                        .unwrap()
                        .as_str(),
                    FMT,
                )
                .unwrap(),
            );
        }
        Self {
            task: regex[0]
                .captures(lines[0])
                .unwrap()
                .get(1)
                .unwrap()
                .as_str()
                .to_string(),
            deadline,
            category: category.to_owned(),
        }
    }

    fn edit(&mut self, categories: &[Category]) {
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
                self.category = category_names[category_index - 1].to_string();
            }
            _ => unreachable!(),
        }
    }
}

fn edit(categories: &mut Vec<Category>, tasks: &mut Vec<Task>) {
    match get_choices(&["Category", "Task"]) {
        1 => {
            clear();
            edit_category(categories, tasks);
        }
        2 => {
            clear();
            edit_task(tasks, categories);
        }
        _ => unreachable!(),
    };
}

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
                clear();
                let (mut tasks, mut categories) = read(&file);
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
                let (tasks, categories) = read(&file);
                display(&categories, tasks, true);
            }
            _ => help(),
        };
    } else {
        let (tasks, categories) = read(&file);
        display(&categories, tasks, false);
    }
}

fn save(categories: &[Category], tasks: &[Task], file: OsString) {
    let mut out = File::create(file).unwrap();
    for category in categories {
        writeln!(
            out,
            "Category name: {}\ncolor: {:?}\nprobability: {}",
            category.name, category.color, category.probability
        )
        .ok();
        for task in tasks {
            if task.category == category.name {
                writeln!(
                    out,
                    "    Task name: {}\n         deadline: {:?}",
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
        (Some(d1), Some(d2)) => d1.cmp(&d2).reverse(),
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
    let mut color_stream = termcolor::StandardStream::stdout(termcolor::ColorChoice::Auto);

    let mut has_task = HashSet::new(); // only
    for task in &tasks {
        // display
        has_task.insert(&task.category); // non-empty
    } // categories
    for category in categories                                                                // whose probability
        .iter()                                                                               // is greater than `rand`
        .filter(|category| category.probability >= rand && has_task.contains(&category.name))
    {
        color_stream
            .set_color(ColorSpec::new().set_fg(Some(category.color)))
            .ok();
        writeln!(color_stream, "{}", category.name).ok();
        for task in &tasks {
            if task.category == category.name {
                // O(tasks * categories) is fine lol
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

fn read(file: &OsString) -> (Vec<Task>, Vec<Category>) {
    let mut categories = vec![];
    let mut tasks = vec![];
    let lines = read_to_string(file).expect("Could not read file");
    let lines: Vec<&str> = lines.lines().collect();

    let category_regex = [
        Regex::new(r"^Category name: .+$").unwrap(),
        Regex::new(r"^color: .+$").unwrap(),
        Regex::new(r"^probability: .+$").unwrap(),
    ];
    let task_regex = [
        Regex::new(r"^    Task name: .+$").unwrap(),
        Regex::new(r#"^         deadline: "(\d{4}-\d{2}-\d{2} \d{2}:\d{2})"|"none"$"#).unwrap(),
    ];

    for line_num in 0..lines.len() {
        if category_regex[0].is_match(lines[line_num])
            && category_regex[1].is_match(lines[line_num + 1])
            && category_regex[2].is_match(lines[line_num + 2])
        {
            categories.push(Category::parse(&lines[line_num..(line_num + 3)]));
        } else if task_regex[0].is_match(lines[line_num])
            && task_regex[1].is_match(lines[line_num + 1])
        {
            let category = categories[categories.len() - 1].name.clone();
            tasks.push(Task::parse(&lines[line_num..(line_num + 2)], &category));
        }
    }
    if !categories.iter().any(|c| c.name == "Unclassified") {
        categories.push(Category {
            name: "Unclassified".to_string(),
            probability: 1.00,
            color: Color::White,
        });
    }
    (tasks, categories)
}

fn edit_category(categories: &mut Vec<Category>, tasks: &mut [Task]) {
    let category_names = get_category_names(categories);
    match get_choices(&["Add category", "Edit category", "Delete category"]) {
        1 => {
            clear();
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
            if category_names[category_index] == "Unclassified" {
                let mut color_stream =
                    termcolor::StandardStream::stdout(termcolor::ColorChoice::Auto);
                color_stream
                    .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
                    .ok();
                writeln!(color_stream, "Cannot delete special category Unclassified").ok();
                std::process::exit(1);
            }
            clear();
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
            clear();
            let category_number = get_choices(&get_category_names(categories));
            clear();
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
            clear();
            let task_names = get_task_names(tasks);
            let task_index = get_choices(&task_names) - 1;
            clear();
            tasks[task_index].edit(categories);
        }
        3 => {
            clear();
            let task_names = get_task_names(tasks);
            let task_index = get_choices(&task_names) - 1;
            clear();
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

fn clear() {
    assert!(std::process::Command::new("cls")
        .status()
        .or_else(|_| std::process::Command::new("clear").status())
        .unwrap()
        .success());
}
