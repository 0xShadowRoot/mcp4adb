# MCP4ADB 

**MCP4ADB** is a blazing-fast, robust, and modular **Model Context Protocol (MCP)** server written in **Rust** that bridges AI models (such as **Gemini**, Claude, and Cursor) directly with the **Android Debug Bridge (ADB)**. 

Designed for deep mobile development, automated testing, and runtime debugging, it allows LLMs to natively inspect devices, capture UI screenshots, stream logs, manage app lifecycles, simulate touch inputs, and analyze performance diagnostics.

---
## Codebase Architecture

MCP4ADB is structured for clean maintainability, separation of concerns, and effortless debugging:

```text
MCP4ADB/
├── Cargo.toml
└── src/
    ├── main.rs         # Entry point & asynchronous stdio loop
    ├── mcp.rs          # JSON-RPC request router, parsing, and tool schemas
    ├── adb_service.rs  # Core business logic interfacing with the `radb` crate
    └── logger.rs       # Centralized stderr logging pipeline (prevents stdout corruption)
```
---

##  Key Features & Tools

MCP4ADB exposes **powerful ADB tools** over standard I/O JSON-RPC:

---

## 1. Application Lifecycle & Package Management
*Focus: Installation, removal, state monitoring, permission auditing, and package extraction for static and dynamic analysis.*

### Installation & Updates
- [x] **Install standard APKs:** Support single APK installation (`adb install`).
- [x] **Install split APKs:** Support multi-APK/bundle installation (`adb install-multiple`).
- [x] **Grant runtime permissions automatically:** Auto-grant permissions during installation (`-g` flag).
- [x] **Allow version downgrade:** Force install over newer version (`-d` flag).
- [x] **Install test-only packages:** Support packages signed with test keys (`-t` flag).

### Removal & Cleanup
- [x] **Uninstall packages:** Remove target application (`adb uninstall`).
- [x] **Keep data on uninstall:** Remove APK while preserving data (`-k` flag).
- [x] **Clear application data:** Reset app storage and cache without full uninstall (`adb shell pm clear <pkg>`).

### Package Inspection & Enumeration
- [x] **List installed packages:** Filter by system, 3rd-party, enabled, or disabled (`adb shell pm list packages [-3|-s|-e|-d]`).
- [x] **Dump package details:** Fetch manifest data, permissions, exported components, and intent filters (`adb shell dumpsys package <pkg>`).
- [x] **Retrieve APK file paths:** Find absolute system paths of target APKs for pulling (`adb shell pm path <pkg>`).
- [ ] **Query default handlers:** List default applications handling specific intents or MIME types.

### Process & Component Control
- [x] **Force-stop application:** Terminate app processes cleanly (`adb shell am force-stop <pkg>`).
- [x] **Enable/Disable components:** Selectively disable/enable specific activities, receivers, services, or providers (`adb shell pm disable/enable <component>`).
- [x] **Set app standby bucket:** Force application into specific power usage buckets (`active`, `working_set`, `frequent`, `rare`).
- [x] **Set background restrictions:** Enable/disable background execution limits for the target package.

---

## 2. Device Configuration & Environment Mocking
*Focus: Altering system states, hardware sensors, and environmental variables to trigger time-bombs, anti-analysis checks, or specific code paths.*

### Locale, Time & Localization
- [x] **Set system locale:** Change language and region to test geo-targeted malware triggers (`adb shell settings put global system_locales <locale>`).
- [x] **Manipulate system clock/timezone:** Change system date/time to bypass time-bomb delays or expiration checks (`adb shell date ...`).
- [x] **Toggle auto-time sync:** Disable network time synchronization to hold static system timestamps.

### Location & Sensor Spoofing
- [x] **Inject GPS coordinates:** Mock current latitude, longitude, and altitude values via location service or intents.
- [x] **Simulate accelerometer/gyroscope:** Inject motion sensor events (rotation, tilt, shaking).
- [x] **Mock light & proximity sensors:** Inject ambient light levels and proximity trigger states.
- [x] **Mock biometric authentication:** Inject success/failure callbacks for fingerprint/face identification triggers.

### Display & UI Configuration
- [x] **Change display resolution:** Scale screen dimensions for UI edge cases or dynamic layout testing (`adb shell wm size <WxH>`).
- [x] **Change display density:** Alter display DPI density (`adb shell wm density <dpi>`).
- [x] **Toggle dark/light mode:** Switch system UI theme state.
- [x] **Toggle layout bounds:** Enable visual layout bounds to assist vision-based AI agents (`adb shell setprop debug.layout true`).

### Hardware & Peripheral States
- [x] **Simulate battery level & status:** Spoof percentage, plugged status (AC, USB, Wireless), and temperature (`adb shell dumpsys battery set level/status/unplug`).
- [x] **Toggle network connectivity states:** Toggle Wi-Fi, Mobile Data, and Airplane Mode status on demand.
- [x] **Simulate physical button input:** Trigger keycodes for Power, Volume UP/DOWN, Home, Back, and Recent Apps (`adb shell input keyevent <code >`).

---

## 3. Communication & Telephony Simulation
*Focus: Injecting external communication triggers to awaken dormant malware triggers or analyze message/call-handling logic.*

### SMS & Messaging Simulation
- [x] **Inject incoming SMS:** Simulate received SMS with custom sender numbers, body text, and timestamps (`adb shell am broadcast -a android.provider.Telephony.SMS_RECEIVED`).
- [x] **Simulate MMS receipt:** Inject mock multimedia messages.
- [x] **Simulate SMS delivery reports:** Trigger mock delivery confirmation events.

### Telephony & Call Simulation
- [x] **Simulate incoming call:** Trigger incoming call state with arbitrary caller IDs.
- [x] **Simulate call state transitions:** Transition telephony state across `IDLE`, `RINGING`, and `OFFHOOK`.
- [x] **Simulate USSD responses:** Trigger mock carrier messages and USSD alert popups.

### Push Notifications & Broadcasts
- [x] **Inject custom system broadcasts:** Fire arbitrary explicit and implicit intent broadcasts with custom extras (`adb shell am broadcast`).
- [x] **Start background service directly:** Force start specific component services (`adb shell am start-service`).
- [x] **Trigger scheduled job execution:** Force execute pending JobScheduler jobs (`adb shell cmd jobscheduler run`).
- [x] **Trigger pending alarms:** Instantly fire pending AlarmManager triggers (`adb shell dumpsys alarm`).

---

## 4. File System & Data Forensics
*Focus: Extracting artifacts, monitoring persistent storage, and investigating file system alterations.*

### Artifact Extraction & Pulling
- [x] **Pull application private data:** Extract app files from `/data/data/<pkg>/` (requires root or debuggable build).
- [x] **Extract databases & SharedPreferences:** Pull SQLite databases and XML preference files for offline forensic inspection.
- [x] **Pull system logs & crash dumps:** Retrieve device tombstone crash files and comprehensive bugreports (`adb bugreport`).
- [x] **Pull external storage data:** Extract files from public `/sdcard/` locations.

### File Injection & Modification
- [ ] **Push files/payloads:** Upload scripts, tools, dynamic libraries, or sample media (`adb push <local> <remote>`).
- [ ] **Modify permissions & ownership:** Alter target permissions via shell (`chmod`, `chown`).
- [ ] **Touch timestamps:** Modify file access and modification times (`touch -t`).

### Live File System Monitoring
- [ ] **Monitor directory changes:** Run dynamic file alteration monitoring using `inotifywait` or polling loops.
- [ ] **Calculate file hashes on-device:** Generate `sha256sum` or `md5sum` directly on device target files before/after execution.

---

## 5. Process & Memory Analysis
*Focus: Introspecting active processes, tracking memory consumption, and dumping memory for analysis.*

### Process Enumeration & Inspection
- [ ] **List running processes:** Fetch PID, UID, and process names (`adb shell ps -ef` or `adb shell top`).
- [ ] **Inspect process memory maps:** Retrieve runtime memory region layouts (`adb shell cat /proc/<pid>/maps`).
- [x] **Dump process heap:** Generate and pull Java/Dalvik HPROF heap dumps (`adb shell am dumpheap <pid> <path>`).

### Debugging & Profiling Controls
- [ ] **Launch app in wait-for-debugger mode:** Start target activity suspended until debugger attaches (`adb shell am start -D -n <activity>`).
- [ ] **Attach JDWP forwarder:** List debuggable PIDs and forward JDWP connections (`adb forward tcp:<port> jdwp:<pid>`).
- [ ] **Trigger process crashes/ANRs:** Force trigger crash dumps or ANRs for diagnostic capture.

---

## 6. Network Monitoring & Traffic Interception
*Focus: Analyzing network activity, proxy configurations, and traffic redirection.*

### Network State & Connection Auditing
- [ ] **List open network sockets:** View active TCP/UDP connections and associated process PIDs (`adb shell netstat -tulnp` or `ss`).
- [x] **Capture raw packet traffic:** Stream or record PCAP files on device interfaces using `tcpdump`.

### Proxy & Certificate Management
- [x] **Set system-wide HTTP proxy:** Direct HTTP traffic to intercepting proxies like Burp/ZAP (`adb shell settings put global http_proxy <ip>:<port>`).
- [x] **Clear system HTTP proxy:** Unset global proxy settings.
- [ ] **Install custom CA certificate:** Push user or root CA certificates to system/user trust stores.
- [ ] **Audit network policy controls:** Inspect background network usage restrictions and VPN configurations.

---

## 7. System Diagnostics, Logging & Tracing
*Focus: Capturing system-wide telemetry, debugging logs, and performance metrics.*

### Logcat Management
- [x] **Stream system logs:** Capture real-time log output (`adb logcat`).
- [ ] **Filter logcat output:** Filter logs by PID, tag, severity priority (`V, D, I, W, E, F`), or regex.
- [ ] **Clear logcat buffer:** Flush log buffers prior to executing test steps (`adb logcat -c`).
- [ ] **Dump crash/event buffers:** Separately read `events`, `radio`, `system`, or `crash` log buffers.

### System State Inspection (`dumpsys`)
- [ ] **Dump activity hierarchy:** Inspect backstack and active window states (`adb shell dumpsys activity activities`).
- [ ] **Dump battery stats:** Obtain detailed power draw breakdown by app (`adb shell dumpsys batterystats`).
- [ ] **Dump notification history:** Retrieve history of active and posted system notifications (`adb shell dumpsys notification`).

### Performance & Tracing
- [ ] **Capture Perfetto/Systrace logs:** Record system-wide performance and kernel trace files.
- [x] **Monitor CPU/RAM metrics:** Track per-process memory (`adb shell dumpsys meminfo <pkg>`) and CPU consumption.

---

## 8. Device Automation & UI Interaction
*Focus: Driving UI workflows automatically for fuzzing or dynamic analysis navigation.*

### Input & Gesture Injection
- [x] **Inject tap/click:** Perform screen coordinate taps (`adb shell input tap x y`).
- [x] **Inject swipe/drag:** Perform swipe gestures with configurable duration (`adb shell input swipe x1 y1 x2 y2 [duration]`).
- [x] **Inject text input:** Type text strings into active input fields (`adb shell input text <string>`).
- [ ] **Inject key events:** Send navigation hardware key events (Home, Back, Enter, Tab, Del).

### Visual & Screen Capture
- [x] **Take screen capture:** Capture PNG screenshot of current screen state (`adb shell screencap -p`).
- [ ] **Record screen video:** Record raw MP4 screen video for session auditing (`adb shell screenrecord`).

### UI Structure Inspection
- [ ] **Dump UI hierarchy XML:** Export exact UI node tree with component resource IDs, bounds, and text using UIAutomator (`adb shell uiautomator dump`).

---

## 9. Security Controls, Root & Privilege Management
*Focus: Evaluating device hardening, root status, and SELinux policies.*

### Privilege & Elevation Controls
- [ ] **Restart ADB daemon as root:** Elevate ADB daemon to root privileges (`adb root`).
- [ ] **Unroot ADB daemon:** Reset ADB daemon to unprivileged mode (`adb unroot`).
- [ ] **Execute root shell commands:** Execute commands with root context via `su` (`adb shell su -c '<command>'`).

### SELinux & Hardening Configuration
- [ ] **Check SELinux mode:** Query current SELinux status (`adb shell getenforce`).
- [ ] **Set SELinux mode:** Toggle SELinux between `Enforcing` (1) and `Permissive` (0) (`adb shell setenforce 0|1`).
- [ ] **Remount system partition:** Remount `/system` as read-write (`adb remount`).

### Security Settings Auditing
- [ ] **Check USB debugging authorization:** Verify connected client key status.
- [ ] **Inspect Device Admin / Owner states:** Query active Device Administrator apps and Enterprise Device Owners (`adb shell dumpsys device_policy`).
- [ ] **Inspect trust stores:** Verify system and user installed CA certificates.

---

## 10. Core ADB Connection & Control 
- [x] **List devices and adb shell**
---
