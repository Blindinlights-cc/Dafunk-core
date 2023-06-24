use serde::Deserialize;


#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct OnebotActionResponse <R>
{
    pub status: String,
    pub retcode: i32,
    pub data: R,
    pub message: String,
    pub echo:Option<String>
}


