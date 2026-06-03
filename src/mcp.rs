use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead};

use crate::session::{list_sessions, load_session, search_sessions};

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub result: Option<Value>,
    pub error: Option<Value>,
}

pub async fn run_server() -> Result<()> {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                send_error(None, -32700, &format!("Parse error: {}", e))?;
                continue;
            }
        };

        if let Err(e) = handle_request(request).await {
            eprintln!("Error handling request: {}", e);
        }
    }
    Ok(())
}

async fn handle_request(req: JsonRpcRequest) -> Result<()> {
    match req.method.as_str() {
        "initialize" => {
            send_response(
                req.id,
                json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "resources": {},
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "runbookai",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                }),
            )?;
        }
        "resources/list" => {
            let sessions = list_sessions()?;
            let resources: Vec<Value> = sessions
                .into_iter()
                .map(|id| {
                    json!({
                        "uri": format!("runbook://sessions/{}", id),
                        "name": format!("Session {}", id),
                        "mimeType": "application/json"
                    })
                })
                .collect();
            send_response(req.id, json!({ "resources": resources }))?;
        }
        "resources/read" => {
            let uri = req
                .params
                .as_ref()
                .and_then(|p| p["uri"].as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing URI"))?;

            if let Some(id) = uri.strip_prefix("runbook://sessions/") {
                let session = load_session(id)?;
                send_response(
                    req.id,
                    json!({
                        "contents": [{
                            "uri": uri,
                            "mimeType": "application/json",
                            "text": serde_json::to_string(&session)?
                        }]
                    }),
                )?;
            } else {
                send_error(req.id, -32602, "Invalid URI")?;
            }
        }
        "tools/list" => {
            send_response(
                req.id,
                json!({
                    "tools": [
                        {
                            "name": "search_sessions",
                            "description": "Search through old RunbookAI sessions",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "query": { "type": "string" }
                                },
                                "required": ["query"]
                            }
                        }
                    ]
                }),
            )?;
        }
        "tools/call" => {
            let name = req
                .params
                .as_ref()
                .and_then(|p| p["name"].as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing tool name"))?;

            if name == "search_sessions" {
                let query = req
                    .params
                    .as_ref()
                    .and_then(|p| p["arguments"]["query"].as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing query argument"))?;

                let results = search_sessions(query)?;
                let text = if results.is_empty() {
                    format!("No sessions found matching: {}", query)
                } else {
                    let mut out = format!("Found {} matching sessions:\n\n", results.len());
                    for s in results {
                        out.push_str(&format!("- {} (ID: {})\n", s.title, s.id));
                    }
                    out
                };

                send_response(
                    req.id,
                    json!({
                        "content": [{
                            "type": "text",
                            "text": text
                        }]
                    }),
                )?;
            } else {
                send_error(req.id, -32601, "Tool not found")?;
            }
        }
        "notifications/initialized" => {
            // No response needed
        }
        _ => {
            send_error(req.id, -32601, "Method not found")?;
        }
    }
    Ok(())
}

fn send_response(id: Option<Value>, result: Value) -> Result<()> {
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(result),
        error: None,
    };
    println!("{}", serde_json::to_string(&response)?);
    Ok(())
}

fn send_error(id: Option<Value>, code: i32, message: &str) -> Result<()> {
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: None,
        error: Some(json!({ "code": code, "message": message })),
    };
    println!("{}", serde_json::to_string(&response)?);
    Ok(())
}
