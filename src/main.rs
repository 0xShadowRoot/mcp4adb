#![recursion_limit = "256"]
mod adb_service;
mod logger;
mod mcp;

use anyhow::Result;
use logger::{log_error, log_info};
use mcp::{handle_mcp_request, JsonRpcRequest, JsonRpcResponse};
use std::io::{self, BufRead};

#[tokio::main]
async fn main() -> Result<()> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut line = String::new();

    log_info("MAIN", "MCP4ADB server successfully initialized and listening on stdio.");

    while reader.read_line(&mut line)? > 0 {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            line.clear();
            continue;
        }

        match serde_json::from_str::<JsonRpcRequest>(trimmed) {
            Ok(request) => {
                let req_id = request.id.clone();
                let response_result = handle_mcp_request(&request).await;

                let resp_obj = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req_id,
                    result: Some(response_result),
                    error: None,
                };

                // Output valid JSON-RPC back to stdout for the LLM client
                if let Ok(json_str) = serde_json::to_string(&resp_obj) {
                    println!("{}", json_str);
                }
            }
            Err(e) => {
                log_error("MAIN", &format!("Failed to parse incoming JSON payload: {}", e));
            }
        }

        line.clear();
    }

    Ok(())
}