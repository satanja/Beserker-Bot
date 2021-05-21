pub enum ResponseType {
    Error,
    Success,
    Warning,
}

pub struct Response {
    pub response_type: ResponseType,
    pub title: String,
    pub contents: String,
}

impl Response {
    pub fn new_error(title: String, contents: String) -> Response {
        Response {
            response_type: ResponseType::Error,
            title,
            contents
        }
    }

    pub fn new_success(title: String, contents: String) -> Response {
        Response {
            response_type: ResponseType::Success,
            title,
            contents
        }
    }

    pub fn new_warning(title: String, contents: String) -> Response {
        Response {
            response_type: ResponseType::Warning,
            title,
            contents
        }
    }
}
