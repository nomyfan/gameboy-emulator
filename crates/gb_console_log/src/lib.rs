use std::str::FromStr;

use env_filter::{Builder, Filter};
use log::{Level, LevelFilter, Log};
use web_sys::{console, wasm_bindgen::JsValue};

const TRACE_STYLE: &str = "color: white; padding: 0 3px; background: gray;";
const DEBUG_STYLE: &str = "color: white; padding: 0 3px; background: blue;";
const INFO_STYLE: &str = "color: white; padding: 0 3px; background: green;";
const WARN_STYLE: &str = "color: white; padding: 0 3px; background: orange;";
const ERROR_STYLE: &str = "color: white; padding: 0 3px; background: darkred;";
const TARGET_STYLE: &str = "font-weight: bold; color: inherit";
const ARGS_STYLE: &str = "background: inherit; color: inherit";

pub struct ConsoleLog {
    filter: Filter,
}

impl Log for ConsoleLog {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.filter.enabled(metadata)
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let level = record.level();
        let message = format!(
            "%c{}%c {}:{}%c {}",
            level,
            record.file().unwrap_or_else(|| record.target()),
            record.line().map_or_else(|| "[Unknown]".to_string(), |line| line.to_string()),
            record.args(),
        );

        let jsv_message = JsValue::from_str(&message);
        let jsv_target_style = JsValue::from_str(TARGET_STYLE);
        let jsv_args_style = JsValue::from_str(ARGS_STYLE);

        match level {
            Level::Error => console::error_4(
                &jsv_message,
                &JsValue::from_str(ERROR_STYLE),
                &jsv_target_style,
                &jsv_args_style,
            ),
            Level::Warn => console::warn_4(
                &jsv_message,
                &JsValue::from_str(WARN_STYLE),
                &jsv_target_style,
                &jsv_args_style,
            ),
            Level::Info => console::info_4(
                &jsv_message,
                &JsValue::from_str(INFO_STYLE),
                &jsv_target_style,
                &jsv_args_style,
            ),
            Level::Debug => console::log_4(
                &jsv_message,
                &JsValue::from_str(DEBUG_STYLE),
                &jsv_target_style,
                &jsv_args_style,
            ),
            Level::Trace => console::log_4(
                &jsv_message,
                &JsValue::from_str(TRACE_STYLE),
                &jsv_target_style,
                &jsv_args_style,
            ),
        }
    }

    fn flush(&self) {}
}

pub fn init(max_level: &str, filters: &str) {
    let filter = Builder::new().parse(filters).build();
    let logger = ConsoleLog { filter };

    log::set_boxed_logger(Box::new(logger)).unwrap();
    log::set_max_level(LevelFilter::from_str(max_level).unwrap());
}
