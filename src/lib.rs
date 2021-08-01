use ext_php_rs::php::class::ClassEntry;

static mut RS_EX: Option<&'static mut ClassEntry> = None;

mod exception;
mod client;
mod common;
mod module;
