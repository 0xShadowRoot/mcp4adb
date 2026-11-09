# MCP4ADB 🚀📱

**MCP4ADB** is a blazing-fast, robust, and modular **Model Context Protocol (MCP)** server written in **Rust** that bridges AI models (such as **Gemini**, Claude, and Cursor) directly with the **Android Debug Bridge (ADB)**. 

Designed for deep mobile development, automated testing, and runtime debugging, it allows LLMs to natively inspect devices, capture UI screenshots, stream logs, manage app lifecycles, simulate touch inputs, and analyze performance diagnostics.

---

## 🌟 Key Features & Tools

MCP4ADB exposes **11 powerful ADB tools** over standard I/O JSON-RPC:

| Tool Name | Description |
| :--- | :--- |
| **`adb_list_devices`** | Scans and lists all connected physical Android devices and emulators. |
| **`adb_shell`** | Executes arbitrary system/shell commands on the active target. |
| **`adb_screenshot`** | Captures the device UI and returns a base64-encoded PNG (ideal for multimodal LLMs like Gemini). |
| **`adb_logcat`** | Fetches recent system and app crash logs (`logcat -d -t N`). |
| **`adb_install_app`** | Installs or updates an APK file onto the device. |
| **`adb_force_stop`** | Instantly kills a running application process by package name. |
| **`adb_clear_app_data`** | Clears user cache and data (`pm clear`) for fresh environment testing. |
| **`adb_tap`** | Simulates a screen touch tap at specific `(x, y)` coordinates. |
| **`adb_swipe`** | Simulates a touch gesture swipe across defined coordinates with custom timing. |
| **`adb_text`** | Automatically types text strings into focused input fields on the device. |
| **`adb_cpu_mem_usage`** | Pulls real-time CPU usage and memory diagnostics (`dumpsys cpuinfo` & `meminfo`). |

---

## 🏗️ Modular Architecture

MCP4ADB is structured for clean maintainability, separation of concerns, and effortless debugging:

```text
MCP4ADB/
├── Cargo.toml
└── src/
    ├── main.rs         # Entry point & asynchronous stdio loop
    ├── mcp.rs          # JSON-RPC request router, parsing, and tool schemas
    ├── adb_service.rs  # Core business logic interfacing with the `radb` crate
    └── logger.rs       # Centralized stderr logging pipeline (prevents stdout corruption)