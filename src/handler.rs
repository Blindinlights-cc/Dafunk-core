use crate::extract::FromEvent;
use crate::response::IntoResponse;
use crate::{response::Response, types::Event};
use std::future::Future;
pub trait Handler<T, Dep, E = Event> {
    async fn call(self, event: E, dep: &mut Dep) -> Response;
}
macro_rules! impl_handler {
    (
        $($genric:ident),*
    ) => {
        #[allow(non_snake_case, unused_mut)]
        impl<F,Fut,Dep,E,Res,$($genric,)*> Handler<($($genric,)*),Dep,E> for F
        where
        F:FnOnce($($genric,)*) -> Fut+Clone+Send+'static,
        Fut:Future<Output=Res>+Send,
        E:Send+'static,
        Res:IntoResponse,
        $($genric:FromEvent<Dep,E>+Send,)*
        {
            async fn call(self, event: E, dep: &mut Dep) -> Response{
                $(
                    let $genric = match $genric::from_event(&event, dep).await{
                    Ok(v)=>v,
                    Err(e)=>return e.into_response(),
                };)*
                let res = self($($genric,)*).await;
                res.into_response()
            }
        }
    };
}
impl_handler!(T1);
impl_handler!(T1, T2);
impl_handler!(T1, T2, T3);
impl_handler!(T1, T2, T3, T4);
impl_handler!(T1, T2, T3, T4, T5);
impl_handler!(T1, T2, T3, T4, T5, T6);
impl_handler!(T1, T2, T3, T4, T5, T6, T7);
impl_handler!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_handler!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_handler!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_handler!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_handler!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_handler!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_handler!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_handler!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
impl_handler!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);
