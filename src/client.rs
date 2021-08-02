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
use crate::common::arg_to_string;
use std::cell::Cell;

type ConnectionWrap = Option<Cell<redis_client::redis::Client>>;

#[derive(ZendObjectHandler)]
pub struct Client {
    client: ConnectionWrap,
    args: Vec<String>
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

        this.client = Some(Cell::new(client));
        this.args = Vec::new();
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

        let client = if let Some(client) = this.client.as_mut() {
            client
        } else {
            unsafe {
                exceptions::throw(crate::RS_EX.as_ref().unwrap(), "Can't get client");
            }

            return;
        };

        let result = redis_client::redis::Cmd::cmd("SET")
            .arg(&key)
            .arg(&value)
            .execute(client.get_mut());

        if let Err(e) = result {
            unsafe {
                exceptions::throw(crate::RS_EX.as_ref().unwrap(), e.to_string().as_str());
            }

            return;
        }
    }

    pub extern "C" fn cmd(execute_data: &mut ExecutionData, return_value: &mut Zval) {
        let mut cmd_arg = Arg::new("cmd", DataType::String);

        parse_args!(execute_data, cmd_arg);

        let mut cmd = String::new();
        arg_to_string(cmd_arg, "cmd", &mut cmd);

        let this = ZendClassObject::<Self>::get(execute_data).unwrap();
        this.args.push(cmd);

        return_value.set_object(execute_data.get_self().unwrap(), false);
    }

    pub extern "C" fn execute(execute_data: &mut ExecutionData, _: &mut Zval) {
        let this = ZendClassObject::<Self>::get(execute_data).unwrap();

        let args: Vec<String> = this.args.clone();
        this.args.clear();

        if args.is_empty() {
            unsafe {
                exceptions::throw(crate::RS_EX.as_ref().unwrap(), "Empty argument list");
            }
        }

        let client = if let Some(client) = this.client.as_mut() {
            client
        } else {
            unsafe {
                exceptions::throw(crate::RS_EX.as_ref().unwrap(), "Can't get client");
            }

            return;
        };

        let mut cmd_option: Option<redis_client::redis::Cmd> = None;
        for (n, arg) in args.iter().enumerate() {
            if n == 0 {
                cmd_option = Some(redis_client::redis::Cmd::cmd(&arg));
                continue;
            }

            if let Some(cmd) = cmd_option {
                cmd_option = Some(cmd.arg(arg));
            }
        } 

        let _ = cmd_option.unwrap().execute(&mut client.get_mut());
    }

    pub extern "C" fn get_value(execute_data: &mut ExecutionData, return_value: &mut Zval) {
        let mut key_arg = Arg::new("key", DataType::String);

        parse_args!(execute_data, key_arg);

        let mut key = String::new();
        arg_to_string(key_arg, "key", &mut key);

        let this = ZendClassObject::<Self>::get(execute_data).unwrap();

        let client = if let Some(client) = this.client.as_mut() {
            client
        } else {
            unsafe {
                exceptions::throw(crate::RS_EX.as_ref().unwrap(), "Can't get client");
            }

            return;
        };

        let result = redis_client::redis::Cmd::cmd("GET")
            .arg(&key)
            .execute(client.get_mut());

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

        let cmd = FunctionBuilder::new("cmd", Client::cmd)
            .arg(Arg::new("cmd", DataType::String))
            .returns(DataType::True, true, false)
            .build();

        let execute = FunctionBuilder::new("execute", Client::execute)
            .build();

        ClassBuilder::new("Redis\\Client")
            .method(constructor, MethodFlags::Public)
            .method(set, MethodFlags::Public)
            .method(get, MethodFlags::Public)
            .method(cmd, MethodFlags::Public)
            .method(execute, MethodFlags::Public)
            .property("addr", "", PropertyFlags::Private)
            .object_override::<Self>()
            .build()
    }
}

impl Default for Client {
    fn default() -> Self {
        Self { client: None, args: Vec::new() }
    }
}
