use async_trait::async_trait;

#[async_trait]
pub trait Component: Send {
    fn name(&self) -> &'static str;

    async fn update(&mut self);
}
