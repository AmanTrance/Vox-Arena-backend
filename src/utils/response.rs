use std::fmt::Debug;

use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
pub struct ErrorResponse<'a> {
    pub error: &'a str
}

#[derive(Debug, PartialEq, Serialize)]
pub struct ApiResponse<T: Serialize + Debug> {
    pub data: T
}
