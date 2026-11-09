use anyhow::Result;
use radb::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead};

#[tokio::main]
async fn main() -> Result<()> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut line = String::new();

    eprintln!("MCP4ADB Server running on stdio...");

    while reader.read_line(&mut line)? > 0 {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            line.clear();
            continue;
        }

        if let Ok(request) = serde_json::from_str::<JsonRpcRequest>(trimmed) {
            handle_request(request).await;
        }

        line.clear();
    }

    Ok(())
}

#[derive(Deserialize, Debug)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

async fn handle_request(req: JsonRpcRequest) {
    let response = match req.method.as_str() {
        "initialize" => json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "mcp4adb",
                "version": "0.1.0"
            }
        }),
        "tools/list" => json!({
            "tools": [
                {
                    "name": "adb_list_devices",
                    "description": "List all connected Android devices/emulators via ADB",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                },
                {
                    "name": "adb_shell",
                    "description": "Run a shell command on the target Android device",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "command": {
                                "type": "string",
                                "description": "The shell command to execute (e.g. ls, getprop)"
                            }
                        },
                        "required": ["command"]
                    }
                }
            ]
        }),
        "tools/call" => {
            if let Some(params) = req.params {
                let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

                match execute_tool(tool_name, arguments).await {
                    Ok(output) => json!({
                        "content": [
                            {
                                "type": "text",
                                "text": output
                            }
                        ]
                    }),
                    Err(err) => json!({
                        "isError": true,
                        "content": [
                            {
                                "type": "text",
                                "text": format!("Error executing tool: {}", err)
                            }
                        ]
                    }),
                }
            } else {
                json!({ "error": "Missing parameters for tools/call" })
            }
        }
        _ => json!({ "error": format!("Method not found: {}", req.method) }),
    };

    let resp_obj = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: req.id,
        result: Some(response),
        error: None,
    };

    if let Ok(json_str) = serde_json::to_string(&resp_obj) {
        println!("{}", json_str);
    }
}

async fn execute_tool(name: &str, args: Value) -> Result<String> {
    // AdbClient::connect is synchronous in radb, so we don't .await it directly
    let mut client = AdbClient::connect("127.0.0.1:5037")?;

    match name {
        "adb_list_devices" => {
            let devices = client.list_devices()?;
            if devices.is_empty() {
                Ok("No Android devices connected.".to_string())
            } else {
                let list: Vec<String> = devices
                    .into_iter()
                    .map(|d| format!("{:?}", d))
                    .collect();
                Ok(list.join("\n"))
            }
        }
        "adb_shell" => {
            let cmd = args.get("command")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'command' argument"))?;

            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next()
                .ok_or_else(|| anyhow::anyhow!("No active ADB device found"))?;

            // Execute shell command
            let output = device.shell([cmd])?;
            Ok(output)
        }
        _ => Err(anyhow::anyhow!("Unknown tool: {}", name)),
    }
}