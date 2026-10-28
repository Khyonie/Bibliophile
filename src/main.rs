mod db;
mod layout;
mod model;
mod tag;

fn main() {
    if let Err(e) = db::open() {
        println!("Failed to open database: {e}");
    };
    layout::initialize();
}
