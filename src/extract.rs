use crate::{types::Event, response::IntoResponse};

pub trait FromEvent<Dep,E=Event>: Sized
{
    type Rejection:IntoResponse;
    async fn from_event(event: &E, container: &Dep) -> Result<Self, Self::Rejection>;
}
pub trait DepSupplier{
    fn get<T>(&self) -> Option<T>;
}