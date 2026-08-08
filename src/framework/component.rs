pub trait Component: Send {
    fn name(&self) -> &'static str;
    fn update(&mut self);
}
