pub trait System {
    fn init() -> Box<Self>;
    fn update(&mut self);
}