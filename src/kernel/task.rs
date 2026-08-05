use crate::kernel::clock::Ms;

#[derive(Clone, Copy, PartialEq)]
pub enum TaskState {
    Ready,
    Running,
    Disabled,
    Failed,
}

pub type TaskCallback = fn();

#[derive(Clone, Copy)]
pub struct Task {
    pub name: &'static str,
    pub period: Ms,
    pub next: Ms,
    pub state: TaskState,
    pub callback: TaskCallback,
}

impl Task {
    pub const fn new(
        name: &'static str,
        period: Ms,
        callback: TaskCallback,
    ) -> Self {
        Self {
            name,
            period,
            next: 0,
            state: TaskState::Ready,
            callback,
        }
    }
}
