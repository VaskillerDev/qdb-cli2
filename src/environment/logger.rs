use crate::environment::about::get_app_config;

pub struct Logger;

impl Logger {
    pub fn error(stream: &str) {
        error!("{}", stream);
    }
    pub fn warn(stream: &str) {
        warn!("{}", stream);
    }
    pub fn info(stream: &str) {
        info!("{}", stream);
    }
    pub fn debug(stream: &str) {
        let debug: bool = get_app_config()
            .value_of("debug")
            .unwrap()
            .parse::<bool>()
            .unwrap();
        if debug == true {
            debug!("{}", stream);
        }
    }
    pub fn trace(stream: &str) {
        trace!("{}", stream);
    }
}
