extern crate chronicle;
use chronicle::{app, graphics};
use chronicle::{CoreLoop, Game, RcCell};
use chronicle::resources::{Resource, Model};

use chronicle::Vector3;

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    let core_loop = CoreLoop::new();
    chronicle::init("Example", Example::new(), &core_loop);
    core_loop.run();
}

struct Example {
    helmet_model: Option<Resource<Model>>,
    helmet_render_model: Option<RcCell<graphics::DynamicRenderModelProperties>>
}

impl Game for Example {
    fn new() -> Box<Self> where Self: Sized {
        Box::new(Example {
            helmet_model: None,
            helmet_render_model: None
        })
    }
    
    fn start(&mut self) {
        self.helmet_model = Some(app().resources()
            .get_model(String::from("assets/models/DamagedHelmet/glTF/DamagedHelmet.gltf"))
        );

        self.helmet_render_model = Some(app().graphics()
            .create_dynamic_model(self.helmet_model.as_ref().unwrap().clone())
        );
    }

    fn update(&mut self, delta_time: f32) {
        self.helmet_render_model.as_ref().unwrap().as_mut()
            .transform.translate(&Vector3::new(0.0, 1.0 * delta_time, 0.0));
    }

    fn stop(&mut self) {
        
    }
}