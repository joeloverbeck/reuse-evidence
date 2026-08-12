use crate::terminal;

pub fn capture_status() -> i32 {
    terminal::success_status()
}

pub fn report_status() -> i32 {
    terminal::success_status()
}

pub fn capture_receipt() -> &'static str {
    "capture: complete"
}

pub fn report_receipt() -> &'static str {
    "report: complete"
}
