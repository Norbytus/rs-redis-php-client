use ext_php_rs::php::enums::DataType;
use ext_php_rs::{
    parse_args,
    php::{
        args::Arg,
        class::{ClassBuilder, ClassEntry},
        exceptions,
        execution_data::ExecutionData,
        flags::{MethodFlags, PropertyFlags},
        function::FunctionBuilder,
        types::{
            object::ZendClassObject,
            zval::Zval
        },
    },
    ZendObjectHandler,
};
use redis_client::redis::Values;
use std::rc::Rc;
use crate::common::arg_to_string;

type ConnectionWrap = Option<Rc<std::sync::Mutex<redis_client::redis::Client>>>;

#[derive(ZendObjectHandler)]
pub struct Client {
    client: ConnectionWrap,
}

impl Client {
    pub extern "C" fn constructor(execute_data: &mut ExecutionData, _: &mut Zval) {
        let mut addr_arg = Arg::new("addr", DataType::String);

        parse_args!(execute_data, addr_arg);

        let mut addr = String::new();
        arg_to_string(addr_arg, "addr", &mut addr);

        let this = ZendClassObject::<Self>::get(execute_data).unwrap();

        execute_data
            .get_self()
            .unwrap()
            .set_property("addr", &addr)
            .unwrap();

        let client = if let Ok(client) = redis_client::redis::Client::new(addr) {
            client
        } else {
            unsafe {
                exceptions::throw(crate::RS_EX.as_ref().unwrap(), "Connection error");
            }

            return;
        };

        this.client = Some(Rc::new(std::sync::Mutex::new(client)));
    }

    pub extern "C" fn set_value(execute_data: &mut ExecutionData, _: &mut Zval) {
        let mut key_arg = Arg::new("key", DataType::String);
        let mut value_arg = Arg::new("value", DataType::String);

        parse_args!(execute_data, key_arg, value_arg);

        let mut key = String::new();
        arg_to_string(key_arg, "key", &mut key);
        let mut value = String::new();
        arg_to_string(value_arg, "value", &mut value);

        let this = ZendClassObject::<Self>::get(execute_data).unwrap();

        let mutex_client = if let Some(client) = this.client.as_mut() {
            client
        } else {
            unsafe {
                exceptions::throw(crate::RS_EX.as_ref().unwrap(), "Can't get client");
            }

            return;
        };

        let mut client = if let Ok(client) = mutex_client.lock() {
            client
        } else {
            unsafe {
                exceptions::throw(crate::RS_EX.as_ref().unwrap(), "Connection already use");
            }

            return;
        };

        let result = redis_client::redis::Cmd::cmd("SET")
            .arg(&key)
            .arg(&value)
            .execute(&mut client);

        if let Err(e) = result {
            unsafe {
                exceptions::throw(crate::RS_EX.as_ref().unwrap(), e.to_string().as_str());
            }

            return;
        }
    }

    pub extern "C" fn get_value(execute_data: &mut ExecutionData, return_value: &mut Zval) {
        let mut key_arg = Arg::new("key", DataType::String);

        parse_args!(execute_data, key_arg);

        let mut key = String::new();
        arg_to_string(key_arg, "key", &mut key);

        let this = ZendClassObject::<Self>::get(execute_data).unwrap();

        let mutex_client = match this.client.as_mut() {
            Some(client) => client,
            None => {
                unsafe {
                    exceptions::throw(crate::RS_EX.as_ref().unwrap(), "Can't get client");
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
                exceptions::throw(crate::RS_EX.as_ref().unwrap(), e.to_string().as_str());
            }

            return;
        }

        Self::values_to_zval(result.unwrap(), return_value);
    }

    fn values_to_zval(value: Values, zval: &mut Zval) {
        match value {
            redis_client::redis::Values::Integers(int) => {
                zval.set_long(int);
            }
            redis_client::redis::Values::Errors(error) => {
                unsafe {
                    exceptions::throw(crate::RS_EX.as_ref().unwrap(), error.as_str());
                }

                return;
            }
            redis_client::redis::Values::BulkString(string) => {
                zval.set_string(string);
            }
            redis_client::redis::Values::SimpleString(string) => {
                zval.set_string(string);
            }
            redis_client::redis::Values::Arrays(_) => {
                unsafe {
                    exceptions::throw(crate::RS_EX.as_ref().unwrap(), "Array values unsupported");
                }

                return;
            }
        }
    }

    pub fn get_build_for_php() -> *mut ClassEntry {
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

        ClassBuilder::new("Redis\\Client")
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
        Self { client: None }
    }
}
