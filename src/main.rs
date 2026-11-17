#![windows_subsystem = "windows"]

mod domain;
mod platform;
mod ui;

fn main() {
    if std::env::args()
        .skip(1)
        .any(|a| a == "--version" || a == "-V" || a == "--self-check")
    {
        std::process::exit(0);
    }

    let code = ui::run_gui();
    std::process::exit(code);
}
