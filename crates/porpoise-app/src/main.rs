// Prevents the console window from appearing on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    porpoise_app_lib::run();
}
