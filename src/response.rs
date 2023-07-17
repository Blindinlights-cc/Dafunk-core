pub struct Response;
pub trait IntoResponse {
    fn into_response(self) -> Response;
}
