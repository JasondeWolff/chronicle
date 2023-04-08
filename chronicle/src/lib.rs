pub use std::rc::Rc;
pub use std::cell::{RefCell, Ref, RefMut};

pub mod core_loop;
pub use core_loop::CoreLoop;

pub mod graphics;
pub mod resources;
pub mod common;
pub mod system;

use common::Timer;
use system::System;

thread_local! {
    static APP: RefCell<Option<App>> = RefCell::new(None);
}

pub fn init<G: Game + 'static>(title: &'static str, game: Box<G>, core_loop: &CoreLoop) {
    APP.with(|x| {
        *x.borrow_mut() = Some(App::new(game));
        x.borrow_mut().as_mut().unwrap().init_window(title, core_loop);
    });
}

pub fn app<R, F: FnOnce(&RefCell<Option<App>>,) -> R>(closure: F) -> R {
    APP.with(closure)
}

#[macro_export]
macro_rules! app_mut {
    ($x: ident) => {
        $x.borrow_mut().as_mut().unwrap()
    }
}

#[macro_export]
macro_rules! app_ref {
    ($x: ident) => {
        $x.borrow().as_ref().unwrap()
    }
}

pub trait Game {
    fn new() -> Box<Self> where Self: Sized;

    fn start(&mut self);
    fn update(&mut self, delta_time: f32);
    fn stop(&mut self);
}

pub struct App {
    window: Option<Box<graphics::Window>>,
    graphics: Option<Box<graphics::Renderer>>,
    resources: Option<Box<resources::Resources>>,
    game_timer: Timer,
    delta_timer: Timer,
    running: bool,
    game: Box<dyn Game>
}

impl App {
    pub fn new<G: Game + 'static>(game: Box<G>) -> Self {
        let mut app = App {
            window: None,
            graphics: None,
            resources: None,
            game_timer: Timer::new(),
            delta_timer: Timer::new(),
            running: false,
            game: game
        };

        app
    }

    fn init_window(&mut self, title: &'static str, core_loop: &CoreLoop) {
        app(|x| {
            let brk = 0;
        });
        self.window = Some(graphics::Window::new(core_loop, title, 1280, 720));
    }

    fn init_systems(&mut self) {
        app(|x| {
            let brk = 0;
        });

        self.resources = Some(resources::Resources::init());
        self.graphics = Some(graphics::Renderer::init(&self.window()));

        self.game.start();
    }

    pub(crate) fn update(&mut self) {
        if self.resources.is_none() {
            self.init_systems();
        }

        let delta_time = self.delta_timer.elapsed();
        self.delta_timer.reset();

        self.game.update(delta_time);

        self.graphics().update();
        self.resources().update();
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

    pub fn window(&mut self) -> &mut graphics::Window {
        self.window.as_mut().unwrap().as_mut()
    }

    pub fn graphics(&mut self) -> &mut graphics::Renderer {
        self.graphics.as_mut().unwrap().as_mut()
    }

    pub fn resources(&mut self) -> &mut resources::Resources {
        self.resources.as_mut().unwrap().as_mut()
    }
}