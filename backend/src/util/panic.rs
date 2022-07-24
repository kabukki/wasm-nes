use wasm_bindgen::prelude::*;

static mut LOGGER: Option<Logger> = None;

struct Logger {
    callback: js_sys::Function,
}

impl Logger {
    pub fn log (&self, info: &String) {
        self.callback.call1(&JsValue::null(), &JsValue::from_str(info)).unwrap();
    }
}

#[wasm_bindgen]
pub fn set_panic_hook (callback: js_sys::Function) {
    unsafe {
        LOGGER = Some(Logger { callback });

        std::panic::set_hook(Box::new(|info: &std::panic::PanicInfo| {
            log::error!("{}", info);
            LOGGER.as_ref().unwrap().log(&format!("{}", info));
        }));
    }
}
