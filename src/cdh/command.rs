pub enum Command {
    On(Target),
    Off(Target),
    Set(Target, Value),
    Do(Action)
}

pub enum Target {
    Stabilization
}

pub enum Action {}