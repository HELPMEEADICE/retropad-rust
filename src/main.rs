#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod win32;
mod resource_ids;
mod file_io;
mod search;
mod app;

fn main() {
    std::process::exit(app::run_app());
}
