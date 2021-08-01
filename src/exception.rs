use ext_php_rs::php::enums::DataType;
use ext_php_rs::{
    parse_args,
    php::{
        args::Arg,
        class::{ClassBuilder, ClassEntry},
        execution_data::ExecutionData,
        flags::{MethodFlags, PropertyFlags},
        function::FunctionBuilder,
        types::zval::Zval,
    },
    ZendObjectHandler,
};

#[derive(ZendObjectHandler)]
pub struct RedisClientException;

impl RedisClientException {
    pub fn build() -> *mut ClassEntry {
        let constructor = FunctionBuilder::constructor(RedisClientException::constructor)
            .not_required()
            .arg(Arg::new("message", DataType::String))
            .arg(Arg::new("code", DataType::Long))
            .arg(Arg::new("line", DataType::String))
            .arg(Arg::new("file", DataType::String))
            .build();

        ClassBuilder::new("Redis\\Exception\\RedisClientException")
            .extends(ClassEntry::exception())
            .method(constructor, MethodFlags::Public)
            .property("message", "", PropertyFlags::Protected)
            .property("code", 0, PropertyFlags::Protected)
            .property("line", "", PropertyFlags::Protected)
            .property("file", "", PropertyFlags::Protected)
            .object_override::<Self>()
            .build()
    }

    pub extern "C" fn constructor(execute_data: &mut ExecutionData, _: &mut Zval) {
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
