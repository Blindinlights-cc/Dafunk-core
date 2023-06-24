
pub enum RequestError {}
pub struct ActionRequest<B,A> {
    pub bot: B,
    pub payload: A,
}
