use std::marker::PhantomData;

use crate::types::Event;
pub struct Bot<E> {
    event:PhantomData<E>,
}
pub trait Handler<T, S, E = Event> {
    type Output;
    async fn call(self, event: Bot<E>, state: &mut S) -> Self::Output;
}
