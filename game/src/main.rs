extern crate chronicle;
use chronicle::*;
use resources::{Resource, Model};
use input::{VirtualKeyCode, MouseButton};

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    let core_loop = CoreLoop::new();
    chronicle::init("Example", Example::new(), &core_loop);
    core_loop.run();
}

struct Example {
    helmet_model: Option<Resource<Model>>,
    helmet_render_model: Option<RcCell<graphics::DynamicRenderModelProperties>>,
    render_camera: Option<RcCell<graphics::RenderCameraProperties>>
}

impl Game for Example {
    fn new() -> Box<Self> where Self: Sized {
        Box::new(Example {
            helmet_model: None,
            helmet_render_model: None,
            render_camera: None
        })
    }
    
    fn start(&mut self) {
        app().input().set_cursor_mode(input::CursorMode::LOCKED);

        self.helmet_model = Some(app().resources()
            .get_model(String::from("assets/models/DamagedHelmet/glTF/DamagedHelmet.gltf"))
        );

        self.helmet_render_model = Some(app().graphics()
            .create_dynamic_model(self.helmet_model.as_ref().unwrap().clone())
        );

        self.render_camera = Some(app().graphics()
            .create_camera()
        );
        self.render_camera.as_ref().unwrap().as_mut()
            .main = true;
    }

    fn update(&mut self, delta_time: f32) {
        self.helmet_render_model.as_ref().unwrap().as_mut()
            .transform.translate(&Vector3::new(0.0, 1.0 * delta_time, 0.0));

        const SPEED: f32 = 10.0;
        let mut translation = Vector3::new(0.0, 0.0, 0.0);
        if app().input().key(VirtualKeyCode::W) {
            translation += forward() * SPEED * delta_time;
        }
        if app().input().key(VirtualKeyCode::S) {
            translation += -forward() * SPEED * delta_time;
        }
        if app().input().key(VirtualKeyCode::D) {
            translation += right() * SPEED * delta_time;
        }
        if app().input().key(VirtualKeyCode::A) {
            translation += -right() * SPEED * delta_time;
        }
        self.render_camera.as_ref().unwrap().as_mut()
            .camera.translate(&translation);

        println!("{:?}", app().input().mouse_delta());
    }

    fn stop(&mut self) {
        
    }
}