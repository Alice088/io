use crate::kernel::clock::Ms;

pub type TaskCallback = fn();

#[derive(Clone, Copy)]
pub struct Task {
    pub name: &'static str,
    pub period: Ms,
    pub next: Ms,
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
            callback,
        }
    }
}
