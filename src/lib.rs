use std::{collections::HashMap, convert::TryInto, rc::Rc};

use ext_php_rs::{ZendObjectHandler, bindings::Debug, info_table_end, info_table_row, info_table_start, parse_args, php::{args::Arg, class::{ClassBuilder, ClassEntry}, exceptions, execution_data::{self, ExecutionData}, flags::{ClassFlags, MethodFlags, PropertyFlags}, function::FunctionBuilder, module::{ModuleBuilder, ModuleEntry}, types::{ZendType, array::ZendHashTable, object::{ZendClassObject, ZendObject}, zval::{self, Zval}}}, throw};
use ext_php_rs::php::enums::DataType;
use redis_client::redis::Values;

static mut RS_EX: Option<&'static mut ClassEntry> = None;

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
    let rs_ex = RedisClientException::build();
    unsafe { RS_EX.replace(rs_ex.as_mut().unwrap()) };
    Client::get_build_for_php();

    0
}

#[derive(ZendObjectHandler)]
struct RedisClientException;

impl RedisClientException {
    fn build() -> *mut ClassEntry {
        let constructor = FunctionBuilder::constructor(RedisClientException::constructor)
            .not_required()
            .arg(Arg::new("message", DataType::String))
            .arg(Arg::new("code", DataType::Long))
            .arg(Arg::new("line", DataType::String))
            .arg(Arg::new("file", DataType::String))
            .build();

        ClassBuilder::new("Rust\\Exception\\RedisClientException")
            .extends(ClassEntry::exception())
            .method(constructor, MethodFlags::Public)
            .property("message", "", PropertyFlags::Protected)
            .property("code", 0, PropertyFlags::Protected)
            .property("line", "", PropertyFlags::Protected)
            .property("file", "", PropertyFlags::Protected)
            .object_override::<Self>()
            .build()
    }

    pub extern "C" fn constructor(execute_data: &mut ExecutionData, retval: &mut Zval)
    {
        let mut message_arg = Arg::new("message", DataType::String);
        let mut code_arg = Arg::new("code", DataType::Long);
        let mut line_arg = Arg::new("line", DataType::String);
        let mut file_arg = Arg::new("file", DataType::String);

        parse_args!(execute_data, message_arg, code_arg, line_arg, file_arg);

        let message = message_arg.val::<String>();
        let code = code_arg.val::<i32>();
        let line = line_arg.val::<String>();
        let file = file_arg.val::<String>();

        let s = execute_data.get_self().unwrap();

        if let Some(message) = message {
            s.set_property("message", message).unwrap();
        }

        if let Some(code) = code {
            s.set_property("code", code).unwrap();
        }

        if let Some(line) = line {
            s.set_property("line", line).unwrap();
        }

        if let Some(file) = file {
            s.set_property("file", file).unwrap();
        }
    }
}

impl Default for RedisClientException {
    fn default() -> Self {
        Self
    }
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
                unsafe {
                    exceptions::throw(RS_EX.as_ref().unwrap(), e.to_string().as_str());
                }

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
                unsafe {
                    exceptions::throw(RS_EX.as_ref().unwrap(), "Can't get client");
                }

                return;
            }
        };

        let mut client = match mutex_client.lock() {
            Ok(client) => client,
            Err(e) => {
                unsafe {
                    exceptions::throw(RS_EX.as_ref().unwrap(), e.to_string().as_str());
                }

                return;
            }
        };

        let result = redis_client::redis::Cmd::cmd("SET")
            .arg(&key)
            .arg(&value)
            .execute(&mut client);

        if let Err(e) = result {
            unsafe {
                exceptions::throw(RS_EX.as_ref().unwrap(), e.to_string().as_str());
            }

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
                unsafe {
                    exceptions::throw(RS_EX.as_ref().unwrap(), "Can't get client");
                }

                return;
            }
        };

        let mut client = mutex_client.lock().unwrap();

        let result = redis_client::redis::Cmd::cmd("GET")
            .arg(&key)
            .execute(&mut client);

        if let Err(e) = result {
            unsafe {
                exceptions::throw(RS_EX.as_ref().unwrap(), e.to_string().as_str());
            }

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
                unsafe {
                    exceptions::throw(RS_EX.as_ref().unwrap(), error.as_str());
                }

                return;
            },
            redis_client::redis::Values::BulkString(string) => {
                zval.set_string(string);
            },
            redis_client::redis::Values::SimpleString(string) => {
                zval.set_string(string);
            },
            redis_client::redis::Values::Arrays(_) => {
                unsafe {
                    exceptions::throw(RS_EX.as_ref().unwrap(), "Array values unsupported");
                }

                return;
            }
        }
    }

    fn get_build_for_php() -> *mut ClassEntry {
        let constructor = FunctionBuilder::constructor(Client::constructor)
            .arg(Arg::new("addr", DataType::String))
            .build();

        let set = FunctionBuilder::new("set", Client::set_value)
            .arg(Arg::new("key", DataType::String))
            .arg(Arg::new("value", DataType::String))
            .returns(DataType::True, false, false)
            .build();

        let get = FunctionBuilder::new("get", Client::get_value)
            .arg(Arg::new("key", DataType::String))
            .returns(DataType::String, false, false)
            .build();

        ClassBuilder::new("Rust\\Client")
            .method(constructor, MethodFlags::Public)
            .method(set, MethodFlags::Public)
            .method(get, MethodFlags::Public)
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
