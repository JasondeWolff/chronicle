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
    helmet_render_models: Vec<RcCell<graphics::DynamicRenderModelProperties>>,
    render_camera: Option<RcCell<graphics::RenderCameraProperties>>
}

impl Game for Example {
    fn new() -> Box<Self> where Self: Sized {
        Box::new(Example {
            helmet_model: None,
            helmet_render_models: Vec::new(),
            render_camera: None
        })
    }
    
    fn start(&mut self) {
        app().input().set_cursor_mode(input::CursorMode::LOCKED);

        self.helmet_model = Some(app().resources()
            .get_model(String::from("assets/models/DamagedHelmet/glTF/DamagedHelmet.gltf"))
        );

        for x in 0..10 {
            for y in 0..10 {
                let dyn_render_model = app().graphics()
                    .create_dynamic_model(self.helmet_model.as_ref().unwrap().clone());

                let translation = Vector3::new(x as f32 * 2.0, y as f32 * 2.0, 0.0);
                dyn_render_model.as_mut().transform.set_translation(&translation);

                self.helmet_render_models.push(dyn_render_model);
            }
        }

        self.render_camera = Some(app().graphics()
            .create_camera()
        );
        self.render_camera.as_ref().unwrap().as_mut()
            .main = true;
    }

    fn update(&mut self, delta_time: f32) {
        // for (i, helmet_render_model) in self.helmet_render_models.iter().enumerate() {
        //     helmet_render_model.as_mut()
        //         .transform.translate(&Vector3::new(0.0, 1.0 * delta_time * i as f32, 0.0));
        // }

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
        if app().input().key(VirtualKeyCode::Q) {
            translation += up() * SPEED * delta_time;
        }
        if app().input().key(VirtualKeyCode::E) {
            translation += -up() * SPEED * delta_time;
        }
        self.render_camera.as_ref().unwrap().as_mut()
            .camera.translate(&translation);
    }

    fn stop(&mut self) {
        
    }
}