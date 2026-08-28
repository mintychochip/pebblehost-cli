use clap::{Args, Parser, Subcommand};
use reqwest::{Client, Method, StatusCode};
use serde_json::{json, Value};
use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;

const DEFAULT_BASE_URL: &str = "https://panel.pebblehost.com";
const API_KEY_PAGE: &str = "https://panel.pebblehost.com/account/api";

#[derive(Debug, Error)]
enum CliError {
    #[error("missing API key: set PEBBLEHOST_API_KEY or run pb login")]
    MissingToken,
    #[error("credential error: {0}")]
    Credential(String),
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("PebbleHost API error ({status}): {message}")]
    Api { status: StatusCode, message: String },
    #[error("invalid input: {0}")]
    Input(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("update failed")]
    Update,
}

#[derive(Parser, Debug)]
#[command(
    name = "pb",
    version,
    about = "Manage PebbleHost servers from the command line"
)]
struct Cli {
    #[arg(long, env = "PEBBLEHOST_BASE_URL", default_value = DEFAULT_BASE_URL)]
    base_url: String,
    #[arg(
        long,
        global = true,
        visible_alias = "verbose",
        help = "Print compact JSON response payload"
    )]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
#[allow(clippy::enum_variant_names)]
enum Command {
    Login,
    Account,
    Servers,
    Server(ServerId),
    Power(PowerArgs),
    Command(CommandArgs),
    Resources(ServerId),
    Activity(ServerId),
    Backups(ServerId),
    Databases(ServerId),
    Allocations(ServerId),
    Schedules(ServerId),
    Players(ServerId),
    Plugins(PluginArgs),
    Modpacks(ModpackArgs),
    Files(FilesArgs),
    ApiCall(ApiCallArgs),
    Operations,
    FileSearch(FileSearchArgs),
    File(FileArgs),
    Update,
}

#[derive(Args, Debug)]
struct ServerId {
    server_id: String,
}
#[derive(Args, Debug)]
struct PowerArgs {
    server_id: String,
    #[arg(long, value_parser = ["start", "stop", "restart", "kill"])]
    action: String,
}
#[derive(Args, Debug)]
struct CommandArgs {
    server_id: String,
    #[arg(long)]
    command: String,
}
#[derive(Args, Debug)]
struct PluginArgs {
    server_id: String,
    #[arg(long)]
    provider: String,
    #[arg(long, default_value_t = 1)]
    page: u32,
    #[arg(long, default_value_t = 20)]
    page_size: u32,
    #[arg(long)]
    search_query: Option<String>,
    #[arg(long)]
    minecraft_version: Option<String>,
}
#[derive(Args, Debug)]
struct ModpackArgs {
    server_id: String,
    #[arg(long)]
    provider: String,
    #[arg(long, default_value_t = 1)]
    page: u32,
    #[arg(long, default_value_t = 20)]
    page_size: u32,
    #[arg(long)]
    search_query: Option<String>,
}
#[derive(Args, Debug)]
struct FilesArgs {
    server_id: String,
    #[arg(long, default_value = "/")]
    directory: String,
}
#[derive(Args, Debug)]
struct FileSearchArgs {
    server_id: String,
    query: String,
    #[arg(long, default_value = "/")]
    root: String,
}
#[derive(Args, Debug)]
struct FileArgs {
    server_id: String,
    path: String,
}
#[derive(Args, Debug)]
struct ApiCallArgs {
    /// HTTP method: GET, POST, PUT, PATCH, or DELETE.
    method: String,
    /// API path, with or without a leading slash.
    path: String,
    /// Query parameter in KEY=VALUE form; repeat for multiple parameters.
    #[arg(long, value_name = "KEY=VALUE", action = clap::ArgAction::Append)]
    query: Vec<String>,
    /// Raw JSON request body.
    #[arg(long, value_name = "JSON")]
    body: Option<String>,
}
struct Api {
    client: Client,
    base_url: String,
    token: String,
}

impl Api {
    fn new(base_url: String, token: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("reqwest client should build"),
            base_url: base_url.trim_end_matches('/').to_owned(),
            token,
        }
    }

    async fn request(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, String)],
        body: Option<Value>,
    ) -> Result<Response, CliError> {
        let mut req = self
            .client
            .request(method, format!("{}{}", self.base_url, path))
            .bearer_auth(&self.token)
            .header("Accept", "application/json")
            .query(query);
        if let Some(body) = body {
            req = req.json(&body);
        }
        let response = req.send().await?;
        let status = response.status();
        let text = response.text().await?;
        if !status.is_success() {
            return Err(CliError::Api {
                status,
                message: if text.is_empty() {
                    status.to_string()
                } else {
                    text
                },
            });
        }
        if text.is_empty() {
            return Ok(Response::Json(Value::Null));
        }
        Ok(serde_json::from_str(&text)
            .map(Response::Json)
            .unwrap_or_else(|_| Response::Text(text)))
    }
}

#[derive(Debug, PartialEq)]
enum Response {
    Json(Value),
    Text(String),
}

fn path_server(server: &str, suffix: &str) -> String {
    format!("/api/client/servers/{server}{suffix}")
}
async fn api_call(api: &Api, args: ApiCallArgs) -> Result<Response, CliError> {
    let method = match args.method.to_ascii_uppercase().as_str() {
        "GET" => Method::GET,
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "PATCH" => Method::PATCH,
        "DELETE" => Method::DELETE,
        other => return Err(CliError::Input(format!("unsupported HTTP method: {other}"))),
    };

    let path = if args.path.starts_with('/') {
        args.path
    } else {
        format!("/{}", args.path)
    };

    let query_pairs: Vec<(String, String)> = args
        .query
        .into_iter()
        .map(|pair| {
            pair.split_once('=')
                .map(|(key, value)| (key.to_owned(), value.to_owned()))
                .ok_or_else(|| CliError::Input(format!("query must be KEY=VALUE: {pair}")))
        })
        .collect::<Result<_, _>>()?;
    let query: Vec<(&str, String)> = query_pairs
        .iter()
        .map(|(key, value)| (key.as_str(), value.clone()))
        .collect();
    let body = args
        .body
        .map(|body| serde_json::from_str::<Value>(&body))
        .transpose()
        .map_err(|error| CliError::Input(format!("invalid JSON body: {error}")))?;

    api.request(method, &path, &query, body).await
}

async fn operations() -> Result<Response, CliError> {
    let value: Value = serde_json::from_str(include_str!("operations.json"))
        .map_err(|error| CliError::Input(format!("invalid bundled operations: {error}")))?;
    Ok(Response::Json(value))
}

async fn execute(api: &Api, command: Command) -> Result<Response, CliError> {
    match command {
        Command::Login => unreachable!("login is handled before API-key resolution"),
        Command::Account => {
            api.request(Method::GET, "/api/client/account", &[], None)
                .await
        }
        Command::Servers => api.request(Method::GET, "/api/client", &[], None).await,
        Command::Server(a) => {
            api.request(Method::GET, &path_server(&a.server_id, ""), &[], None)
                .await
        }
        Command::Power(a) => {
            api.request(
                Method::POST,
                &path_server(&a.server_id, "/power"),
                &[],
                Some(json!({"signal": a.action})),
            )
            .await
        }
        Command::Command(a) => {
            api.request(
                Method::POST,
                &path_server(&a.server_id, "/command"),
                &[],
                Some(json!({"command": a.command})),
            )
            .await
        }
        Command::Resources(a) => {
            api.request(
                Method::GET,
                &path_server(&a.server_id, "/resources"),
                &[],
                None,
            )
            .await
        }
        Command::Activity(a) => {
            api.request(
                Method::GET,
                &path_server(&a.server_id, "/activity"),
                &[],
                None,
            )
            .await
        }
        Command::Backups(a) => {
            api.request(
                Method::GET,
                &path_server(&a.server_id, "/backups"),
                &[],
                None,
            )
            .await
        }
        Command::Databases(a) => {
            api.request(
                Method::GET,
                &path_server(&a.server_id, "/databases"),
                &[],
                None,
            )
            .await
        }
        Command::Allocations(a) => {
            api.request(
                Method::GET,
                &path_server(&a.server_id, "/network/allocations"),
                &[],
                None,
            )
            .await
        }
        Command::Schedules(a) => {
            api.request(
                Method::GET,
                &path_server(&a.server_id, "/schedules"),
                &[],
                None,
            )
            .await
        }
        Command::Players(a) => {
            api.request(
                Method::GET,
                &path_server(&a.server_id, "/minecraft/players"),
                &[],
                None,
            )
            .await
        }
        Command::Plugins(a) => {
            search(
                api,
                &a.server_id,
                &a.provider,
                a.page,
                a.page_size,
                a.search_query.as_deref(),
                a.minecraft_version.as_deref(),
                "plugins",
            )
            .await
        }
        Command::Modpacks(a) => {
            search(
                api,
                &a.server_id,
                &a.provider,
                a.page,
                a.page_size,
                a.search_query.as_deref(),
                None,
                "modpacks",
            )
            .await
        }
        Command::Files(a) => {
            api.request(
                Method::GET,
                &path_server(&a.server_id, "/files/list"),
                &[("directory", a.directory)],
                None,
            )
            .await
        }
        Command::FileSearch(a) => {
            api.request(
                Method::GET,
                &path_server(&a.server_id, "/files/search"),
                &[("root", a.root), ("query", a.query)],
                None,
            )
            .await
        }
        Command::File(a) => {
            api.request(
                Method::GET,
                &path_server(&a.server_id, "/files/contents"),
                &[("file", a.path)],
                None,
            )
            .await
        }
        Command::ApiCall(args) => api_call(api, args).await,
        Command::Operations => operations().await,
        Command::Update => update().await,
    }
}

#[allow(clippy::too_many_arguments)]
async fn search(
    api: &Api,
    server_id: &str,
    provider: &str,
    page: u32,
    page_size: u32,
    search_query: Option<&str>,
    minecraft_version: Option<&str>,
    kind: &str,
) -> Result<Response, CliError> {
    let mut q = vec![
        ("provider", provider.to_owned()),
        ("page", page.to_string()),
        ("page_size", page_size.to_string()),
    ];
    if let Some(v) = search_query {
        q.push(("search_query", v.to_owned()));
    }
    if let Some(v) = minecraft_version {
        q.push(("minecraft_version", v.to_owned()));
    }
    api.request(
        Method::GET,
        &path_server(server_id, &format!("/minecraft/{kind}")),
        &q,
        None,
    )
    .await
}

fn config_root() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        if let Some(path) = std::env::var_os("XDG_CONFIG_HOME").filter(|value| !value.is_empty()) {
            return Some(PathBuf::from(path));
        }
        std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config"))
    }

    #[cfg(target_os = "macos")]
    {
        return std::env::var_os("HOME").map(|home| {
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
        });
    }

    #[cfg(target_os = "windows")]
    {
        return std::env::var_os("APPDATA").map(PathBuf::from);
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config"))
    }
}

fn credential_path() -> Option<PathBuf> {
    config_root().map(|root| root.join("pebblehost-cli").join("api-key"))
}

#[cfg(unix)]
fn validate_credential_metadata(path: &Path, metadata: &std::fs::Metadata) -> Result<(), CliError> {
    use std::os::unix::fs::PermissionsExt;

    if metadata.permissions().mode() & 0o077 != 0 {
        return Err(CliError::Credential(format!(
            "credential file has unsafe permissions: {}",
            path.display()
        )));
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_credential_metadata(
    _path: &Path,
    _metadata: &std::fs::Metadata,
) -> Result<(), CliError> {
    Ok(())
}

fn existing_credential(path: &Path) -> Result<bool, CliError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => {
            return Err(CliError::Credential(format!(
                "cannot inspect credential file: {error}"
            )))
        }
    };

    if metadata.file_type().is_symlink() {
        return Err(CliError::Credential(format!(
            "credential path is a symlink: {}",
            path.display()
        )));
    }
    if !metadata.file_type().is_file() {
        return Err(CliError::Credential(format!(
            "credential path is not a regular file: {}",
            path.display()
        )));
    }
    validate_credential_metadata(path, &metadata)?;
    Ok(true)
}

fn open_credential(path: &Path) -> Result<Option<std::fs::File>, CliError> {
    let mut options = OpenOptions::new();
    options.read(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }

    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT;
        options.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
    }

    match options.open(path) {
        Ok(file) => Ok(Some(file)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(CliError::Credential(format!(
            "cannot open credential file: {error}"
        ))),
    }
}

fn load_stored_token(path: &Path) -> Result<Option<String>, CliError> {
    let Some(mut file) = open_credential(path)? else {
        return Ok(None);
    };

    let metadata = file.metadata().map_err(|error| {
        CliError::Credential(format!("cannot inspect credential file: {error}"))
    })?;
    if metadata.file_type().is_symlink() {
        return Err(CliError::Credential(
            "credential path is a symlink".into(),
        ));
    }
    if !metadata.file_type().is_file() {
        return Err(CliError::Credential(
            "credential path is not a regular file".into(),
        ));
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(CliError::Credential(
                "credential path is a reparse point".into(),
            ));
        }
    }
    validate_credential_metadata(path, &metadata)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents).map_err(|error| {
        CliError::Credential(format!("cannot read credential file: {error}"))
    })?;
    let token = contents.trim();
    if token.is_empty() {
        return Err(CliError::Credential(
            "credential file contains an empty token".into(),
        ));
    }
    Ok(Some(token.to_owned()))
}

#[cfg(windows)]
fn replace_credential_file(temporary_path: &Path, path: &Path) -> Result<(), CliError> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{
        GetLastError, ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND,
    };
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, ReplaceFileW, MOVEFILE_WRITE_THROUGH, REPLACEFILE_WRITE_THROUGH,
    };

    let path_as_wide = |path: &Path| {
        path.as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>()
    };
    let destination = path_as_wide(path);
    let replacement = path_as_wide(temporary_path);

    let replaced = unsafe {
        ReplaceFileW(
            destination.as_ptr(),
            replacement.as_ptr(),
            std::ptr::null(),
            REPLACEFILE_WRITE_THROUGH,
            std::ptr::null(),
            std::ptr::null(),
        )
    };
    if replaced != 0 {
        return Ok(());
    }

    let replace_error = unsafe { GetLastError() };
    if replace_error != ERROR_FILE_NOT_FOUND && replace_error != ERROR_PATH_NOT_FOUND {
        return Err(CliError::Credential(format!(
            "cannot replace credential file: Windows error {replace_error}"
        )));
    }

    // ReplaceFileW requires an existing destination. MoveFileExW without
    // MOVEFILE_REPLACE_EXISTING fills the absent-destination case without
    // overwriting a destination that appears concurrently.
    let moved = unsafe {
        MoveFileExW(
            replacement.as_ptr(),
            destination.as_ptr(),
            MOVEFILE_WRITE_THROUGH,
        )
    };
    if moved != 0 {
        return Ok(());
    }

    let move_error = unsafe { GetLastError() };
    Err(CliError::Credential(format!(
        "cannot replace credential file: Windows error {move_error}"
    )))
}

fn save_stored_token(path: &Path, token: &str) -> Result<(), CliError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let parent = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };
    fs::create_dir_all(parent).map_err(|error| {
        CliError::Credential(format!("cannot create credential directory: {error}"))
    })?;
    #[cfg(unix)]
    fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700)).map_err(|error| {
        CliError::Credential(format!("cannot secure credential directory: {error}"))
    })?;

    existing_credential(path)?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let pid = std::process::id();
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("api-key");
    let mut temporary = None;
    for attempt in 0..100u32 {
        let temporary_path = parent.join(format!(".{file_name}.tmp-{pid}-{timestamp}-{attempt}"));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&temporary_path) {
            Ok(file) => {
                temporary = Some((temporary_path, file));
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(CliError::Credential(format!(
                    "cannot create temporary credential file: {error}"
                )))
            }
        }
    }
    let (temporary_path, mut file) = temporary.ok_or_else(|| {
        CliError::Credential("cannot create a unique temporary credential file".into())
    })?;

    #[cfg(unix)]
    if let Err(error) = fs::set_permissions(&temporary_path, std::fs::Permissions::from_mode(0o600))
    {
        drop(file);
        let _ = fs::remove_file(&temporary_path);
        return Err(CliError::Credential(format!(
            "cannot secure temporary credential file: {error}"
        )));
    }

    let result = (|| -> Result<(), CliError> {
        file.write_all(token.as_bytes()).map_err(|error| {
            CliError::Credential(format!("cannot write temporary credential file: {error}"))
        })?;
        file.write_all(b"\n").map_err(|error| {
            CliError::Credential(format!("cannot write temporary credential file: {error}"))
        })?;
        file.flush().map_err(|error| {
            CliError::Credential(format!("cannot flush temporary credential file: {error}"))
        })?;
        file.sync_all().map_err(|error| {
            CliError::Credential(format!("cannot sync temporary credential file: {error}"))
        })?;
        drop(file);

        #[cfg(unix)]
        fs::rename(&temporary_path, path).map_err(|error| {
            CliError::Credential(format!("cannot replace credential file: {error}"))
        })?;

        #[cfg(windows)]
        replace_credential_file(&temporary_path, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary_path);
    }
    result
}

fn resolve_token_from(env_token: Option<&OsStr>, path: &Path) -> Result<String, CliError> {
    if let Some(env_token) = env_token {
        let token = env_token
            .to_str()
            .ok_or_else(|| CliError::Credential("environment API key is not valid UTF-8".into()))?;
        let token = token.trim();
        if token.is_empty() {
            return Err(CliError::MissingToken);
        }
        return Ok(token.to_owned());
    }

    load_stored_token(path)?.ok_or(CliError::MissingToken)
}

fn resolve_token() -> Result<String, CliError> {
    if let Some(env_token) = std::env::var_os("PEBBLEHOST_API_KEY") {
        return resolve_token_from(Some(&env_token), Path::new(""));
    }

    let path = credential_path().ok_or(CliError::MissingToken)?;
    resolve_token_from(None, &path)
}

fn open_api_key_page() -> io::Result<()> {
    #[cfg(target_os = "linux")]
    let status = ProcessCommand::new("xdg-open").arg(API_KEY_PAGE).status()?;
    #[cfg(target_os = "macos")]
    let status = ProcessCommand::new("open").arg(API_KEY_PAGE).status()?;
    #[cfg(target_os = "windows")]
    let status = ProcessCommand::new("cmd")
        .args(["/C", "start", "", API_KEY_PAGE])
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(
            "browser launcher returned a non-success status",
        ))
    }
}

fn prompt_api_key() -> io::Result<String> {
    rpassword::prompt_password("API key: ")
}

async fn login_with<O, P>(
    base_url: &str,
    path: &Path,
    open: O,
    prompt: P,
) -> Result<Response, CliError>
where
    O: FnOnce() -> io::Result<()>,
    P: FnOnce() -> io::Result<String>,
{
    eprintln!("Open {API_KEY_PAGE} to generate an API key.");
    if let Err(error) = open() {
        eprintln!("Could not open the API-key page automatically: {error}");
    }
    let token = prompt()?.trim().to_owned();
    if token.is_empty() {
        return Err(CliError::Input("API key cannot be empty".into()));
    }

    let api = Api::new(base_url.to_owned(), token.clone());
    api.request(Method::GET, "/api/client/account", &[], None)
        .await
        .map_err(|error| match error {
            CliError::Api { status, .. } => CliError::Api {
                status,
                message: "API key validation failed".into(),
            },
            error => error,
        })?;
    save_stored_token(path, &token)?;

    Ok(Response::Text("API key saved successfully.".into()))
}

async fn login(base_url: &str) -> Result<Response, CliError> {
    let path = credential_path()
        .ok_or_else(|| CliError::Credential("cannot determine credential path".into()))?;
    login_with(base_url, &path, open_api_key_page, prompt_api_key).await
}

async fn run(cli: Cli) -> Result<Response, CliError> {
    if matches!(cli.command, Command::Login) {
        return login(&cli.base_url).await;
    }
    if matches!(cli.command, Command::Operations) {
        return operations().await;
    }
    if matches!(cli.command, Command::Update) {
        return update().await;
    }
    let token = resolve_token()?;
    run_with_token(cli, token).await
}

async fn run_with_token(cli: Cli, token: String) -> Result<Response, CliError> {
    execute(&Api::new(cli.base_url.clone(), token), cli.command).await
}

fn sort_value(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut entries: Vec<_> = map.into_iter().collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            Value::Object(
                entries
                    .into_iter()
                    .map(|(k, v)| (k, sort_value(v)))
                    .collect(),
            )
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(sort_value).collect()),
        other => other,
    }
}

fn format_response(response: Response, json: bool) -> String {
    match response {
        Response::Json(value) => {
            let sorted = sort_value(value);
            if json {
                sorted.to_string()
            } else {
                serde_json::to_string_pretty(&sorted).unwrap()
            }
        }
        Response::Text(text) => text,
    }
}

fn print_response(response: Response, json: bool) {
    println!("{}", format_response(response, json));
}

const VERSION_REMINDER_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
const VERSION_REMINDER_CONFIG_DIR: &str = ".config/pebblehost-cli";
const VERSION_REMINDER_FILE: &str = "version-reminder";

fn version_reminder_path() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME").map(|home| {
        let mut path = std::path::PathBuf::from(home);
        path.push(VERSION_REMINDER_CONFIG_DIR);
        path.push(VERSION_REMINDER_FILE);
        path
    })
}

fn maybe_show_version_reminder(json: bool, command: &Command) {
    if json {
        return;
    }
    if matches!(command, Command::Login | Command::Update) {
        return;
    }
    let Some(path) = version_reminder_path() else {
        return;
    };

    let current_version = env!("CARGO_PKG_VERSION");
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let mut should_print = true;

    if let Ok(content) = std::fs::read_to_string(&path) {
        let mut lines = content.lines();
        if let (Some(stored_version), Some(stored_ts)) = (lines.next(), lines.next()) {
            if stored_version == current_version {
                if let Ok(ts) = stored_ts.parse::<u64>() {
                    if now.saturating_sub(ts) < VERSION_REMINDER_INTERVAL.as_secs() {
                        should_print = false;
                    }
                }
            }
        }
    }

    if should_print {
        eprintln!(
            "pb {} is installed. Run `pb update` to check for the latest version.",
            current_version
        );
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, format!("{}\n{}\n", current_version, now));
    }
}

async fn update() -> Result<Response, CliError> {
    eprintln!("updating pb...");
    let script = "set -e; tmp=$(mktemp); curl -fsSL https://raw.githubusercontent.com/mintychochip/pebblehost-cli/master/scripts/update.sh -o \"$tmp\"; sh \"$tmp\"";
    let status = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(script)
        .status()
        .await?;

    if status.success() {
        Ok(Response::Text(
            "pb updated successfully. Run `pb --version` to verify.".to_string(),
        ))
    } else {
        Err(CliError::Update)
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let as_json = cli.json;
    maybe_show_version_reminder(as_json, &cli.command);
    match run(cli).await {
        Ok(response) => print_response(response, as_json),
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        matchers::{body_json, header, method, path, query_param},
        Mock, MockServer, ResponseTemplate,
    };

    static ENVIRONMENT_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    struct EnvironmentRestore {
        values: Vec<(&'static str, Option<std::ffi::OsString>)>,
    }

    impl EnvironmentRestore {
        fn capture(names: &[&'static str]) -> Self {
            Self {
                values: names
                    .iter()
                    .map(|&name| (name, std::env::var_os(name)))
                    .collect(),
            }
        }
    }

    impl Drop for EnvironmentRestore {
        fn drop(&mut self) {
            for (name, value) in self.values.drain(..) {
                if let Some(value) = value {
                    std::env::set_var(name, value);
                } else {
                    std::env::remove_var(name);
                }
            }
        }
    }

    fn test_api(server: &MockServer, token: &str) -> Api {
        Api::new(server.uri(), token.to_owned())
    }

    #[test]
    fn stored_token_round_trips_and_trims_newline() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pebblehost-cli").join("api-key");

        save_stored_token(&path, "secret").unwrap();

        assert_eq!(load_stored_token(&path).unwrap(), Some("secret".into()));
    }

    #[test]
    fn nonempty_environment_token_wins_over_stored_token() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("api-key");
        save_stored_token(&path, "stored").unwrap();

        assert_eq!(
            resolve_token_from(Some(std::ffi::OsStr::new("environment")), &path).unwrap(),
            "environment"
        );
    }
    #[test]
    fn nonempty_environment_token_resolves_without_config_root() {
        let _environment_lock = ENVIRONMENT_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let _environment = EnvironmentRestore::capture(&[
            "HOME",
            "XDG_CONFIG_HOME",
            "APPDATA",
            "PEBBLEHOST_API_KEY",
        ]);

        std::env::remove_var("HOME");
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::remove_var("APPDATA");
        std::env::set_var("PEBBLEHOST_API_KEY", "environment");

        assert_eq!(resolve_token().unwrap(), "environment");
    }

    #[test]
    fn explicit_empty_environment_token_does_not_fallback() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("api-key");
        save_stored_token(&path, "stored").unwrap();

        assert!(matches!(
            resolve_token_from(Some(std::ffi::OsStr::new("   ")), &path),
            Err(CliError::MissingToken)
        ));
    }

    #[test]
    fn absent_environment_token_uses_stored_token() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("api-key");
        save_stored_token(&path, "stored").unwrap();

        assert_eq!(resolve_token_from(None, &path).unwrap(), "stored");
    }

    #[cfg(unix)]
    #[test]
    fn unsafe_credential_paths_are_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("real-key");
        std::fs::write(&target, "secret\n").unwrap();
        let link = dir.path().join("api-key");
        std::os::unix::fs::symlink(&target, &link).unwrap();

        assert!(matches!(
            load_stored_token(&link),
            Err(CliError::Credential(_))
        ));
    }

    #[cfg(unix)]
    #[test]
    fn fifo_credential_path_is_rejected_without_a_writer() {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("api-key");
        let c_path = CString::new(path.as_os_str().as_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(c_path.as_ptr(), 0o600) }, 0);

        assert!(matches!(
            load_stored_token(&path),
            Err(CliError::Credential(_))
        ));
    }

    #[cfg(unix)]
    #[test]
    fn saved_token_has_owner_only_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("api-key");
        save_stored_token(&path, "secret").unwrap();

        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o077, 0);
    }

    fn cli_with(base_url: String, command: Command) -> Cli {
        Cli {
            base_url,
            json: false,
            command,
        }
    }
    #[test]
    fn verbose_flag_is_global_alias_for_json_output() {
        let before_subcommand = Cli::try_parse_from(["pb", "--verbose", "account"]).unwrap();
        let after_subcommand = Cli::try_parse_from(["pb", "servers", "--verbose"]).unwrap();

        assert!(before_subcommand.json);
        assert!(after_subcommand.json);
    }
    #[test]
    fn login_command_parses_without_credentials() {
        let cli = Cli::try_parse_from(["pb", "login"]).unwrap();
        assert!(matches!(cli.command, Command::Login));
    }

    #[tokio::test]
    async fn login_validates_then_saves_key_even_when_browser_open_fails() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/client/account"))
            .and(header("Authorization", "Bearer secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": 1})))
            .mount(&server)
            .await;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("api-key");

        let response = login_with(
            &server.uri(),
            &path,
            || Err(std::io::Error::other("browser unavailable")),
            || Ok(" secret ".into()),
        )
        .await
        .unwrap();

        assert_eq!(
            response,
            Response::Text("API key saved successfully.".into())
        );
        assert_eq!(load_stored_token(&path).unwrap(), Some("secret".into()));
    }

    #[tokio::test]
    async fn login_does_not_replace_key_when_validation_fails() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/client/account"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&server)
            .await;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("api-key");
        save_stored_token(&path, "old-secret").unwrap();

        assert!(
            login_with(&server.uri(), &path, || Ok(()), || Ok("new-secret".into()))
                .await
                .is_err()
        );
        assert_eq!(load_stored_token(&path).unwrap(), Some("old-secret".into()));
    }
    #[tokio::test]
    async fn login_validation_error_does_not_echo_submitted_key() {
        let server = MockServer::start().await;
        let submitted_key = "submitted-key-must-not-leak";
        Mock::given(method("GET"))
            .and(path("/api/client/account"))
            .respond_with(
                ResponseTemplate::new(401)
                    .set_body_string(format!("invalid API key: {submitted_key}")),
            )
            .mount(&server)
            .await;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("api-key");
        save_stored_token(&path, "old-secret").unwrap();

        let error = login_with(&server.uri(), &path, || Ok(()), || Ok(submitted_key.into()))
            .await
            .unwrap_err();

        assert!(!error.to_string().contains(submitted_key));
        assert_eq!(load_stored_token(&path).unwrap(), Some("old-secret".into()));
    }

    #[test]
    fn compact_response_format_is_sorted_single_line_json() {
        let output = format_response(Response::Json(json!({"z": 1, "a": 2})), true);

        assert_eq!(output, r#"{"a":2,"z":1}"#);
    }

    #[test]
    fn default_response_format_remains_pretty_json() {
        let output = format_response(Response::Json(json!({"z": 1, "a": 2})), false);

        assert_eq!(output, "{\n  \"a\": 2,\n  \"z\": 1\n}");
    }

    #[tokio::test]
    async fn api_uses_bearer_and_decodes_json() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/client"))
            .and(header("Authorization", "Bearer secret"))
            .and(header("Accept", "application/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": []})))
            .mount(&server)
            .await;
        let api = test_api(&server, "secret");
        let resp = api
            .request(Method::GET, "/api/client", &[], None)
            .await
            .unwrap();
        assert_eq!(resp, Response::Json(json!({"data": []})));
    }

    #[tokio::test]
    async fn api_reports_api_errors() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/client/account"))
            .and(header("Authorization", "Bearer secret"))
            .respond_with(
                ResponseTemplate::new(401)
                    .set_body_json(json!({"errors": [{"detail": "Unauthenticated"}]})),
            )
            .mount(&server)
            .await;
        let api = test_api(&server, "secret");
        let err = api
            .request(Method::GET, "/api/client/account", &[], None)
            .await
            .unwrap_err();
        match err {
            CliError::Api { status, message } => {
                assert_eq!(status, 401);
                assert!(message.contains("Unauthenticated"));
            }
            _ => panic!("expected Api error, got {err:?}"),
        }
    }

    #[tokio::test]
    async fn command_sends_user_command_body() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/client/servers/srv-1/command"))
            .and(header("Authorization", "Bearer secret"))
            .and(body_json(json!({"command": "say hello"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
            .mount(&server)
            .await;
        let cli = cli_with(
            server.uri(),
            Command::Command(CommandArgs {
                server_id: "srv-1".into(),
                command: "say hello".into(),
            }),
        );
        let resp = run_with_token(cli, "secret".into()).await.unwrap();
        assert_eq!(resp, Response::Json(json!({"ok": true})));
    }

    #[tokio::test]
    async fn power_sends_signal_body() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/client/servers/srv-1/power"))
            .and(header("Authorization", "Bearer secret"))
            .and(body_json(json!({"signal": "start"})))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;
        let cli = cli_with(
            server.uri(),
            Command::Power(PowerArgs {
                server_id: "srv-1".into(),
                action: "start".into(),
            }),
        );
        let resp = run_with_token(cli, "secret".into()).await.unwrap();
        assert_eq!(resp, Response::Json(Value::Null));
    }

    #[tokio::test]
    async fn resources_path_is_exact() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/client/servers/srv-1/resources"))
            .and(header("Authorization", "Bearer secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"resources": {}})))
            .mount(&server)
            .await;
        let cli = cli_with(
            server.uri(),
            Command::Resources(ServerId {
                server_id: "srv-1".into(),
            }),
        );
        let resp = run_with_token(cli, "secret".into()).await.unwrap();
        assert_eq!(resp, Response::Json(json!({"resources": {}})));
    }

    #[tokio::test]
    async fn plugins_sends_documented_query() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/client/servers/srv-1/minecraft/plugins"))
            .and(header("Authorization", "Bearer secret"))
            .and(query_param("provider", "modrinth"))
            .and(query_param("page", "2"))
            .and(query_param("page_size", "10"))
            .and(query_param("search_query", "worldedit"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": []})))
            .mount(&server)
            .await;
        let cli = cli_with(
            server.uri(),
            Command::Plugins(PluginArgs {
                server_id: "srv-1".into(),
                provider: "modrinth".into(),
                page: 2,
                page_size: 10,
                search_query: Some("worldedit".into()),
                minecraft_version: None,
            }),
        );
        let resp = run_with_token(cli, "secret".into()).await.unwrap();
        assert_eq!(resp, Response::Json(json!({"data": []})));
    }

    #[tokio::test]
    async fn search_files_sends_documented_parameters() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/client/servers/srv-1/files/search"))
            .and(header("Authorization", "Bearer secret"))
            .and(query_param("root", "/plugins"))
            .and(query_param("query", "paper"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": []})))
            .mount(&server)
            .await;
        let cli = cli_with(
            server.uri(),
            Command::FileSearch(FileSearchArgs {
                server_id: "srv-1".into(),
                query: "paper".into(),
                root: "/plugins".into(),
            }),
        );
        let resp = run_with_token(cli, "secret".into()).await.unwrap();
        assert_eq!(resp, Response::Json(json!({"data": []})));
    }

    #[tokio::test]
    async fn raw_text_success_body_is_preserved() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/client/servers/srv-1/files/contents"))
            .and(header("Authorization", "Bearer secret"))
            .and(query_param("file", "server.properties"))
            .respond_with(ResponseTemplate::new(200).set_body_string("motd=A Minecraft Server\n"))
            .mount(&server)
            .await;
        let cli = cli_with(
            server.uri(),
            Command::File(FileArgs {
                server_id: "srv-1".into(),
                path: "server.properties".into(),
            }),
        );
        let resp = run_with_token(cli, "secret".into()).await.unwrap();
        assert_eq!(resp, Response::Text("motd=A Minecraft Server\n".into()));
    }

    #[tokio::test]
    async fn run_resolves_api_key_from_environment() {
        let _environment_lock = ENVIRONMENT_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let _environment = EnvironmentRestore::capture(&["PEBBLEHOST_API_KEY"]);
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/client"))
            .and(header("Authorization", "Bearer secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": []})))
            .mount(&server)
            .await;
        let cli_for = || Cli {
            base_url: server.uri(),
            json: false,
            command: Command::Servers,
        };

        std::env::set_var("PEBBLEHOST_API_KEY", "secret");
        assert_eq!(
            run(cli_for()).await.unwrap(),
            Response::Json(json!({"data": []}))
        );

        for key in ["", "   "] {
            std::env::set_var("PEBBLEHOST_API_KEY", key);
            assert!(matches!(run(cli_for()).await, Err(CliError::MissingToken)));
        }

        assert!(matches!(run(cli_for()).await, Err(CliError::MissingToken)));
    }

    #[tokio::test]
    async fn json_flag_produces_compact_sorted_output() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/client"))
            .and(header("Authorization", "Bearer secret"))
            .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"z":1,"a":2}"#))
            .mount(&server)
            .await;
        let cli = Cli {
            base_url: server.uri(),
            json: true,
            command: Command::Servers,
        };
        let resp = run_with_token(cli, "secret".into()).await.unwrap();
        assert_eq!(resp, Response::Json(json!({"z": 1, "a": 2})));
        if let Response::Json(value) = resp {
            let sorted = sort_value(value);
            assert_eq!(serde_json::to_string(&sorted).unwrap(), r#"{"a":2,"z":1}"#);
        }
    }

    #[tokio::test]
    async fn api_call_sends_method_path_query_and_body() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/client/servers/srv-1/command"))
            .and(header("Authorization", "Bearer secret"))
            .and(query_param("dry_run", "true"))
            .and(body_json(json!({"command": "say hi"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
            .mount(&server)
            .await;

        let cli = cli_with(
            server.uri(),
            Command::ApiCall(ApiCallArgs {
                method: "POST".into(),
                path: "/api/client/servers/srv-1/command".into(),
                query: vec!["dry_run=true".into()],
                body: Some(r#"{"command":"say hi"}"#.into()),
            }),
        );
        assert_eq!(
            run_with_token(cli, "secret".into()).await.unwrap(),
            Response::Json(json!({"ok": true}))
        );
    }

    #[tokio::test]
    async fn operations_returns_bundled_api_operations() {
        let cli = cli_with("http://unused".into(), Command::Operations);
        match run(cli).await.unwrap() {
            Response::Json(value) => {
                let operations = value
                    .get("operations")
                    .and_then(Value::as_array)
                    .expect("operations array");
                assert_eq!(operations.len(), 141);
            }
            _ => panic!("expected JSON operations"),
        }
    }
}
