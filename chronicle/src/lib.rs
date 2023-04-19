#![feature(thread_local)]

pub extern crate imgui;

pub use cgmath::*;

pub mod common;
pub use common::*;

pub use std::rc::Rc;
pub use std::cell::{RefCell, Ref, RefMut};

pub mod core_loop;
pub use core_loop::CoreLoop;

pub mod window;
pub use window::Window;
pub mod graphics;
pub mod resources;
pub mod input;

use common::Timer;

#[thread_local]
static mut APP: Option<Box<App>> = None;

pub fn init<G: Game + 'static>(title: &'static str, game: Box<G>, core_loop: &CoreLoop) {
    unsafe {
        APP = Some(Box::new(App::new(game)));
        APP.as_mut().unwrap().init_window(title, core_loop);
    }
}

pub fn app() -> &'static mut App {
    unsafe {
        APP.as_mut().unwrap()
    }
}

pub trait Game {
    fn new() -> Box<Self> where Self: Sized;

    fn start(&mut self);
    fn update(&mut self, delta_time: f32);
    fn gui(&mut self, delta_time: f32, gui: &mut graphics::ImGuiUI);
    fn stop(&mut self);
}

pub struct App {
    window: Option<Box<Window>>,
    graphics: Option<Box<graphics::Renderer>>,
    resources: Option<Box<resources::Resources>>,
    input: Option<Box<input::Input>>,
    game_timer: Timer,
    delta_timer: Timer,
    running: bool,
    game: Box<dyn Game>
}

impl App {
    pub fn new<G: Game + 'static>(game: Box<G>) -> Self {
        let app = App {
            window: None,
            graphics: None,
            resources: None,
            input: None,
            game_timer: Timer::new(),
            delta_timer: Timer::new(),
            running: false,
            game: game
        };

        app
    }

    fn init_window(&mut self, title: &'static str, core_loop: &CoreLoop) {
        self.window = Some(Window::new(core_loop, title, 1280, 720));
    }

    fn init_systems(&mut self) {
        self.resources = Some(resources::Resources::init());
        self.graphics = Some(graphics::Renderer::init(&self.window()));
        self.input = Some(input::Input::init());

        self.game.start();
    }

    pub(crate) fn is_init(&self) -> bool {
        self.input.is_some()
    }

    pub(crate) fn update(&mut self) {
        if self.resources.is_none() {
            self.init_systems();
        }

        let delta_time = self.delta_timer.elapsed();
        self.delta_timer.reset();

        self.game.update(delta_time);
        self.game.gui(delta_time, app().graphics().imgui_frame());

        self.graphics().update();
        self.resources().update();
        self.input().update();
    }

    pub fn quit(&mut self) {
        if self.running {
            self.running = false;
            self.game.stop();
        }
    }

    pub fn time(&self) -> f32 {
        self.game_timer.elapsed()
    }

    pub fn window(&mut self) -> &mut Window {
        self.window.as_mut().unwrap().as_mut()
    }

    pub fn graphics(&mut self) -> &mut graphics::Renderer {
        self.graphics.as_mut().unwrap().as_mut()
    }

    pub fn resources(&mut self) -> &mut resources::Resources {
        self.resources.as_mut().unwrap().as_mut()
    }

    pub fn input(&mut self) -> &mut input::Input {
        self.input.as_mut().unwrap().as_mut()
    }
}