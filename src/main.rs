use std::env;

fn main() {
    let mut args = env::args();
    args.next();

    todo_cras::run(&mut args);

    //todo_cras::read("~/todo.txt".to_string());
    //if let Some(s) = args.next() {
    //    let p = match s.as_str() {
    //        "-e" => "edit",
    //        _ => "help",
    //    };
    //    println!("{}", p);
    //} else {
    //    println!("display");
    //}
}
