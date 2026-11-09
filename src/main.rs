mod db;
mod ui;
mod model;
mod tag;
mod fs;

fn main() {
    if let Err(e) = db::open() {
        println!("Failed to open database: {e}");
    };
    ui::initialize();
}
