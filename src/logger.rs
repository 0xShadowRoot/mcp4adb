pub fn log_info(module: &str, message: &str) {
    eprintln!("[INFO][{}] {}", module, message);
}

pub fn log_error(module: &str, message: &str) {
    eprintln!("[ERROR][{}] ❌ {}", module, message);
}