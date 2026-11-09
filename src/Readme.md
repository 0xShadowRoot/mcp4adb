# MCP4ADB 📱

**MCP4ADB** is a modular, high-performance **Model Context Protocol (MCP)** server written in **Rust** that bridges AI models (such as **Gemini**, Claude, and Cursor) directly with the **Android Debug Bridge (ADB)**. 

Designed for deep mobile engineering, automated QA, security auditing, and runtime debugging, it equips LLMs to natively inspect devices, capture UI vision screenshots, stream logs, manage app lifecycles, simulate touch/sensor inputs, extract forensic data, and inspect performance metrics.

---

## 🌟 Capabilities & Tool Suite

MCP4ADB exposes **22 comprehensive ADB tools** over standard I/O JSON-RPC:

### 📱 1. Core Device & Diagnostics
| Tool Name | Description |
| :--- | :--- |
| **`adb_list_devices`** | Scans and lists all connected physical Android devices and emulators. |
| **`adb_shell`** | Executes arbitrary system/shell commands on the active target. |
| **`adb_screenshot`** | Captures the device UI and returns a base64-encoded PNG string for visual LLM evaluation. |
| **`adb_logcat`** | Fetches recent system and app crash logs (`logcat -d -t N`). |

### 📦 2. Application Lifecycle Management
| Tool Name | Description |
| :--- | :--- |
| **`adb_install_app`** | Installs or updates an APK file on the target device. |
| **`adb_force_stop`** | Instantly terminates a running application process by package name. |
| **`adb_clear_app_data`** | Clears user cache and data (`pm clear`) to simulate a fresh app launch. |

### 👆 3. Touch Input & Automation
| Tool Name | Description |
| :--- | :--- |
| **`adb_tap`** | Simulates a screen touch tap at specific `(x, y)` coordinates. |
| **`adb_swipe`** | Simulates a touch gesture swipe across defined coordinates with configurable timing. |
| **`adb_text`** | Types text strings into the currently focused input field. |

### 🌐 4. Environment & Network Simulation
| Tool Name | Description |
| :--- | :--- |
| **`adb_mock_location`** | Sets mock GPS coordinates (`latitude`, `longitude`) on emulators/devices. |
| **`adb_set_locale`** | Changes the system language/locale and restarts runtime components. |
| **`adb_simulate_sms`** | Simulates an incoming SMS broadcast with custom body text and sender number. |
| **`adb_simulate_call`** | Simulates an incoming phone call notification. |
| **`adb_set_battery`** | Overrides battery level percentage and toggles charging/discharging states. |
| **`adb_set_proxy`** | Sets or clears global HTTP proxy settings for interception and security tools. |
| **`adb_tcpdump_capture`** | Runs a raw network packet capture (`tcpdump`) on the device for a set duration. |

### 🔍 5. Forensics & Memory Diagnostics
| Tool Name | Description |
| :--- | :--- |
| **`adb_cpu_mem_usage`** | Pulls real-time CPU usage and memory diagnostics (`dumpsys cpuinfo` & `meminfo`). |
| **`adb_pull_file`** | Downloads files or directories from the device to the host machine. |
| **`adb_extract_app_data`** | Extracts private app sandboxes (SQLite DBs, `shared_prefs`) via `run-as`. |
| **`adb_dump_heap`** | Triggers a Java heap dump (`.hprof`) for process memory leak analysis. |
| **`adb_memory_stats`** | Retrieves detailed memory allocation and RAM usage metrics for an application. |

---

## 🏗️ Modular Architecture

MCP4ADB is structured cleanly for maintainability, high concurrency, and ease of debugging:

```text
MCP4ADB/
├── Cargo.toml
├── .github/
│   └── workflows/
│       ├── build.yml    # Continuous integration multi-platform build workflow
│       └── release.yml  # Automated cross-platform release pipeline on tag push
└── src/
    ├── main.rs         # Entry point & asynchronous stdio event loop
    ├── mcp.rs          # JSON-RPC request router, parsing, and tool schema registry
    ├── adb_service.rs  # Core business logic interfacing with the `radb` crate
    └── logger.rs       # Centralized stderr logging pipeline (prevents stdout stream corruption)