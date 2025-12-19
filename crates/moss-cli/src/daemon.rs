use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Serialize)]
#[serde(tag = "cmd")]
pub enum Request {
    #[serde(rename = "path")]
    Path { query: String },
    #[serde(rename = "symbols")]
    Symbols { file: String },
    #[serde(rename = "callers")]
    Callers { symbol: String },
    #[serde(rename = "callees")]
    Callees { symbol: String, file: String },
    #[serde(rename = "expand")]
    Expand { symbol: String, file: Option<String> },
    #[serde(rename = "status")]
    Status,
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub ok: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

pub struct DaemonClient {
    socket_path: String,
}

impl DaemonClient {
    pub fn new(root: &Path) -> Self {
        let socket_path = root.join(".moss/daemon.sock").to_string_lossy().to_string();
        Self { socket_path }
    }

    pub fn is_available(&self) -> bool {
        Path::new(&self.socket_path).exists()
    }

    pub fn query(&self, request: &Request) -> Result<Response, String> {
        let mut stream = UnixStream::connect(&self.socket_path)
            .map_err(|e| format!("Failed to connect to daemon: {}", e))?;

        stream
            .set_read_timeout(Some(Duration::from_secs(30)))
            .ok();
        stream
            .set_write_timeout(Some(Duration::from_secs(5)))
            .ok();

        let request_json = serde_json::to_string(request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;

        stream
            .write_all(request_json.as_bytes())
            .map_err(|e| format!("Failed to send request: {}", e))?;
        stream
            .write_all(b"\n")
            .map_err(|e| format!("Failed to send newline: {}", e))?;

        let mut reader = BufReader::new(stream);
        let mut response_line = String::new();
        reader
            .read_line(&mut response_line)
            .map_err(|e| format!("Failed to read response: {}", e))?;

        serde_json::from_str(&response_line)
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    pub fn path_query(&self, query: &str) -> Result<Vec<PathMatch>, String> {
        let response = self.query(&Request::Path { query: query.to_string() })?;
        if !response.ok {
            return Err(response.error.unwrap_or_else(|| "Unknown error".to_string()));
        }
        let data = response.data.ok_or("No data in response")?;
        serde_json::from_value(data).map_err(|e| format!("Failed to parse path matches: {}", e))
    }
}

#[derive(Debug, Deserialize)]
pub struct PathMatch {
    pub path: String,
    pub kind: String,
    pub score: i32,
}
