# todo-cras

Simple cli todo manager, inspired by [cras](https://sr.ht/~arivigo/cras/) and [jonhoo's shell greeting](https://github.com/jonhoo/configs/blob/1d472ea4bb2c43afdb87e18f42afc754b6441219/shell/.config/fish/config.fish#L216-L302).

It lets you sort tasks by category. Categories have color and probability: the category is shown in the chosen color, and in probability mode with the given probability.

Probability mode (`-p`) can be used as a shell greeting. For example, if a category's probability is 0.7 and color is cyan then it will be printed in cyan and shown 70% of the time.
When invoked without any arguments all tasks are displayed.

Deadlines are to be input as `YYYY-MM-DD hh:mm`.
Tasks whose deadline has passed will have `[BACKLOG]` appended to them.

![alt text](https://git.sr.ht/~p00f/todo-cras/blob/main/screenshot.png)

If `$TODO_FILE` is set then that file is used, otherwise `$HOME/todo.txt` is used.

Installation:
 - Release
 `cargo install todo-cras`
 - Current
 `cargo install --git https://git.sr.ht/~p00f/todo-cras` or clone this repo and `cargo build --release`

Usage:
```
todo-cras <no-arguments>: Display all tasks
          -p            : Display tasks according to probability
          -e            : Edit your todo list
          -h            : Display help
```
