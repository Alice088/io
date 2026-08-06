use std::future::Future;
use std::pin::Pin;

use crate::kernel::clock::Ms;

pub type TaskFuture = Pin<Box<dyn Future<Output = ()>>>;

pub type TaskCallback = Box<dyn FnMut() -> TaskFuture>;

pub struct Task {
    pub name: &'static str,
    pub period: Ms,
    pub next: Ms,
    pub callback: TaskCallback,
}

impl Task {
    pub fn new(name: &'static str, period: Ms, callback: TaskCallback) -> Self {
        Self {
            name,
            period,
            next: 0,
            callback,
        }
    }
}
