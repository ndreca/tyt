use std::io::{Error, Result};

/// The response body cap in bytes. The cap leaves room for generated 3D assets
/// such as GLB and FBX models.
const RESPONSE_LIMIT: u64 = 512 * 1024 * 1024;

/// Sends an HTTP GET request with the given headers, returning the response
/// status code and body bytes.
///
/// Non-2xx responses are returned as `Ok` rather than an error so the caller
/// can inspect the body.
pub fn http_get(url: &str, headers: &[(&str, &str)]) -> Result<(u16, Vec<u8>)> {
    let mut request = ureq::get(url).config().http_status_as_error(false).build();
    for (name, value) in headers {
        request = request.header(*name, *value);
    }

    let mut response = request.call().map_err(Error::other)?;
    let status = response.status().as_u16();
    let bytes = response
        .body_mut()
        .with_config()
        .limit(RESPONSE_LIMIT)
        .read_to_vec()
        .map_err(Error::other)?;

    Ok((status, bytes))
}
