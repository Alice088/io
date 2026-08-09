//! Runs stayputnik async calls synchronously.
//!
//! HAL code uses `self.exec.block_on(any_stayputnik_call)` instead of
//! async/await. Runtime is created once in `ksp::world` and lives here.

use std::{future::Future, sync::Arc};

/// tokio-runtime wrapper. Clone to share between HAL devices.
#[derive(Clone)]
pub struct Executor {
    rt: Arc<tokio::runtime::Runtime>,
}

impl Executor {
    /// Only `hal::world` (worker thread) creates it.
    pub(crate) fn new(rt: tokio::runtime::Runtime) -> Self {
        Self { rt: Arc::new(rt) }
    }

    /// Blocks current thread until the future completes.
    pub fn block_on<F: Future>(&self, fut: F) -> F::Output {
        self.rt.block_on(fut)
    }
}

#[cfg(test)]
mod tests {
    use super::Executor;

    fn test_exec() -> Executor {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime build");
        Executor::new(rt)
    }

    #[test]
    fn runs_async_future() {
        let exec = test_exec();
        assert_eq!(exec.block_on(async { 40 + 2 }), 42);
    }

    #[test]
    fn reusable_across_many_block_ons() {
        let exec = test_exec();
        for i in 0..10 {
            assert_eq!(exec.block_on(async move { i * 2 }), i * 2);
        }
    }

    #[test]
    fn clonable() {
        let exec = test_exec();
        let exec2 = exec.clone();
        assert_eq!(exec2.block_on(async { "ok" }), "ok");
        assert_eq!(exec.block_on(async { 1 }), 1);
    }
}
