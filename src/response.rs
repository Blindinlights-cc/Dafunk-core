use crate::types::action::OnebotAction;

pub enum Response {
    Empty,
    Actions(OnebotAction<String>),
}
pub trait IntoResponse {
    fn into_response(&self) -> Response;
}
