mod module {
    use ext_php_rs::prelude::*;
    use ext_php_rs::php::class::ClassEntry;
    use ext_php_rs::{
        info_table_end, info_table_row, info_table_start,
        php::{
            module::{ModuleBuilder, ModuleEntry},
        },
    };

    include!("./client.rs");

    include!("./exception.rs");

    include!("./common.rs");


    #[no_mangle]
    pub extern "C" fn php_module_info(_module: *mut ModuleEntry) {
        info_table_start!();
        info_table_row!("redis-client", "enabled");
        info_table_end!();
    }

    #[php_module]
    pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
        module
            .info_function(php_module_info)
    }
}

