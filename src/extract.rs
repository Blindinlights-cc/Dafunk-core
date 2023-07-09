use std::ops::{Deref, DerefMut};

pub use http::Extensions;

use crate::error::Rejection;

pub trait FromEvent<Event>: Sized {
    type Rejection;
    fn from_event(event: &Event, container: &DependencyMap) -> Result<Self, Self::Rejection>;
}
pub struct DependencyMap {
    map: Extensions,
}
impl Deref for DependencyMap {
    type Target = Extensions;
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}
impl DerefMut for DependencyMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}
impl DependencyMap {
    pub fn new() -> Self {
        Self {
            map: Extensions::new(),
        }
    }
}
pub struct Dep<T>(T);

impl<Event, T> FromEvent<Event> for Dep<T>
where
    T: Send + Sync + 'static + Clone,
{
    type Rejection = Rejection;
    fn from_event(_event: &Event, container: &DependencyMap) -> Result<Self, Self::Rejection> {
        let dep = container.get::<T>().ok_or_else(|| Rejection::NoDependecy)?;
        Ok(Self(dep.clone()))
    }
}
