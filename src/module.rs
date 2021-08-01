use ext_php_rs::{
    info_table_end, info_table_row, info_table_start,
    php::{
        module::{ModuleBuilder, ModuleEntry},
    },
};

#[no_mangle]
pub extern "C" fn php_module_info(_module: *mut ModuleEntry) {
    info_table_start!();
    info_table_row!("redis-client", "enabled");
    info_table_end!();
}

#[no_mangle]
pub extern "C" fn get_module() -> *mut ModuleEntry {
    ModuleBuilder::new("redis_client", "0.1")
        .info_function(php_module_info)
        .startup_function(module_init)
        .build()
        .into_raw()
}

#[no_mangle]
pub extern "C" fn module_init(_type: i32, _module_number: i32) -> i32 {
    let rs_ex = crate::exception::RedisClientException::build();
    unsafe { crate::RS_EX.replace(rs_ex.as_mut().unwrap()) };
    crate::client::Client::get_build_for_php();

    0
}

