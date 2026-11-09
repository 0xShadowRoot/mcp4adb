use crate::logger::{log_error, log_info};
use anyhow::Result;
use radb::prelude::*;
use serde_json::Value;

pub fn execute_adb_tool(name: &str, args: &Value) -> Result<String> {
    log_info(
        "ADB",
        &format!("Attempting connection to ADB daemon at 127.0.0.1:5037"),
    );

    let mut client = match AdbClient::connect("127.0.0.1:5037") {
        Ok(c) => {
            log_info("ADB", "Successfully connected to ADB daemon.");
            c
        }
        Err(e) => {
            log_error(
                "ADB",
                &format!(
                    "Connection failed. Is the adb server running? Error: {}",
                    e
                ),
            );
            return Err(e.into());
        }
    };

    match name {
        "adb_list_devices" => {
            log_info("ADB", "Executing: list connected devices");
            let devices = client.list_devices()?;
            if devices.is_empty() {
                log_info("ADB", "Query complete: No Android devices found.");
                Ok("No Android devices connected.".to_string())
            } else {
                log_info("ADB", &format!("Query complete: Found {} device(s).", devices.len()));
                let list: Vec<String> = devices.into_iter().map(|d| format!("{:?}", d)).collect();
                Ok(list.join("\n"))
            }
        }
        "adb_shell" => {
            let cmd = args
                .get("command")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    log_error("ADB", "Missing required argument 'command' for adb_shell");
                    anyhow::anyhow!("Missing 'command' argument")
                })?;

            log_info("ADB", &format!("Executing shell command on device: {}", cmd));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| {
                log_error("ADB", "Shell execution aborted: No active device found");
                anyhow::anyhow!("No active ADB device found")
            })?;

            let output = device.shell(cmd)?;
            log_info("ADB", "Shell command executed successfully.");
            Ok(output)
        }
        "adb_screenshot" => {
            log_info("ADB", "Initiating screen capture sequence...");
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| {
                log_error("ADB", "Screenshot aborted: No active device found");
                anyhow::anyhow!("No active ADB device found")
            })?;

            let output = device.shell("screencap -p | base64")?;
            log_info("ADB", "Screenshot successfully captured and encoded to base64.");
            Ok(format!("data:image/png;base64,\n{}", output.trim()))
        }
        "adb_logcat" => {
            let lines = args.get("lines").and_then(|v| v.as_i64()).unwrap_or(50);
            log_info("ADB", &format!("Fetching last {} lines of system logcat...", lines));

            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| {
                log_error("ADB", "Logcat aborted: No active device found");
                anyhow::anyhow!("No active ADB device found")
            })?;

            let cmd = format!("logcat -d -t {}", lines);
            let output = device.shell(cmd)?;
            log_info("ADB", "Logcat stream fetched successfully.");
            Ok(output)
        }
        "adb_install_app" => {
            let apk_path = args
                .get("apk_path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    log_error("ADB", "Missing required argument 'apk_path'");
                    anyhow::anyhow!("Missing 'apk_path' argument")
                })?;

            log_info("ADB", &format!("Installing APK from path: {}", apk_path));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| {
                log_error("ADB", "Installation aborted: No active device found");
                anyhow::anyhow!("No active ADB device found")
            })?;

            let output = device.install(apk_path)?;
            log_info("ADB", "APK installed successfully.");
            Ok(format!("Successfully installed APK: {:?}", output))
        }
        "adb_force_stop" => {
            let package_name = args
                .get("package_name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    log_error("ADB", "Missing required argument 'package_name'");
                    anyhow::anyhow!("Missing 'package_name' argument")
                })?;

            log_info("ADB", &format!("Force stopping package: {}", package_name));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| {
                log_error("ADB", "Force stop aborted: No active device found");
                anyhow::anyhow!("No active ADB device found")
            })?;

            let cmd = format!("am force-stop {}", package_name);
            let output = device.shell(cmd)?;
            log_info("ADB", &format!("Package {} force stopped.", package_name));
            Ok(format!("Force stopped {}. Output: {}", package_name, output))
        }
        "adb_clear_app_data" => {
            let package_name = args
                .get("package_name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    log_error("ADB", "Missing required argument 'package_name'");
                    anyhow::anyhow!("Missing 'package_name' argument")
                })?;

            log_info("ADB", &format!("Clearing user data/cache for package: {}", package_name));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| {
                log_error("ADB", "Clear data aborted: No active device found");
                anyhow::anyhow!("No active ADB device found")
            })?;

            let cmd = format!("pm clear {}", package_name);
            let output = device.shell(cmd)?;
            log_info("ADB", &format!("Cleared data for package {}.", package_name));
            Ok(format!("Cleared data for {}. Output: {}", package_name, output))
        }
        // --- TOUCH & SIMULATION TOOLS ---
        "adb_tap" => {
            let x = args.get("x").and_then(|v| v.as_i64()).ok_or_else(|| {
                log_error("ADB", "Missing required argument 'x'");
                anyhow::anyhow!("Missing 'x' coordinate")
            })?;
            let y = args.get("y").and_then(|v| v.as_i64()).ok_or_else(|| {
                log_error("ADB", "Missing required argument 'y'");
                anyhow::anyhow!("Missing 'y' coordinate")
            })?;

            log_info("ADB", &format!("Executing tap at coordinates ({}, {})", x, y));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| {
                log_error("ADB", "Tap aborted: No active device found");
                anyhow::anyhow!("No active ADB device found")
            })?;

            let cmd = format!("input tap {} {}", x, y);
            let output = device.shell(cmd)?;
            log_info("ADB", "Tap executed successfully.");
            Ok(format!("Tap executed at ({}, {}). Output: {}", x, y, output))
        }
        "adb_swipe" => {
            let x1 = args.get("x1").and_then(|v| v.as_i64()).ok_or_else(|| anyhow::anyhow!("Missing 'x1'"))?;
            let y1 = args.get("y1").and_then(|v| v.as_i64()).ok_or_else(|| anyhow::anyhow!("Missing 'y1'"))?;
            let x2 = args.get("x2").and_then(|v| v.as_i64()).ok_or_else(|| anyhow::anyhow!("Missing 'x2'"))?;
            let y2 = args.get("y2").and_then(|v| v.as_i64()).ok_or_else(|| anyhow::anyhow!("Missing 'y2'"))?;
            let duration = args.get("duration").and_then(|v| v.as_i64()).unwrap_or(300);

            log_info("ADB", &format!("Executing swipe from ({}, {}) to ({}, {}) over {}ms", x1, y1, x2, y2, duration));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| {
                log_error("ADB", "Swipe aborted: No active device found");
                anyhow::anyhow!("No active ADB device found")
            })?;

            let cmd = format!("input swipe {} {} {} {} {}", x1, y1, x2, y2, duration);
            let output = device.shell(cmd)?;
            log_info("ADB", "Swipe executed successfully.");
            Ok(format!("Swipe executed. Output: {}", output))
        }
        "adb_text" => {
            let text = args.get("text").and_then(|v| v.as_str()).ok_or_else(|| {
                log_error("ADB", "Missing required argument 'text'");
                anyhow::anyhow!("Missing 'text' argument")
            })?;

            // ADB input text replaces spaces with %s
            let formatted_text = text.replace(' ', "%s");
            log_info("ADB", &format!("Simulating typing text: {}", text));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| {
                log_error("ADB", "Text input aborted: No active device found");
                anyhow::anyhow!("No active ADB device found")
            })?;

            let cmd = format!("input text {}", formatted_text);
            let output = device.shell(cmd)?;
            log_info("ADB", "Text typed successfully.");
            Ok(format!("Text input sent. Output: {}", output))
        }
        // --- PERFORMANCE DIAGNOSTICS TOOL ---
        "adb_cpu_mem_usage" => {
            let package_name = args.get("package_name").and_then(|v| v.as_str());
            log_info("ADB", "Pulling performance metrics (CPU and Memory diagnostics)...");
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| {
                log_error("ADB", "Diagnostics aborted: No active device found");
                anyhow::anyhow!("No active ADB device found")
            })?;

            let cpu_output = device.shell("dumpsys cpuinfo")?;
            let mem_output = if let Some(pkg) = package_name {
                device.shell(format!("dumpsys meminfo {}", pkg))?
            } else {
                device.shell("dumpsys meminfo")?
            };

            log_info("ADB", "Performance metrics pulled successfully.");
            Ok(format!("=== CPU INFO ===\n{}\n\n=== MEMORY INFO ===\n{}", cpu_output, mem_output))
        }
        // --- MOCK GPS & LOCALE ---
        "adb_mock_location" => {
            let lat = args.get("latitude").and_then(|v| v.as_f64()).ok_or_else(|| anyhow::anyhow!("Missing 'latitude'"))?;
            let lon = args.get("longitude").and_then(|v| v.as_f64()).ok_or_else(|| anyhow::anyhow!("Missing 'longitude'"))?;

            log_info("ADB", &format!("Setting mock GPS coordinates to: Lat {}, Lon {}", lat, lon));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            // Works natively on emulators via geo fix
            let cmd = format!("geo fix {} {}", lon, lat);
            let output = device.shell(cmd)?;
            log_info("ADB", "Mock GPS location updated.");
            Ok(format!("Mock GPS set. Output: {}", output))
        }
        "adb_set_locale" => {
            let locale = args.get("locale").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'locale'"))?;

            log_info("ADB", &format!("Changing system locale to: {}", locale));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            // Sets locale and restarts runtime components to apply
            let cmd = format!("setprop persist.sys.locale {}; setprop ctl.restart zygote", locale);
            let output = device.shell(cmd)?;
            log_info("ADB", "Locale changed successfully.");
            Ok(format!("Locale set to {}. Output: {}", locale, output))
        }

        // --- TELEPHONY & MESSAGE SIMULATION ---
        "adb_simulate_sms" => {
            let sender = args.get("sender").and_then(|v| v.as_str()).unwrap_or("+15555215554");
            let message = args.get("message").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'message'"))?;

            log_info("ADB", &format!("Simulating incoming SMS from {}: '{}'", sender, message));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let cmd = format!("am broadcast -a android.provider.Telephony.SMS_RECEIVED --es sms_body \"{}\" --es originating_address \"{}\"", message, sender);
            let output = device.shell(cmd)?;
            log_info("ADB", "Simulated SMS broadcasted.");
            Ok(format!("SMS simulated. Output: {}", output))
        }
        "adb_simulate_call" => {
            let phone = args.get("phone_number").and_then(|v| v.as_str()).unwrap_or("+15555215554");

            log_info("ADB", &format!("Simulating incoming phone call from: {}", phone));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let cmd = format!("am start -a android.intent.action.CALL -d tel:{}", phone);
            let output = device.shell(cmd)?;
            log_info("ADB", "Incoming call simulated.");
            Ok(format!("Call simulated for {}. Output: {}", phone, output))
        }

        // --- BATTERY & POWER STATES ---
        "adb_set_battery" => {
            let level = args.get("level").and_then(|v| v.as_i64()).unwrap_or(50);
            let charging = args.get("charging").and_then(|v| v.as_bool()).unwrap_or(true);

            log_info("ADB", &format!("Altering battery: level={}%, charging={}", level, charging));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            device.shell("dumpsys battery unplug")?;
            device.shell(format!("dumpsys battery set level {}", level))?;
            let status = if charging { 2 } else { 3 }; // 2 = charging, 3 = discharging
            let output = device.shell(format!("dumpsys battery set status {}", status))?;
            
            log_info("ADB", "Battery states altered successfully.");
            Ok(format!("Battery updated to {}% (Charging: {}). Output: {}", level, charging, output))
        }

        // --- PROXY & TRAFFIC REDIRECTION ---
        "adb_set_proxy" => {
            let host = args.get("host").and_then(|v| v.as_str()).unwrap_or("");
            let port = args.get("port").and_then(|v| v.as_i64()).unwrap_or(8080);

            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let output = if host.is_empty() {
                log_info("ADB", "Clearing global HTTP proxy settings...");
                device.shell("settings put global http_proxy :0")?
            } else {
                log_info("ADB", &format!("Setting global HTTP proxy to {}:{}", host, port));
                device.shell(format!("settings put global http_proxy {}:{}", host, port))?
            };

            Ok(format!("Proxy configuration updated. Output: {}", output))
        }

        // --- TCPDUMP PACKET CAPTURE ---
        "adb_tcpdump_capture" => {
            let duration = args.get("duration").and_then(|v| v.as_i64()).unwrap_or(10);
            log_info("ADB", &format!("Starting tcpdump packet capture for {} seconds...", duration));

            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            // Assumes tcpdump is pushed to /data/local/tmp/tcpdump with chmod +x
            let cmd = format!("timeout {} /data/local/tmp/tcpdump -w /data/local/tmp/capture.pcap -s 0", duration);
            let output = device.shell(cmd)?;
            
            log_info("ADB", "Packet capture completed. File saved at /data/local/tmp/capture.pcap on device.");
            Ok(format!("Tcpdump captured for {}s. You can pull /data/local/tmp/capture.pcap using ADB.\nOutput: {}", duration, output))
        }

        // --- FORENSIC FILE & DATABASE EXTRACTION ---
        "adb_pull_file" => {
            let remote_path = args.get("remote_path").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'remote_path'"))?;
            let local_path = args.get("local_path").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'local_path'"))?;

            log_info("ADB", &format!("Pulling file from device: {} -> {}", remote_path, local_path));
            
            // Using host adb command for reliable file/directory synchronization
            let output = std::process::Command::new("adb")
                .args(["pull", remote_path, local_path])
                .output()?;

            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                log_info("ADB", "File successfully pulled to host.");
                Ok(format!("Successfully pulled {} to {}.\n{}", remote_path, local_path, stdout))
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                log_error("ADB", &format!("Failed to pull file: {}", stderr));
                Err(anyhow::anyhow!("ADB pull failed: {}", stderr))
            }
        }
        "adb_extract_app_data" => {
            let package_name = args.get("package_name").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'package_name'"))?;
            let sub_dir = args.get("sub_dir").and_then(|v| v.as_str()).unwrap_or("databases"); // databases or shared_prefs

            log_info("ADB", &format!("Extracting app '{}' directory '{}' via run-as...", package_name, sub_dir));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            // Step 1: Copy target folder inside sandbox to readable tmp location using run-as (works on debuggable apps)
            let backup_cmd = format!("run-as {} tar -czf /data/local/tmp/{}_{}.tar.gz -C /data/data/{}/ .", package_name, package_name, sub_dir, package_name);
            let _ = device.shell(backup_cmd)?;

            // Step 2: Pull the archive to host
            let remote_archive = format!("/data/local/tmp/{}_{}.tar.gz", package_name, sub_dir);
            let local_dest = format!("./{}_{}.tar.gz", package_name, sub_dir);

            let output = std::process::Command::new("adb")
                .args(["pull", &remote_archive, &local_dest])
                .output()?;

            if output.status.success() {
                log_info("ADB", &format!("App data archive extracted to host: {}", local_dest));
                Ok(format!("Successfully extracted {} data for package {}. Saved to host path: {}", sub_dir, package_name, local_dest))
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                log_error("ADB", &format!("Extraction failed (app may not be debuggable): {}", stderr));
                Err(anyhow::anyhow!("App extraction failed. Ensure the app is debuggable or device is rooted: {}", stderr))
            }
        }

        // --- MEMORY & HEAP DUMP DIAGNOSTICS ---
        "adb_dump_heap" => {
            let package_name = args.get("package_name").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'package_name'"))?;
            log_info("ADB", &format!("Triggering Java heap dump for package: {}", package_name));

            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let remote_hprof = format!("/data/local/tmp/{}_heap.hprof", package_name);
            let cmd = format!("am dumpheap {} {}", package_name, remote_hprof);
            let output = device.shell(cmd)?;

            log_info("ADB", &format!("Heap dump generated at device path: {}", remote_hprof));
            Ok(format!("Heap dump triggered for {}. Stored on device at {}.\nOutput: {}", package_name, remote_hprof, output))
        }
        "adb_memory_stats" => {
            let package_name = args.get("package_name").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'package_name'"))?;
            log_info("ADB", &format!("Gathering detailed memory statistics (dumpsys meminfo) for: {}", package_name));

            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let output = device.shell(format!("dumpsys meminfo {}", package_name))?;
            log_info("ADB", "Memory statistics collected successfully.");
            Ok(output)
        }


        _ => {
            log_error("ADB", &format!("Received call for unrecognized tool: {}", name));
            Err(anyhow::anyhow!("Unknown tool: {}", name))
        }
    }
}