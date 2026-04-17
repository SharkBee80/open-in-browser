use tiny_http::{Header, Response, StatusCode};

pub fn json_response<T: serde::Serialize>(
    data: &T,
    status: StatusCode,
) -> Response<std::io::Cursor<Vec<u8>>> {
    let body = match serde_json::to_string(data) {
        Ok(s) => s,
        Err(_) => {
            return Response::from_data(
                serde_json::json!({ "error": "Failed to serialize JSON" })
                    .to_string()
                    .into_bytes(),
            )
            .with_status_code(500);
        }
    };

    let mut resp = Response::from_data(body.into_bytes()).with_status_code(status);

    // Set Content-Type: application/json
    if let Ok(header) = Header::from_bytes(b"Content-Type", b"application/json") {
        resp = resp.with_header(header);
    }

    resp
}
