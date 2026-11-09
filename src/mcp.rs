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
                // ==========================================
                // 1. APPLICATION LIFECYCLE & PACKAGE MANAGEMENT
                // ==========================================
                {
                    "name": "adb_install_app",
                    "description": "Install or update an Android application package (APK) with optional flags (-g, -d, -t)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "apk_path": { "type": "string", "description": "Absolute or relative path to the .apk file on host" },
                            "grant_permissions": { "type": "boolean", "description": "Auto-grant runtime permissions (-g flag)" },
                            "allow_downgrade": { "type": "boolean", "description": "Allow version downgrade (-d flag)" },
                            "test_only": { "type": "boolean", "description": "Allow install of test-only APKs (-t flag)" }
                        },
                        "required": ["apk_path"]
                    }
                },
                {
                    "name": "adb_install_multiple",
                    "description": "Install split APKs or multiple APK bundles simultaneously (adb install-multiple)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "apk_paths": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "List of APK file paths on host"
                            },
                            "grant_permissions": { "type": "boolean", "description": "Auto-grant runtime permissions (-g flag)" }
                        },
                        "required": ["apk_paths"]
                    }
                },
                {
                    "name": "adb_uninstall_app",
                    "description": "Uninstall an application package, with an option to preserve data (-k)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "package_name": { "type": "string", "description": "Target application package identifier" },
                            "keep_data": { "type": "boolean", "description": "Keep data and cache directories around after package removal (-k)" }
                        },
                        "required": ["package_name"]
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
                    "name": "adb_list_packages",
                    "description": "List installed packages with optional filters (-3 for third-party, -s for system, etc.)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "filter": { "type": "string", "description": "Filter flags such as -3, -s, -e, -d" }
                        }
                    }
                },
                {
                    "name": "adb_dump_package",
                    "description": "Fetch complete manifest data, permissions, and component details via dumpsys package",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "package_name": { "type": "string", "description": "Target package name" }
                        },
                        "required": ["package_name"]
                    }
                },
                {
                    "name": "adb_get_package_path",
                    "description": "Retrieve absolute system APK file paths for a package",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "package_name": { "type": "string", "description": "Target package name" }
                        },
                        "required": ["package_name"]
                    }
                },
                {
                    "name": "adb_set_component_state",
                    "description": "Enable or disable specific activities, receivers, services, or providers",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "component_name": { "type": "string", "description": "Full component path (e.g., com.pkg/.MyReceiver)" },
                            "enabled": { "type": "boolean", "description": "True to enable, false to disable" }
                        },
                        "required": ["component_name", "enabled"]
                    }
                },
                {
                    "name": "adb_set_standby_bucket",
                    "description": "Force application into a specific power standby bucket (active, working_set, frequent, rare, restricted)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "package_name": { "type": "string", "description": "Target package name" },
                            "bucket": { "type": "string", "description": "Standby bucket tier" }
                        },
                        "required": ["package_name", "bucket"]
                    }
                },
                {
                    "name": "adb_set_background_restrictions",
                    "description": "Enable or disable background execution limits for the target package",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "package_name": { "type": "string", "description": "Target package name" },
                            "restricted": { "type": "boolean", "description": "True to restrict background execution, false to allow" }
                        },
                        "required": ["package_name", "restricted"]
                    }
                },

                // ==========================================
                // 2. DEVICE CONFIGURATION & ENVIRONMENT MOCKING
                // ==========================================
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
                    "name": "adb_set_time",
                    "description": "Manipulate system date/time or timezone to bypass time-bombs",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "datetime": { "type": "string", "description": "Target date string (MMDDhhmm[[CC]YY][.ss])" },
                            "timezone": { "type": "string", "description": "Target timezone string (e.g. America/New_York)" }
                        }
                    }
                },
                {
                    "name": "adb_toggle_auto_time",
                    "description": "Enable or disable network time synchronization",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "enabled": { "type": "boolean", "description": "True to enable auto-time, false to hold static" }
                        },
                        "required": ["enabled"]
                    }
                },
                {
                    "name": "adb_simulate_sensor",
                    "description": "Inject sensor events (accelerometer, gyroscope, light, proximity)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "sensor": { "type": "string", "description": "Sensor type (light, proximity, accelerometer, gyroscope)" },
                            "values": { "type": "string", "description": "Sensor data values to inject" }
                        },
                        "required": ["sensor", "values"]
                    }
                },
                {
                    "name": "adb_mock_biometric",
                    "description": "Inject success or failure callback for fingerprint/face identification triggers",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "success": { "type": "boolean", "description": "True for successful match, false for failure" }
                        },
                        "required": ["success"]
                    }
                },
                {
                    "name": "adb_set_display_size",
                    "description": "Change screen dimensions or reset to default",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "width": { "type": "integer", "description": "Display width in pixels" },
                            "height": { "type": "integer", "description": "Display height in pixels" }
                        }
                    }
                },
                {
                    "name": "adb_set_display_density",
                    "description": "Alter display DPI density or reset to default",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "density": { "type": "integer", "description": "DPI density value (e.g. 420)" }
                        }
                    }
                },
                {
                    "name": "adb_set_dark_mode",
                    "description": "Switch system UI theme state between dark and light mode",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "enabled": { "type": "boolean", "description": "True for dark mode, false for light mode" }
                        },
                        "required": ["enabled"]
                    }
                },
                {
                    "name": "adb_toggle_layout_bounds",
                    "description": "Enable or disable visual layout bounds to assist vision-based AI agents",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "enabled": { "type": "boolean", "description": "True to show layout bounds, false to hide" }
                        },
                        "required": ["enabled"]
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
                    "name": "adb_toggle_connectivity",
                    "description": "Toggle Wi-Fi, Mobile Data, and Airplane Mode status on demand",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "wifi": { "type": "boolean", "description": "True to enable Wi-Fi, false to disable" },
                            "mobile_data": { "type": "boolean", "description": "True to enable Mobile Data, false to disable" },
                            "airplane_mode": { "type": "boolean", "description": "True to enable Airplane Mode, false to disable" }
                        }
                    }
                },
                {
                    "name": "adb_keyevent",
                    "description": "Trigger physical button input keycodes (e.g., 3 for Home, 4 for Back, 26 for Power)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "key_code": { "type": "integer", "description": "Android keycode integer" }
                        },
                        "required": ["key_code"]
                    }
                },

                // ==========================================
                // 3. COMMUNICATION & TELEPHONY SIMULATION
                // ==========================================
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
                    "name": "adb_simulate_mms",
                    "description": "Inject mock multimedia messages (MMS receipt)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "sender": { "type": "string", "description": "Sender phone number" },
                            "subject": { "type": "string", "description": "MMS subject text" }
                        }
                    }
                },
                {
                    "name": "adb_simulate_sms_delivery",
                    "description": "Trigger mock SMS delivery confirmation events",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "recipient": { "type": "string", "description": "Recipient phone number" }
                        }
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
                    "name": "adb_simulate_call_state",
                    "description": "Transition telephony state across IDLE, RINGING, and OFFHOOK",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "state": { "type": "string", "description": "Telephony state (IDLE, RINGING, OFFHOOK)" },
                            "phone_number": { "type": "string", "description": "Associated phone number" }
                        },
                        "required": ["state"]
                    }
                },
                {
                    "name": "adb_simulate_ussd",
                    "description": "Trigger mock carrier messages and USSD alert popups",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "ussd_code": { "type": "string", "description": "USSD code string (e.g. *#123#)" }
                        },
                        "required": ["ussd_code"]
                    }
                },
                {
                    "name": "adb_inject_broadcast",
                    "description": "Fire arbitrary explicit and implicit intent broadcasts with custom options",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "action": { "type": "string", "description": "Intent action string" },
                            "component": { "type": "string", "description": "Optional specific component target" }
                        },
                        "required": ["action"]
                    }
                },
                {
                    "name": "adb_start_service",
                    "description": "Force start specific background service component",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "component": { "type": "string", "description": "Target service component path" }
                        },
                        "required": ["component"]
                    }
                },
                {
                    "name": "adb_run_scheduled_job",
                    "description": "Force execute pending JobScheduler jobs",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "package_name": { "type": "string", "description": "Target package name" },
                            "job_id": { "type": "integer", "description": "Job ID integer" }
                        },
                        "required": ["package_name"]
                    }
                },
                {
                    "name": "adb_trigger_alarms",
                    "description": "Inspect and fire pending AlarmManager triggers",
                    "inputSchema": { "type": "object", "properties": {} }
                },

                // ==========================================
                // 4. FILE SYSTEM & DATA FORENSICS
                // ==========================================
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

                // ==========================================
                // 5. PROCESS & MEMORY ANALYSIS
                // ==========================================
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
                },

                // ==========================================
                // 6. NETWORK MONITORING & TRAFFIC INTERCEPTION
                // ==========================================
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

                // ==========================================
                // 7. SYSTEM DIAGNOSTICS, LOGGING & TRACING
                // ==========================================
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
                    "name": "adb_logcat",
                    "description": "Fetch recent logcat output to debug application behavior or crashes",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "lines": { "type": "integer", "description": "Number of log lines to fetch" }
                        }
                    }
                },

                // ==========================================
                // 8. DEVICE AUTOMATION & UI INTERACTION
                // ==========================================
                {
                    "name": "adb_screenshot",
                    "description": "Capture a screenshot from the active device as base64 PNG data",
                    "inputSchema": { "type": "object", "properties": {} }
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