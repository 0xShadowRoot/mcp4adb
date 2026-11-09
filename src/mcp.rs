use crate::adb_service::execute_adb_tool;
use crate::logger::{log_error, log_info};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
}

pub async fn handle_mcp_request(req: &JsonRpcRequest) -> Value {
    log_info("MCP", &format!("Routing incoming protocol method: {}", req.method));

    match req.method.as_str() {
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
                    "inputSchema": { "type": "object", "properties": {} }
                },
                {
                    "name": "adb_shell",
                    "description": "Run a shell command on the target Android device",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "command": { "type": "string", "description": "The shell command to execute" }
                        },
                        "required": ["command"]
                    }
                },
                {
                    "name": "adb_screenshot",
                    "description": "Capture a screenshot from the active device as base64 PNG data",
                    "inputSchema": { "type": "object", "properties": {} }
                },
                {
                    "name": "adb_logcat",
                    "description": "Fetch recent logcat output to debug application behavior or crashes",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "lines": { "type": "integer", "description": "Number of log lines to fetch" }
                        }
                    }
                },
                {
                    "name": "adb_install_app",
                    "description": "Install or update an Android application package (APK) on the device",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "apk_path": { "type": "string", "description": "Absolute or relative path to the .apk file on host" }
                        },
                        "required": ["apk_path"]
                    }
                },
                {
                    "name": "adb_force_stop",
                    "description": "Force stop a running Android application by its package name",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "package_name": { "type": "string", "description": "The application package identifier (e.g. com.example.app)" }
                        },
                        "required": ["package_name"]
                    }
                },
                {
                    "name": "adb_clear_app_data",
                    "description": "Clear all user data and cache for a specified package name",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "package_name": { "type": "string", "description": "The application package identifier (e.g. com.example.app)" }
                        },
                        "required": ["package_name"]
                    }
                },
                {
                    "name": "adb_tap",
                    "description": "Simulate a screen touch tap at specified screen coordinates (x, y)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "x": { "type": "integer", "description": "X coordinate on screen" },
                            "y": { "type": "integer", "description": "Y coordinate on screen" }
                        },
                        "required": ["x", "y"]
                    }
                },
                {
                    "name": "adb_swipe",
                    "description": "Simulate a touch swipe gesture from (x1, y1) to (x2, y2) with optional duration",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "x1": { "type": "integer", "description": "Start X" },
                            "y1": { "type": "integer", "description": "Start Y" },
                            "x2": { "type": "integer", "description": "End X" },
                            "y2": { "type": "integer", "description": "End Y" },
                            "duration": { "type": "integer", "description": "Duration in milliseconds (default 300)" }
                        },
                        "required": ["x1", "y1", "x2", "y2"]
                    }
                },
                {
                    "name": "adb_text",
                    "description": "Simulate typing text into the currently focused input field on the device",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "text": { "type": "string", "description": "The text string to type" }
                        },
                        "required": ["text"]
                    }
                },
                {
                "name": "adb_mock_location",
                "description": "Set mock GPS coordinates (latitude, longitude) for emulators",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "latitude": { "type": "number", "description": "Latitude value" },
                        "longitude": { "type": "number", "description": "Longitude value" }
                    },
                    "required": ["latitude", "longitude"]
                }
                },
                {
                "name": "adb_set_locale",
                "description": "Change the system locale and restart runtime (e.g., 'es_ES', 'ja_JP')",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "locale": { "type": "string", "description": "Target locale string" }
                    },
                    "required": ["locale"]
                }
                },
                {
                "name": "adb_simulate_sms",
                "description": "Simulate an incoming SMS text message from a sender phone number",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "sender": { "type": "string", "description": "Sender phone number" },
                        "message": { "type": "string", "description": "SMS message body text" }
                    },
                    "required": ["message"]
                }
                },
                {
                "name": "adb_simulate_call",
                "description": "Simulate an incoming phone call notification",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "phone_number": { "type": "string", "description": "Incoming caller phone number" }
                    }
                }
                },
                {
                "name": "adb_set_battery",
                "description": "Override device battery level percentage and charging state",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "level": { "type": "integer", "description": "Battery percentage (0 to 100)" },
                        "charging": { "type": "boolean", "description": "True if charging, false if discharging" }
                    },
                    "required": ["level", "charging"]
                }
                },
                {
                "name": "adb_set_proxy",
                "description": "Configure or clear a global HTTP proxy for interception tools (leave host empty to clear)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "host": { "type": "string", "description": "Proxy server IP/hostname (empty to clear)" },
                        "port": { "type": "integer", "description": "Proxy port (default 8080)" }
                    }
                }
                },
                {
                "name": "adb_tcpdump_capture",
                "description": "Capture raw network traffic packets using a deployed tcpdump binary on device",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "duration": { "type": "integer", "description": "Duration in seconds to run capture (default 10)" }
                    }
                }
                },

                {
                "name": "adb_pull_file",
                "description": "Pull a file or directory from the Android device to the host machine",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "remote_path": { "type": "string", "description": "Absolute path on the device (e.g. /sdcard/Download/file.txt)" },
                        "local_path": { "type": "string", "description": "Destination path on the host machine" }
                    },
                    "required": ["remote_path", "local_path"]
                }
                },
                {
                "name": "adb_extract_app_data",
                "description": "Extract private SQLite databases or shared preferences from an app sandbox via run-as",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "package_name": { "type": "string", "description": "Target application package identifier" },
                        "sub_dir": { "type": "string", "description": "Target subfolder to extract (default 'databases', or 'shared_prefs')" }
                    },
                    "required": ["package_name"]
                }
                },
                {
                "name": "adb_dump_heap",
                "description": "Trigger a Java heap memory dump (.hprof) for a running application process",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "package_name": { "type": "string", "description": "Target application package identifier" }
                    },
                    "required": ["package_name"]
                }
                },
                {
                "name": "adb_memory_stats",
                "description": "Retrieve comprehensive memory allocation and RAM usage statistics for an application",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "package_name": { "type": "string", "description": "Target application package identifier" }
                    },
                    "required": ["package_name"]
                }
                },


                {
                    "name": "adb_cpu_mem_usage",
                    "description": "Pull CPU usage and memory diagnostics (dumpsys cpuinfo & meminfo)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "package_name": { "type": "string", "description": "Optional specific package to inspect memory for" }
                        }
                    }
                }
            ]
        }),
        "tools/call" => {
            if let Some(params) = &req.params {
                let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

                log_info("MCP", &format!("Dispatching tool call request to service layer: {}", tool_name));

                match execute_adb_tool(tool_name, &arguments) {
                    Ok(output) => json!({
                        "content": [
                            {
                                "type": "text",
                                "text": output
                            }
                        ]
                    }),
                    Err(err) => {
                        log_error("MCP", &format!("Tool execution failed: {}", err));
                        json!({
                            "isError": true,
                            "content": [
                                {
                                    "type": "text",
                                    "text": format!("Error executing tool: {}", err)
                                }
                            ]
                        })
                    }
                }
            } else {
                log_error("MCP", "Malformed request: 'tools/call' missing parameters object");
                json!({ "error": "Missing parameters for tools/call" })
            }
        }
        _ => {
            log_error("MCP", &format!("Unsupported method requested: {}", req.method));
            json!({ "error": format!("Method not found: {}", req.method) })
        }
    }
}