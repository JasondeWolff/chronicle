extern crate chronicle;
use chronicle::{CoreLoop, Game};
use chronicle::{app, app_mut};

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    let core_loop = CoreLoop::new();
    chronicle::init("Example", Example::new(), &core_loop);
    core_loop.run();
}

struct Example {

}

impl Game for Example {
    fn new() -> Box<Self> where Self: Sized {
        Box::new(Example {

        })
    }
    
    fn start(&mut self) {
        
    }

    fn update(&mut self, delta_time: f32) {
        
    }

    fn stop(&mut self) {
        
    }
}