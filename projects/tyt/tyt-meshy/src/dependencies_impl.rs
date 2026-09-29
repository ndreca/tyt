use crate::{
    Dependencies, Error, MeshOutput, MeshProcessed, MeshRequest, MeshTask, MeshTaskFile, Result,
    TaskFileHead, TextureRequest, TextureTaskFile, UsrPrefs,
};
use std::{
    env, fs,
    io::{Error as IOError, ErrorKind},
    path::{Path, PathBuf},
    thread,
    time::Duration,
};
use ty_preferences::{
    DependenciesImpl as PrefsDependenciesImpl, JsoncCodec, load_prefs_from_dir,
    resolve_git_root_dir_from_cwd,
};
use tyt_injection::serde_json::{self, Error as JsonError, Map, Value, json};

/// The Meshy image-to-3D endpoint, used to create tasks (POST) and retrieve them
/// (GET `/{id}`).
const IMAGE_TO_3D_URL: &str = "https://api.meshy.ai/openapi/v1/image-to-3d";

/// The Meshy retexture endpoint, used to create tasks (POST) and retrieve them
/// (GET `/{id}`).
const RETEXTURE_URL: &str = "https://api.meshy.ai/openapi/v1/retexture";

/// The [`Dependencies`] backed by the Meshy HTTP API, `.tytusrconfig`, the
/// filesystem, and the terminal.
#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    fn meshy_api_key(&self) -> Result<Option<String>> {
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
            "meshy",
        )
        .map_err(Error::IO)?;

        Ok(prefs.and_then(|prefs| prefs.api_key))
    }

    fn current_dir(&self) -> Result<PathBuf> {
        Ok(env::current_dir()?)
    }

    fn read_text(&self, path: &Path) -> Result<String> {
        let bytes = fs::read(path).map_err(Error::IO)?;
        String::from_utf8(bytes).map_err(|e| Error::IO(IOError::new(ErrorKind::InvalidData, e)))
    }

    fn read_input_task_id(&self, path: &Path) -> Result<String> {
        let bytes = fs::read(path).map_err(Error::IO)?;
        let value: Value = serde_json::from_slice(&bytes)
            .map_err(|e| Error::InvalidTaskFile(format!("could not parse task file: {e}")))?;
        value
            .get("taskId")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| Error::InvalidTaskFile("task file is missing 'taskId'".to_owned()))
    }

    fn read_task_file(&self, path: &Path) -> Result<TaskFileHead> {
        let bytes = fs::read(path).map_err(Error::IO)?;
        let value: Value = serde_json::from_slice(&bytes)
            .map_err(|e| Error::InvalidTaskFile(format!("could not parse task file: {e}")))?;
        let task_id = field_str(&value, "taskId")?.to_owned();
        let task_kind = field_str(&value, "taskKind")?.to_owned();
        let input = value
            .get("payload")
            .and_then(|payload| payload.get("input"))
            .ok_or_else(|| {
                Error::InvalidTaskFile("task file is missing 'payload.input'".to_owned())
            })?;
        let input_json = serde_json::to_vec(input).map_err(invalid_data)?;
        Ok(TaskFileHead {
            task_id,
            task_kind,
            input_json,
        })
    }

    fn create_task(&self, api_key: &str, request: &MeshRequest) -> Result<String> {
        let body = build_create_body(request)?;
        create(IMAGE_TO_3D_URL, api_key, &body)
    }

    fn create_texture_task(&self, api_key: &str, request: &TextureRequest) -> Result<String> {
        let body = build_texture_body(request)?;
        create(RETEXTURE_URL, api_key, &body)
    }

    fn get_task(&self, api_key: &str, task_id: &str) -> Result<MeshTask> {
        get(IMAGE_TO_3D_URL, api_key, task_id)
    }

    fn get_texture_task(&self, api_key: &str, task_id: &str) -> Result<MeshTask> {
        get(RETEXTURE_URL, api_key, task_id)
    }

    fn download(&self, url: &str) -> Result<Vec<u8>> {
        let (status, bytes) =
            tyt_injection::http_get(url, &[]).map_err(|e| Error::Http(e.to_string()))?;
        if !(200..300).contains(&status) {
            return Err(Error::Api(format!("download failed with HTTP {status}")));
        }
        Ok(bytes)
    }

    fn display_image_in_terminal(&self, path: &Path) -> Result<()> {
        Ok(tyt_injection::display_image_in_terminal(path)?)
    }

    fn display_images_in_grid(
        &self,
        paths: &[&Path],
        columns: u32,
        side_percent: u32,
    ) -> Result<()> {
        Ok(tyt_injection::display_images_in_grid(
            paths,
            columns,
            side_percent,
        )?)
    }

    fn write_file(&self, path: &Path, bytes: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_file_atomic(path, bytes)?)
    }

    fn write_task_file(&self, path: &Path, file: &MeshTaskFile) -> Result<()> {
        let input = serde_json::to_value(&file.input).map_err(invalid_data)?;
        let bytes = build_task_file(&file.task_id, "image-to-3d", input, &file.output)?;
        Ok(tyt_injection::write_file_atomic(path, &bytes)?)
    }

    fn write_texture_task_file(&self, path: &Path, file: &TextureTaskFile) -> Result<()> {
        let input = serde_json::to_value(&file.input).map_err(invalid_data)?;
        let bytes = build_task_file(&file.task_id, "retexture", input, &file.output)?;
        Ok(tyt_injection::write_file_atomic(path, &bytes)?)
    }

    fn write_polled_task_file(
        &self,
        path: &Path,
        head: &TaskFileHead,
        output: &MeshOutput,
    ) -> Result<()> {
        let input = serde_json::from_slice(&head.input_json).map_err(invalid_data)?;
        let bytes = build_task_file(&head.task_id, &head.task_kind, input, output)?;
        Ok(tyt_injection::write_file_atomic(path, &bytes)?)
    }

    fn sleep(&self, seconds: u64) -> Result<()> {
        thread::sleep(Duration::from_secs(seconds));
        Ok(())
    }

    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_stdout(contents)?)
    }
}

/// Posts a create request body to an endpoint and returns the task id.
fn create(url: &str, api_key: &str, body: &Value) -> Result<String> {
    let body_bytes = serde_json::to_vec(body).map_err(|e| Error::Http(e.to_string()))?;

    let authorization = format!("Bearer {api_key}");
    let headers = [
        ("Authorization", authorization.as_str()),
        ("Content-Type", "application/json"),
    ];

    let (status, response_bytes) = tyt_injection::http_post(url, &headers, &body_bytes)
        .map_err(|e| Error::Http(e.to_string()))?;

    parse_create_response(status, &response_bytes)
}

/// Builds the create request body from the stored input, swapping the local
/// `image` (and `texture_image`) paths for the API's base64 `image_url`
/// (`texture_image_url`).
fn build_create_body(request: &MeshRequest) -> Result<Value> {
    let mut body = serde_json::to_value(&request.input).map_err(|e| Error::Http(e.to_string()))?;
    let object = body
        .as_object_mut()
        .ok_or_else(|| Error::Http("input did not serialize to an object".to_owned()))?;

    object.remove("image");
    object.insert(
        "image_url".to_owned(),
        Value::String(image_data_uri(&request.image_path)?),
    );

    if object.remove("texture_image").is_some() {
        let path = request
            .texture_image_path
            .as_ref()
            .ok_or_else(|| Error::Http("missing texture image path".to_owned()))?;
        object.insert(
            "texture_image_url".to_owned(),
            Value::String(image_data_uri(path)?),
        );
    }

    Ok(body)
}

/// Builds the retexture create request body, swapping the local
/// `image_style_url` path for the API's base64 data URI.
fn build_texture_body(request: &TextureRequest) -> Result<Value> {
    let mut body = serde_json::to_value(&request.input).map_err(|e| Error::Http(e.to_string()))?;
    let object = body
        .as_object_mut()
        .ok_or_else(|| Error::Http("input did not serialize to an object".to_owned()))?;

    if object.remove("image_style_url").is_some() {
        let path = request
            .image_style_path
            .as_ref()
            .ok_or_else(|| Error::Http("missing style image path".to_owned()))?;
        object.insert(
            "image_style_url".to_owned(),
            Value::String(image_data_uri(path)?),
        );
    }

    Ok(body)
}

/// Returns a required top-level string field, erroring if it is absent.
fn field_str<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| Error::InvalidTaskFile(format!("task file is missing '{key}'")))
}

/// Reads an image file and encodes it as a base64 `data:` URI, choosing the MIME
/// type from the file extension.
fn image_data_uri(path: &Path) -> Result<String> {
    let mime = mime_for(path)?;
    let bytes = fs::read(path).map_err(Error::IO)?;
    Ok(format!(
        "data:{mime};base64,{}",
        tyt_injection::encode_base64(&bytes)
    ))
}

/// Returns the MIME type for an image path, erroring on formats Meshy does not
/// accept.
fn mime_for(path: &Path) -> Result<&'static str> {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase);
    match extension.as_deref() {
        Some("png") => Ok("image/png"),

        Some("jpg" | "jpeg") => Ok("image/jpeg"),

        other => Err(Error::UnsupportedImageFormat(
            other.unwrap_or("").to_owned(),
        )),
    }
}

/// Parses the create response, returning the task id under the `result` key.
fn parse_create_response(status: u16, bytes: &[u8]) -> Result<String> {
    let value: Value = serde_json::from_slice(bytes).map_err(|e| {
        Error::InvalidResponse(format!("could not parse response (HTTP {status}): {e}"))
    })?;

    if !(200..300).contains(&status) {
        return Err(Error::Api(api_message(&value, status)));
    }

    let result = value
        .get("result")
        .and_then(Value::as_str)
        .ok_or_else(|| Error::InvalidResponse("response is missing 'result'".to_owned()))?;
    Ok(result.to_owned())
}

/// Retrieves a task by id from the given endpoint.
fn get(base_url: &str, api_key: &str, task_id: &str) -> Result<MeshTask> {
    let url = format!("{base_url}/{task_id}");
    let authorization = format!("Bearer {api_key}");
    let headers = [("Authorization", authorization.as_str())];

    let (status, bytes) =
        tyt_injection::http_get(&url, &headers).map_err(|e| Error::Http(e.to_string()))?;

    parse_task_response(status, bytes)
}

/// Parses the retrieve response into a [`MeshTask`], keeping the verbatim body.
fn parse_task_response(status: u16, bytes: Vec<u8>) -> Result<MeshTask> {
    let value: Value = serde_json::from_slice(&bytes).map_err(|e| {
        Error::InvalidResponse(format!("could not parse task (HTTP {status}): {e}"))
    })?;

    if !(200..300).contains(&status) {
        return Err(Error::Api(api_message(&value, status)));
    }

    let status = value
        .get("status")
        .and_then(Value::as_str)
        .ok_or_else(|| Error::InvalidResponse("task is missing 'status'".to_owned()))?
        .to_owned();
    let progress = value.get("progress").and_then(Value::as_u64).unwrap_or(0) as u8;
    let model_urls = string_pairs(value.get("model_urls"));
    let texture_urls = value
        .get("texture_urls")
        .and_then(Value::as_array)
        .and_then(|sets| sets.first())
        .map(texture_pairs)
        .unwrap_or_default();
    let thumbnail_urls = thumbnail_pairs(&value);
    let error_message = value
        .get("task_error")
        .and_then(|error| error.get("message"))
        .and_then(Value::as_str)
        .filter(|message| !message.is_empty())
        .map(str::to_owned);

    Ok(MeshTask {
        status,
        progress,
        model_urls,
        texture_urls,
        thumbnail_urls,
        error_message,
        raw_json: bytes,
    })
}

/// Collects the string-valued entries of a JSON object as `(key, value)` pairs,
/// preserving their order. Meshy reports assets it did not generate as empty
/// strings, so empty values are treated as absent.
fn string_pairs(value: Option<&Value>) -> Vec<(String, String)> {
    value
        .and_then(Value::as_object)
        .map(|object| {
            object
                .iter()
                .filter_map(|(key, value)| {
                    value
                        .as_str()
                        .filter(|url| !url.is_empty())
                        .map(|url| (key.clone(), url.to_owned()))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Collects a task's thumbnails as ordered `(name, url)` pairs. A task created
/// with `multi_view_thumbnails` returns a `thumbnail_urls` object holding the
/// `front`/`right`/`back`/`left` cardinal views; otherwise Meshy returns a
/// single `thumbnail_url`, recorded as `default`. Empty URLs are treated as
/// absent.
fn thumbnail_pairs(value: &Value) -> Vec<(String, String)> {
    const VIEWS: [&str; 4] = ["front", "right", "back", "left"];
    if let Some(object) = value.get("thumbnail_urls").and_then(Value::as_object) {
        let views: Vec<(String, String)> = VIEWS
            .iter()
            .filter_map(|view| {
                object
                    .get(*view)
                    .and_then(Value::as_str)
                    .filter(|url| !url.is_empty())
                    .map(|url| ((*view).to_owned(), url.to_owned()))
            })
            .collect();
        if !views.is_empty() {
            return views;
        }
    }
    value
        .get("thumbnail_url")
        .and_then(Value::as_str)
        .filter(|url| !url.is_empty())
        .map(|url| vec![("default".to_owned(), url.to_owned())])
        .unwrap_or_default()
}

/// Collects a texture set's map URLs as `(map, url)` pairs in a stable order.
/// Meshy reports maps it did not generate as empty strings, so empty values are
/// treated as absent.
fn texture_pairs(set: &Value) -> Vec<(String, String)> {
    const MAPS: [&str; 5] = ["base_color", "metallic", "normal", "roughness", "emission"];
    let Some(object) = set.as_object() else {
        return Vec::new();
    };
    MAPS.iter()
        .filter_map(|map| {
            object
                .get(*map)
                .and_then(Value::as_str)
                .filter(|url| !url.is_empty())
                .map(|url| ((*map).to_owned(), url.to_owned()))
        })
        .collect()
}

/// Builds a task file's pretty JSON bytes from its id, kind, input, and output.
fn build_task_file(
    task_id: &str,
    task_kind: &str,
    input: Value,
    output: &MeshOutput,
) -> Result<Vec<u8>> {
    let output = match output {
        MeshOutput::Pending => Value::String("pending".to_owned()),

        MeshOutput::Done(done) => {
            let raw: Value = serde_json::from_slice(&done.raw_json).map_err(invalid_data)?;
            json!({ "raw": raw, "processed": processed_value(&done.processed) })
        }
    };

    let root = json!({
        "taskId": task_id,
        "taskKind": task_kind,
        "payload": { "input": input, "output": output },
    });
    serde_json::to_vec_pretty(&root).map_err(invalid_data)
}

/// Builds the `processed` object, omitting empty categories.
fn processed_value(processed: &MeshProcessed) -> Value {
    let mut object = Map::new();
    if !processed.model_files.is_empty() {
        object.insert("modelFiles".to_owned(), pairs_value(&processed.model_files));
    }
    if !processed.texture_files.is_empty() {
        object.insert(
            "textureFiles".to_owned(),
            pairs_value(&processed.texture_files),
        );
    }
    if !processed.thumbnail_files.is_empty() {
        object.insert(
            "thumbnailFiles".to_owned(),
            pairs_value(&processed.thumbnail_files),
        );
    }
    Value::Object(object)
}

/// Builds a JSON object from ordered `(key, value)` string pairs.
fn pairs_value(pairs: &[(String, String)]) -> Value {
    let mut object = Map::new();
    for (key, value) in pairs {
        object.insert(key.clone(), Value::String(value.clone()));
    }
    Value::Object(object)
}

/// Extracts a Meshy error body's `message`, falling back to the HTTP status.
fn api_message(value: &Value, status: u16) -> String {
    let message = value
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("unknown error");
    format!("{message} (HTTP {status})")
}

/// Maps a serialization error to an [`Error::IO`] with [`ErrorKind::InvalidData`].
fn invalid_data(e: JsonError) -> Error {
    Error::IO(IOError::new(ErrorKind::InvalidData, e))
}
