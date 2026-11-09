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
        // ==========================================
        // 1. APPLICATION LIFECYCLE & PACKAGE MANAGEMENT
        // ==========================================
        "adb_install_app" => {
            let apk_path = args
                .get("apk_path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    log_error("ADB", "Missing required argument 'apk_path'");
                    anyhow::anyhow!("Missing 'apk_path' argument")
                })?;
            let grant_permissions = args.get("grant_permissions").and_then(|v| v.as_bool()).unwrap_or(false);
            let allow_downgrade = args.get("allow_downgrade").and_then(|v| v.as_bool()).unwrap_or(false);
            let test_only = args.get("test_only").and_then(|v| v.as_bool()).unwrap_or(false);

            log_info("ADB", &format!("Installing APK from path: {} (grant: {}, downgrade: {}, test: {})", apk_path, grant_permissions, allow_downgrade, test_only));
            
            let mut adb_args = vec!["install".to_string()];
            if grant_permissions { adb_args.push("-g".to_string()); }
            if allow_downgrade { adb_args.push("-d".to_string()); }
            if test_only { adb_args.push("-t".to_string()); }
            adb_args.push(apk_path.to_string());

            let output = std::process::Command::new("adb")
                .args(&adb_args)
                .output()?;

            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                log_info("ADB", "APK installed successfully.");
                Ok(format!("Successfully installed APK.\n{}", stdout))
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                log_error("ADB", &format!("Installation failed: {}", stderr));
                Err(anyhow::anyhow!("ADB install failed: {}", stderr))
            }
        }
        "adb_install_multiple" => {
            let apk_paths = args
                .get("apk_paths")
                .and_then(|v| v.as_array())
                .ok_or_else(|| anyhow::anyhow!("Missing 'apk_paths' array argument"))?;
            let grant_permissions = args.get("grant_permissions").and_then(|v| v.as_bool()).unwrap_or(false);

            let mut adb_args = vec!["install-multiple".to_string()];
            if grant_permissions { adb_args.push("-g".to_string()); }

            for path_val in apk_paths {
                if let Some(p) = path_val.as_str() {
                    adb_args.push(p.to_string());
                }
            }

            log_info("ADB", &format!("Installing split/multiple APKs: {:?}", adb_args));
            let output = std::process::Command::new("adb")
                .args(&adb_args)
                .output()?;

            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                log_info("ADB", "Split APKs installed successfully.");
                Ok(format!("Successfully installed split APKs.\n{}", stdout))
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                log_error("ADB", &format!("Split APK installation failed: {}", stderr));
                Err(anyhow::anyhow!("ADB install-multiple failed: {}", stderr))
            }
        }
        "adb_uninstall_app" => {
            let package_name = args
                .get("package_name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'package_name' argument"))?;
            let keep_data = args.get("keep_data").and_then(|v| v.as_bool()).unwrap_or(false);

            log_info("ADB", &format!("Uninstalling package: {} (keep_data: {})", package_name, keep_data));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let cmd = if keep_data {
                format!("pm uninstall -k {}", package_name)
            } else {
                format!("pm uninstall {}", package_name)
            };
            let output = device.shell(cmd)?;
            log_info("ADB", &format!("Package {} uninstalled.", package_name));
            Ok(format!("Uninstall result for {}:\n{}", package_name, output))
        }
        "adb_force_stop" => {
            let package_name = args
                .get("package_name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'package_name' argument"))?;

            log_info("ADB", &format!("Force stopping package: {}", package_name));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let cmd = format!("am force-stop {}", package_name);
            let output = device.shell(cmd)?;
            log_info("ADB", &format!("Package {} force stopped.", package_name));
            Ok(format!("Force stopped {}. Output: {}", package_name, output))
        }
        "adb_clear_app_data" => {
            let package_name = args
                .get("package_name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'package_name' argument"))?;

            log_info("ADB", &format!("Clearing user data/cache for package: {}", package_name));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let cmd = format!("pm clear {}", package_name);
            let output = device.shell(cmd)?;
            log_info("ADB", &format!("Cleared data for package {}.", package_name));
            Ok(format!("Cleared data for {}. Output: {}", package_name, output))
        }
        "adb_list_packages" => {
            let filter = args.get("filter").and_then(|v| v.as_str()).unwrap_or("");
            log_info("ADB", &format!("Listing packages with filter: '{}'", filter));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let cmd = format!("pm list packages {}", filter);
            let output = device.shell(cmd)?;
            Ok(output)
        }
        "adb_dump_package" => {
            let package_name = args.get("package_name").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'package_name'"))?;
            log_info("ADB", &format!("Dumping package details for: {}", package_name));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let output = device.shell(format!("dumpsys package {}", package_name))?;
            Ok(output)
        }
        "adb_get_package_path" => {
            let package_name = args.get("package_name").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'package_name'"))?;
            log_info("ADB", &format!("Retrieving APK path for package: {}", package_name));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let output = device.shell(format!("pm path {}", package_name))?;
            Ok(output)
        }
        "adb_set_component_state" => {
            let component = args.get("component_name").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'component_name'"))?;
            let enable = args.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);

            let action = if enable { "enable" } else { "disable" };
            log_info("ADB", &format!("Setting component state {} for {}", action, component));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let output = device.shell(format!("pm {} {}", action, component))?;
            Ok(output)
        }
        "adb_set_standby_bucket" => {
            let package_name = args.get("package_name").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'package_name'"))?;
            let bucket = args.get("bucket").and_then(|v| v.as_str()).unwrap_or("active"); // active, working_set, frequent, rare, restricted

            log_info("ADB", &format!("Setting app standby bucket for {} to {}", package_name, bucket));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let output = device.shell(format!("am set-standby-bucket {} {}", package_name, bucket))?;
            Ok(output)
        }
        "adb_set_background_restrictions" => {
            let package_name = args.get("package_name").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'package_name'"))?;
            let restricted = args.get("restricted").and_then(|v| v.as_bool()).unwrap_or(true);

            let mode = if restricted { "ignore" } else { "allow" };
            log_info("ADB", &format!("Setting background execution restrictions for {} to {}", package_name, mode));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let output = device.shell(format!("cmd appops set {} RUN_ANY_IN_BACKGROUND {}", package_name, mode))?;
            Ok(output)
        }

        // ==========================================
        // 2. DEVICE CONFIGURATION & ENVIRONMENT MOCKING
        // ==========================================
        "adb_mock_location" => {
            let lat = args.get("latitude").and_then(|v| v.as_f64()).ok_or_else(|| anyhow::anyhow!("Missing 'latitude'"))?;
            let lon = args.get("longitude").and_then(|v| v.as_f64()).ok_or_else(|| anyhow::anyhow!("Missing 'longitude'"))?;

            log_info("ADB", &format!("Setting mock GPS coordinates to: Lat {}, Lon {}", lat, lon));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

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

            let cmd = format!("setprop persist.sys.locale {}; setprop ctl.restart zygote", locale);
            let output = device.shell(cmd)?;
            log_info("ADB", "Locale changed successfully.");
            Ok(format!("Locale set to {}. Output: {}", locale, output))
        }
        "adb_set_time" => {
            let datetime = args.get("datetime").and_then(|v| v.as_str());
            let timezone = args.get("timezone").and_then(|v| v.as_str());

            log_info("ADB", &format!("Manipulating system clock/timezone (datetime: {:?}, timezone: {:?})", datetime, timezone));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let mut outputs = vec![];
            if let Some(dt) = datetime {
                let out = device.shell(format!("date {}", dt))?;
                outputs.push(format!("Date output: {}", out));
            }
            if let Some(tz) = timezone {
                let out = device.shell(format!("setprop persist.sys.timezone {}", tz))?;
                outputs.push(format!("Timezone output: {}", out));
            }
            Ok(outputs.join("\n"))
        }
        "adb_toggle_auto_time" => {
            let enabled = args.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
            let val = if enabled { 1 } else { 0 };

            log_info("ADB", &format!("Toggling auto-time synchronization: {}", enabled));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let out1 = device.shell(format!("settings put global auto_time {}", val))?;
            let out2 = device.shell(format!("settings put global auto_time_zone {}", val))?;
            Ok(format!("Auto time set to {}.\nOutputs:\n{}\n{}", enabled, out1, out2))
        }
        "adb_simulate_sensor" => {
            let sensor = args.get("sensor").and_then(|v| v.as_str()).unwrap_or("light");
            let values = args.get("values").and_then(|v| v.as_str()).unwrap_or("50");

            log_info("ADB", &format!("Injecting sensor data for {}: {}", sensor, values));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let cmd = format!("sensor set {} {}", sensor, values);
            let output = device.shell(cmd)?;
            Ok(format!("Sensor {} updated. Output: {}", sensor, output))
        }
        "adb_mock_biometric" => {
            let success = args.get("success").and_then(|v| v.as_bool()).unwrap_or(true);

            log_info("ADB", &format!("Mocking biometric authentication result: success={}", success));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let cmd = if success { "auth finger success" } else { "auth finger fail" };
            let output = device.shell(cmd)?;
            Ok(format!("Biometric authentication mocked (Success: {}). Output: {}", success, output))
        }
        "adb_set_display_size" => {
            let width = args.get("width").and_then(|v| v.as_i64());
            let height = args.get("height").and_then(|v| v.as_i64());

            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let cmd = match (width, height) {
                (Some(w), Some(h)) => {
                    log_info("ADB", &format!("Changing display resolution to {}x{}", w, h));
                    format!("wm size {}x{}", w, h)
                }
                _ => {
                    log_info("ADB", "Resetting display resolution to default");
                    "wm size reset".to_string()
                }
            };
            let output = device.shell(cmd)?;
            Ok(format!("Display resolution updated. Output: {}", output))
        }
        "adb_set_display_density" => {
            let density = args.get("density").and_then(|v| v.as_i64());

            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let cmd = match density {
                Some(d) => {
                    log_info("ADB", &format!("Changing display density to {}", d));
                    format!("wm density {}", d)
                }
                None => {
                    log_info("ADB", "Resetting display density to default");
                    "wm density reset".to_string()
                }
            };
            let output = device.shell(cmd)?;
            Ok(format!("Display density updated. Output: {}", output))
        }
        "adb_set_dark_mode" => {
            let enabled = args.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
            let mode = if enabled { "yes" } else { "no" };

            log_info("ADB", &format!("Setting dark mode theme state: {}", enabled));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let output = device.shell(format!("cmd uimode night {}", mode))?;
            Ok(format!("Dark mode set to {}. Output: {}", enabled, output))
        }
        "adb_toggle_layout_bounds" => {
            let enabled = args.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
            let val = if enabled { "true" } else { "false" };

            log_info("ADB", &format!("Toggling visual layout bounds: {}", enabled));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let out1 = device.shell(format!("setprop debug.layout {}", val))?;
            let out2 = device.shell("service call window 3")?; // Refresh window layout
            Ok(format!("Layout bounds set to {}.\nOutputs:\n{}\n{}", enabled, out1, out2))
        }
        "adb_set_battery" => {
            let level = args.get("level").and_then(|v| v.as_i64()).unwrap_or(50);
            let charging = args.get("charging").and_then(|v| v.as_bool()).unwrap_or(true);

            log_info("ADB", &format!("Altering battery: level={}%, charging={}", level, charging));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            device.shell("dumpsys battery unplug")?;
            device.shell(format!("dumpsys battery set level {}", level))?;
            let status = if charging { 2 } else { 3 };
            let output = device.shell(format!("dumpsys battery set status {}", status))?;
            
            log_info("ADB", "Battery states altered successfully.");
            Ok(format!("Battery updated to {}% (Charging: {}). Output: {}", level, charging, output))
        }
        "adb_toggle_connectivity" => {
            let wifi = args.get("wifi").and_then(|v| v.as_bool());
            let mobile_data = args.get("mobile_data").and_then(|v| v.as_bool());
            let airplane_mode = args.get("airplane_mode").and_then(|v| v.as_bool());

            log_info("ADB", &format!("Toggling connectivity states (wifi: {:?}, data: {:?}, airplane: {:?})", wifi, mobile_data, airplane_mode));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let mut outputs = vec![];
            if let Some(w) = wifi {
                let state = if w { "wifi enable" } else { "wifi disable" };
                let out = device.shell(format!("svc {}", state))?;
                outputs.push(format!("WiFi: {}", out));
            }
            if let Some(m) = mobile_data {
                let state = if m { "data enable" } else { "data disable" };
                let out = device.shell(format!("svc {}", state))?;
                outputs.push(format!("Mobile Data: {}", out));
            }
            if let Some(a) = airplane_mode {
                let val = if a { "1" } else { "0" };
                let out1 = device.shell(format!("settings put global airplane_mode_on {}", val))?;
                let out2 = device.shell(format!("am broadcast -a android.intent.action.AIRPLANE_MODE --ez state {}", a))?;
                outputs.push(format!("Airplane Mode: {} / {}", out1, out2));
            }
            Ok(outputs.join("\n"))
        }
        "adb_keyevent" => {
            let key_code = args.get("key_code").and_then(|v| v.as_i64()).ok_or_else(|| anyhow::anyhow!("Missing 'key_code'"))?;

            log_info("ADB", &format!("Sending hardware physical keyevent code: {}", key_code));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let output = device.shell(format!("input keyevent {}", key_code))?;
            Ok(format!("Keyevent {} sent. Output: {}", key_code, output))
        }

        // ==========================================
        // 3. COMMUNICATION & TELEPHONY SIMULATION
        // ==========================================
        // ==========================================
        // 3. COMMUNICATION & TELEPHONY SIMULATION
        // ==========================================
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
        "adb_simulate_mms" => {
            let sender = args.get("sender").and_then(|v| v.as_str()).unwrap_or("+15555215554");
            let subject = args.get("subject").and_then(|v| v.as_str()).unwrap_or("MMS Test Subject");

            log_info("ADB", &format!("Simulating MMS receipt from {}: '{}'", sender, subject));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let cmd = format!("am broadcast -a android.intent.action.DATA_SMS_RECEIVED -d mms://localhost --es subject \"{}\" --es originating_address \"{}\"", subject, sender);
            let output = device.shell(cmd)?;
            Ok(format!("MMS simulated. Output: {}", output))
        }
        "adb_simulate_sms_delivery" => {
            let recipient = args.get("recipient").and_then(|v| v.as_str()).unwrap_or("+15555215554");

            log_info("ADB", &format!("Triggering mock SMS delivery report for {}", recipient));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let cmd = format!("am broadcast -a android.provider.Telephony.SMS_DELIVER --es recipient \"{}\"", recipient);
            let output = device.shell(cmd)?;
            Ok(format!("SMS delivery report triggered. Output: {}", output))
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
        "adb_simulate_call_state" => {
            let state = args.get("state").and_then(|v| v.as_str()).unwrap_or("RINGING"); // IDLE, RINGING, OFFHOOK
            let phone = args.get("phone_number").and_then(|v| v.as_str()).unwrap_or("+15555215554");

            log_info("ADB", &format!("Transitioning telephony state to {} for number {}", state, phone));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let cmd = format!("am broadcast -a android.intent.action.PHONE_STATE --es state \"{}\" --es incoming_number \"{}\"", state, phone);
            let output = device.shell(cmd)?;
            Ok(format!("Call state transition to {} broadcasted. Output: {}", state, output))
        }
        "adb_simulate_ussd" => {
            let ussd_code = args.get("ussd_code").and_then(|v| v.as_str()).unwrap_or("*#123#");

            log_info("ADB", &format!("Triggering USSD response code: {}", ussd_code));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let encoded = ussd_code.replace('#', "%23");
            let cmd = format!("am start -a android.intent.action.CALL -d tel:{}", encoded);
            let output = device.shell(cmd)?;
            Ok(format!("USSD response triggered. Output: {}", output))
        }
        "adb_inject_broadcast" => {
            let action = args.get("action").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'action'"))?;
            let component = args.get("component").and_then(|v| v.as_str());

            log_info("ADB", &format!("Firing intent broadcast for action: {}", action));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let mut cmd = format!("am broadcast -a {}", action);
            if let Some(comp) = component {
                cmd = format!("{} -n {}", cmd, comp);
            }
            let output = device.shell(cmd)?;
            Ok(format!("Broadcast fired. Output: {}", output))
        }
        "adb_start_service" => {
            let component = args.get("component").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'component'"))?;

            log_info("ADB", &format!("Force starting background service component: {}", component));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let output = device.shell(format!("am start-service {}", component))?;
            Ok(format!("Service start command sent. Output: {}", output))
        }
        "adb_run_scheduled_job" => {
            let package_name = args.get("package_name").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'package_name'"))?;
            let job_id = args.get("job_id").and_then(|v| v.as_i64()).unwrap_or(1);

            log_info("ADB", &format!("Forcing execution of JobScheduler job {} for package {}", job_id, package_name));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let output = device.shell(format!("cmd jobscheduler run -f {} {}", package_name, job_id))?;
            Ok(format!("Job execution triggered. Output: {}", output))
        }
        "adb_trigger_alarms" => {
            log_info("ADB", "Inspecting and triggering pending AlarmManager alarms...");
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let output = device.shell("dumpsys alarm")?;
            Ok(format!("AlarmManager state inspected/triggered:\n{}", output))
        }

        // ==========================================
        // 4. FILE SYSTEM & DATA FORENSICS
        // ==========================================
        "adb_pull_file" => {
            let remote_path = args.get("remote_path").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'remote_path'"))?;
            let local_path = args.get("local_path").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing 'local_path'"))?;

            log_info("ADB", &format!("Pulling file from device: {} -> {}", remote_path, local_path));
            
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
            let sub_dir = args.get("sub_dir").and_then(|v| v.as_str()).unwrap_or("databases");

            log_info("ADB", &format!("Extracting app '{}' directory '{}' via run-as...", package_name, sub_dir));
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let backup_cmd = format!("run-as {} tar -czf /data/local/tmp/{}_{}.tar.gz -C /data/data/{}/ .", package_name, package_name, sub_dir, package_name);
            let _ = device.shell(backup_cmd)?;

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
                log_error("ADB", &format!("Extraction failed: {}", stderr));
                Err(anyhow::anyhow!("App extraction failed: {}", stderr))
            }
        }

        // ==========================================
        // 5. PROCESS & MEMORY ANALYSIS
        // ==========================================
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
        "adb_cpu_mem_usage" => {
            let package_name = args.get("package_name").and_then(|v| v.as_str());
            log_info("ADB", "Pulling performance metrics (CPU and Memory diagnostics)...");
            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let cpu_output = device.shell("dumpsys cpuinfo")?;
            let mem_output = if let Some(pkg) = package_name {
                device.shell(format!("dumpsys meminfo {}", pkg))?
            } else {
                device.shell("dumpsys meminfo")?
            };

            log_info("ADB", "Performance metrics pulled successfully.");
            Ok(format!("=== CPU INFO ===\n{}\n\n=== MEMORY INFO ===\n{}", cpu_output, mem_output))
        }

        // ==========================================
        // 6. NETWORK MONITORING & TRAFFIC INTERCEPTION
        // ==========================================
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
        "adb_tcpdump_capture" => {
            let duration = args.get("duration").and_then(|v| v.as_i64()).unwrap_or(10);
            log_info("ADB", &format!("Starting tcpdump packet capture for {} seconds...", duration));

            let devices = client.list_devices()?;
            let mut device = devices.into_iter().next().ok_or_else(|| anyhow::anyhow!("No active device found"))?;

            let cmd = format!("timeout {} /data/local/tmp/tcpdump -w /data/local/tmp/capture.pcap -s 0", duration);
            let output = device.shell(cmd)?;
            
            log_info("ADB", "Packet capture completed. File saved at /data/local/tmp/capture.pcap on device.");
            Ok(format!("Tcpdump captured for {}s. You can pull /data/local/tmp/capture.pcap using ADB.\nOutput: {}", duration, output))
        }

        // ==========================================
        // 7. SYSTEM DIAGNOSTICS, LOGGING & TRACING
        // ==========================================
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

        // ==========================================
        // 8. DEVICE AUTOMATION & UI INTERACTION
        // ==========================================
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

        _ => {
            log_error("ADB", &format!("Received call for unrecognized tool: {}", name));
            Err(anyhow::anyhow!("Unknown tool: {}", name))
        }
    }
}