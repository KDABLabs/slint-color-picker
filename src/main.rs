pub mod ui;
pub mod app;
// pub mod debug;

use crate::app::App;

fn main() {
    let app = App::new();
    app.run().ok();
}
