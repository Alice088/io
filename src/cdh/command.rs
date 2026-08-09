#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    On(Target),
    Off(Target),
    Set(Target, Value),
    Do(Action),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Stabilization,
    Damping,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {}
