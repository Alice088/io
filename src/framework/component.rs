pub trait Component {
    fn name(&self) -> &'static str;
    fn update(&mut self);
}
