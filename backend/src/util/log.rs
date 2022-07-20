use wasm_bindgen::prelude::*;

static mut LOGGER: Option<Logger> = None;

#[derive(serde::Serialize)]
struct Log {
    text: String,
    level: String,
    location: String,
}

struct Logger {
    callback: js_sys::Function,
}

impl log::Log for Logger {
    fn enabled (&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log (&self, record: &log::Record) {
        self.callback.call1(&JsValue::null(), &JsValue::from_serde(&Log {
            text: format!("{}", record.args()),
            level: format!("{}", record.level()),
            location: match (record.file(), record.line()) {
                (Some(file), Some(line))    =>  format!("{}:{}", file, line),
                _                           =>  format!("unknown"),
            },
        }).unwrap()).unwrap();
    }

    fn flush (&self) {}
}

unsafe impl Sync for Logger {}
unsafe impl Send for Logger {}

#[wasm_bindgen]
pub fn set_logger (callback: js_sys::Function) {
    unsafe {
        LOGGER = Some(Logger { callback });

        log::set_logger(LOGGER.as_ref().unwrap()).unwrap();
        log::set_max_level(log::LevelFilter::Off);
    }
}

#[wasm_bindgen]
pub fn set_log_level (level: &JsValue) {
    log::set_max_level(level.into_serde().unwrap());
}

#[wasm_bindgen]
pub fn get_log_level () -> JsValue {
    JsValue::from_serde(&log::max_level()).unwrap()
}
