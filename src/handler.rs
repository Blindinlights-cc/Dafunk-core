use futures::Future;

use crate::{extract::DependencyMap, response::IntoResponse};

pub trait Handler<T, E> {
    type Output: IntoResponse;
    async fn call(self, event: E, dep: &mut DependencyMap) -> Self::Output;
}

impl<E, Func, Ret, Fut> Handler<((),), E> for Func
where
    Func: FnOnce() -> Fut+ Send + 'static,
    Fut: Future<Output = Ret> + Send,
    Ret: IntoResponse,
{
    type Output = Ret;
    async fn call(self, _event: E, _dep: &mut DependencyMap) -> Self::Output {
        self().await
    }
}
