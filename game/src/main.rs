extern crate chronicle;

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    let app = chronicle::App::new("Chronicle", 1280, 720);
    app.run();
}