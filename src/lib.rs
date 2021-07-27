use std::{collections::HashMap, convert::TryInto, rc::Rc};

use ext_php_rs::{ZendObjectHandler, info_table_end, info_table_row, info_table_start, parse_args, php::{args::Arg, class::{ClassBuilder, ClassEntry}, exceptions, execution_data::{self, ExecutionData}, flags::{MethodFlags, PropertyFlags}, function::FunctionBuilder, module::{ModuleBuilder, ModuleEntry}, types::{array::ZendHashTable, object::{ZendClassObject, ZendObject}, zval::{self, Zval}}}, throw};
use ext_php_rs::php::enums::DataType;
use redis_client::redis::Values;

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
pub extern "C" fn module_init(_type: i32, module_number: i32) -> i32 {
    Client::get_build_for_php();

    0
}

#[derive(ZendObjectHandler)]
struct Client {
    client: Option<Rc<std::sync::Mutex<redis_client::redis::Client>>>,
}

impl Client {
    pub extern "C" fn constructor(execute_data: &mut ExecutionData, _retval: &mut Zval) {
        let mut addr_arg = Arg::new("add", DataType::String);

        parse_args!(execute_data, addr_arg);

        let addr = addr_arg.val::<String>().unwrap();

        let this = ZendClassObject::<Self>::get(execute_data).unwrap();

        execute_data.get_self().unwrap().set_property("addr", &addr).unwrap();
        let client = match redis_client::redis::Client::new(addr) {
            Ok(client) => client,
            Err(e) => {
                exceptions::throw(ClassEntry::exception(), &e.to_string());

                return;
            }
        };

        this.client = Some(Rc::new(std::sync::Mutex::new(client)));
    }

    pub extern "C" fn set_value(execute_data: &mut ExecutionData, _retval: &mut Zval) {
        let mut key_arg = Arg::new("key", DataType::String);
        let mut value_arg = Arg::new("value", DataType::String);

        parse_args!(execute_data, key_arg, value_arg);

        let key = key_arg.val::<String>().unwrap();
        let value = value_arg.val::<String>().unwrap();

        let this = ZendClassObject::<Self>::get(execute_data).unwrap();

        let mutex_client = match this.client.as_mut() {
            Some(client) => client,
            None => {
                exceptions::throw(ClassEntry::exception(), "Can't get client");

                return;
            }
        };

        let mut client = match mutex_client.lock() {
            Ok(client) => client,
            Err(e) => {
                exceptions::throw(ClassEntry::exception(), e.to_string().as_str());

                return;
            }
        };

        let result = redis_client::redis::Cmd::cmd("SET")
            .arg(&key)
            .arg(&value)
            .execute(&mut client);

        if let Err(e) = result {
            exceptions::throw(ClassEntry::exception(), e.to_string().as_str());

            return;
        }
    }

    pub extern "C" fn get_value(execute_data: &mut ExecutionData, retval: &mut Zval) {
        let mut key_arg = Arg::new("key", DataType::String);

        parse_args!(execute_data, key_arg);

        let key = key_arg.val::<String>().unwrap();

        let this = ZendClassObject::<Self>::get(execute_data).unwrap();

        let mutex_client = match this.client.as_mut() {
            Some(client) => client,
            None => {
                exceptions::throw(ClassEntry::exception(), "Can't get client");

                return;
            }
        };

        let mut client = mutex_client.lock().unwrap();

        let result = redis_client::redis::Cmd::cmd("GET")
            .arg(&key)
            .execute(&mut client);

        if let Err(e) = result {
            exceptions::throw(ClassEntry::exception(), e.to_string().as_str());

            return;
        }

        Self::values_to_zval(result.unwrap(), retval);
    }

    fn values_to_zval(value: Values, zval: &mut Zval) {
        match value {
            redis_client::redis::Values::Integers(int) => {
                zval.set_long(int);
            },
            redis_client::redis::Values::Errors(error) => {
                exceptions::throw(ClassEntry::exception(), error.as_str());

                return;
            },
            redis_client::redis::Values::BulkString(string) => {
                zval.set_string(string);
            },
            redis_client::redis::Values::SimpleString(string) => {
                zval.set_string(string);
            },
            redis_client::redis::Values::Arrays(values) => {
                exceptions::throw(ClassEntry::exception(), "Array values unsupported");

                return;
            }
        }
    }

    fn get_build_for_php() -> *mut ClassEntry {
        let constructor = FunctionBuilder::constructor(Client::constructor)
            .arg(Arg::new("addr", DataType::String))
            .build();

        let set_value = FunctionBuilder::new("setValue", Client::set_value)
            .arg(Arg::new("key", DataType::String))
            .arg(Arg::new("value", DataType::String))
            .returns(DataType::True, false, false)
            .build();

        let get_value = FunctionBuilder::new("getValue", Client::get_value)
            .arg(Arg::new("key", DataType::String))
            .returns(DataType::String, false, false)
            .build();

        ClassBuilder::new("Rust\\Client")
            .method(constructor, MethodFlags::Public)
            .method(set_value, MethodFlags::Public)
            .method(get_value, MethodFlags::Public)
            .property("addr", "", PropertyFlags::Private)
            .object_override::<Self>()
            .build()
    }
}

impl Default for Client {
    fn default() -> Self {
        Self {
            client: None,
        }
    }
}
