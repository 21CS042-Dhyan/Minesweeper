mod gui;
mod db;

fn main() -> Result<(), eframe::Error> {
    gui::run()
}