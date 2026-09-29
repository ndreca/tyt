use crate::{
    Conv, Dependencies, Error, InputMessage, OaiRequest, OaiResponse, Result, Role, UsrPrefs,
};
use std::{
    env, fs,
    io::{Error as IOError, ErrorKind},
    path::{Path, PathBuf},
};
use ty_preferences::{
    Dependencies as _, DependenciesImpl as PrefsDependenciesImpl, JsoncCodec, load_prefs_from_dir,
    load_sources_prefs, resolve_git_root_dir_from_cwd,
};
use tyt_injection::serde_json::{self, Map, Value, json};

/// The OpenAI Responses API endpoint.
const OPENAI_URL: &str = "https://api.openai.com/v1/responses";

/// The [`Dependencies`] backed by the git root's config files, the real
/// filesystem and stdout, and the OpenAI API.
#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    fn oai_api_key(&self) -> Result<Option<String>> {
        let Some(git_root) =
            resolve_git_root_dir_from_cwd(&self.current_dir()?).map_err(Error::IO)?
        else {
            return Ok(None);
        };

        let prefs: Option<UsrPrefs> = load_prefs_from_dir(
            &PrefsDependenciesImpl,
            &JsoncCodec,
            &git_root,
            ".tytusrconfig",
            "oai",
        )
        .map_err(Error::IO)?;

        Ok(prefs.and_then(|p| p.api_key))
    }

    fn current_dir(&self) -> Result<PathBuf> {
        Ok(env::current_dir()?)
    }

    fn is_dir(&self, path: &Path) -> Result<bool> {
        Ok(path.is_dir())
    }

    fn system_prompts_dir(&self) -> Result<Option<PathBuf>> {
        let cwd = self.current_dir()?;

        let Some(git_root) = resolve_git_root_dir_from_cwd(&cwd).map_err(Error::IO)? else {
            return Ok(None);
        };

        // `.tytconfig` is read first so a `systemPromptsDir` in `.tytusrconfig`
        // overrides it.
        let sources = [
            (git_root.as_path(), ".tytconfig"),
            (git_root.as_path(), ".tytusrconfig"),
        ];

        let mut dir = None;
        for layer in
            load_sources_prefs::<UsrPrefs>(&PrefsDependenciesImpl, &JsoncCodec, &sources, "oai")
                .map_err(Error::IO)?
        {
            if let Some(configured) = layer.prefs.img.and_then(|img| img.system_prompts_dir) {
                dir = Some(configured);
            }
        }

        Ok(dir.map(|dir| {
            let path = PathBuf::from(&dir);
            if path.is_absolute() {
                path
            } else {
                git_root.join(path)
            }
        }))
    }

    fn read_system_prompt(&self, path: &Path) -> Result<Option<String>> {
        let Some(bytes) = PrefsDependenciesImpl.read_file(path).map_err(Error::IO)? else {
            return Ok(None);
        };
        let text = String::from_utf8(bytes).map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;
        Ok(Some(text))
    }

    fn read_stdin(&self) -> Result<String> {
        Ok(tyt_injection::read_stdin()?)
    }

    fn read_conv(&self, path: &Path) -> Result<Option<Conv>> {
        let Some(bytes) = PrefsDependenciesImpl.read_file(path).map_err(Error::IO)? else {
            return Ok(None);
        };
        let conv =
            serde_json::from_slice(&bytes).map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;
        Ok(Some(conv))
    }

    fn write_conv(&self, path: &Path, conv: &Conv) -> Result<()> {
        let bytes =
            serde_json::to_vec_pretty(conv).map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;
        Ok(tyt_injection::write_file_atomic(path, &bytes)?)
    }

    fn write_image(&self, path: &Path, bytes: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_file_atomic(path, bytes)?)
    }

    fn generate_image(&self, api_key: &str, request: &OaiRequest) -> Result<OaiResponse> {
        send_image_request(api_key, request)
    }

    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_stdout(contents)?)
    }

    fn display_image_in_terminal(&self, path: &Path) -> Result<()> {
        Ok(tyt_injection::display_image_in_terminal(path)?)
    }
}

/// Sends an [`OaiRequest`] to the OpenAI Responses API and parses the result.
fn send_image_request(api_key: &str, request: &OaiRequest) -> Result<OaiResponse> {
    let body = build_request_body(request)?;
    let body_bytes = serde_json::to_vec(&body).map_err(|e| Error::Http(e.to_string()))?;

    let authorization = format!("Bearer {api_key}");
    let headers = [
        ("Authorization", authorization.as_str()),
        ("Content-Type", "application/json"),
    ];

    // The OpenAI error body (including an expired previous response) is surfaced
    // on non-2xx responses, so `http_post` returns the status for us to inspect.
    let (status, response_bytes) =
        tyt_injection::http_post(OPENAI_URL, &headers, body_bytes.as_slice())
            .map_err(|e| Error::Http(e.to_string()))?;

    parse_response(status, &response_bytes)
}

/// Builds the JSON request body for the Responses API.
fn build_request_body(request: &OaiRequest) -> Result<Value> {
    let mut body = Map::new();
    body.insert("model".to_owned(), Value::String(request.model.clone()));
    if let Some(previous_response_id) = &request.previous_response_id {
        body.insert(
            "previous_response_id".to_owned(),
            Value::String(previous_response_id.clone()),
        );
    }
    body.insert("input".to_owned(), build_input(&request.input)?);
    if request.generate_image {
        body.insert(
            "tools".to_owned(),
            json!([{ "type": "image_generation", "quality": request.quality.as_api_str() }]),
        );
    }
    Ok(Value::Object(body))
}

/// Builds the `input` field, using the plain-string form for a single text-only
/// user message and the array form otherwise.
fn build_input(messages: &[InputMessage]) -> Result<Value> {
    if let [message] = messages
        && message.role == Role::User
        && message.image.is_none()
        && let Some(text) = &message.text
    {
        return Ok(Value::String(text.clone()));
    }

    let mut items = Vec::with_capacity(messages.len());
    for message in messages {
        items.push(build_message(message)?);
    }
    Ok(Value::Array(items))
}

/// Builds a single `input` message as JSON.
fn build_message(message: &InputMessage) -> Result<Value> {
    let role = match message.role {
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::System => "system",
    };

    let content = match &message.image {
        None => Value::String(message.text.clone().unwrap_or_default()),
        Some(image) => {
            let image_part = json!({
                "type": "input_image",
                "image_url": image_data_uri(image)?,
            });
            let mut parts = Vec::with_capacity(2);
            match &message.text {
                Some(text) => {
                    let text_part = json!({ "type": "input_text", "text": text });
                    if message.image_first {
                        parts.push(image_part);
                        parts.push(text_part);
                    } else {
                        parts.push(text_part);
                        parts.push(image_part);
                    }
                }
                None => parts.push(image_part),
            }
            Value::Array(parts)
        }
    };

    Ok(json!({ "role": role, "content": content }))
}

/// Reads an image file and encodes it as a base64 `data:` URI.
fn image_data_uri(path: &Path) -> Result<String> {
    let bytes = fs::read(path).map_err(Error::IO)?;
    Ok(format!(
        "data:image/png;base64,{}",
        tyt_injection::encode_base64(&bytes)
    ))
}

/// Parses the Responses API result, mapping OpenAI errors to [`Error`].
fn parse_response(status: u16, bytes: &[u8]) -> Result<OaiResponse> {
    let value: Value = serde_json::from_slice(bytes).map_err(|e| {
        Error::InvalidResponse(format!("could not parse response (HTTP {status}): {e}"))
    })?;

    // A successful Responses object still carries an `"error": null` field, so
    // only a non-null error indicates an actual failure.
    if let Some(error) = value.get("error").filter(|error| !error.is_null()) {
        let code = error
            .get("code")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if code == "previous_response_not_found" {
            return Err(Error::PreviousResponseExpired);
        }
        let message = error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("unknown error");
        return Err(Error::Api(format!("{message} (HTTP {status})")));
    }

    if !(200..300).contains(&status) {
        return Err(Error::Api(format!("request failed with HTTP {status}")));
    }

    let response_id = value
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| Error::InvalidResponse("response is missing 'id'".to_owned()))?
        .to_owned();

    let output = value
        .get("output")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::InvalidResponse("response is missing 'output'".to_owned()))?;

    let mut text = String::new();
    let mut image_png = None;
    let mut revised_prompt = None;
    for item in output {
        match item.get("type").and_then(Value::as_str) {
            Some("message") => collect_message_text(item, &mut text),
            Some("image_generation_call") if image_png.is_none() => {
                if let Some(result) = item.get("result").and_then(Value::as_str) {
                    let decoded = tyt_injection::decode_base64(result).map_err(|e| {
                        Error::InvalidResponse(format!("could not decode image: {e}"))
                    })?;
                    image_png = Some(decoded);
                    // `revised_prompt` may be absent or null; `as_str` maps both
                    // to None.
                    revised_prompt = item
                        .get("revised_prompt")
                        .and_then(Value::as_str)
                        .map(str::to_owned);
                }
            }
            _ => {}
        }
    }

    Ok(OaiResponse {
        text,
        image_png,
        revised_prompt,
        response_id,
    })
}

/// Appends the `output_text` parts of an assistant `message` item to `text`.
fn collect_message_text(item: &Value, text: &mut String) {
    let Some(content) = item.get("content").and_then(Value::as_array) else {
        return;
    };
    for part in content {
        if part.get("type").and_then(Value::as_str) == Some("output_text")
            && let Some(chunk) = part.get("text").and_then(Value::as_str)
        {
            text.push_str(chunk);
        }
    }
}
