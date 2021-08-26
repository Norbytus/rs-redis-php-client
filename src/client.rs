use ext_php_rs::php::exceptions::PhpException;
use ext_php_rs::php::types::zval::IntoZval;
use redis_client::redis::Values;

type ConnectionWrap = Option<redis_client::redis::Client>;

#[php_class(name = "Redis\\Client")]
pub struct RedisClient {
    client: ConnectionWrap,
    args: Vec<String>
}

impl Default for RedisClient {
    fn default() -> Self {
        Self { client: None, args: Vec::new() }
    }
}

impl IntoZval for RedisClient {
    fn set_zval(&self, zv: &mut ext_php_rs::php::types::zval::Zval, persistent: bool) -> ext_php_rs::errors::Result<()> {
        zv.set_object(self.as_zval(persistent).unwrap().object().unwrap(), false);

        Ok(())
    }
}

#[php_impl]
impl RedisClient {
    pub fn __constructor(&mut self, addr: String) -> Result<bool, PhpException<'static>> {
        match redis_client::redis::Client::new(addr) {
            Ok(client) => {
                self.client = Some(client);
                Ok(true)
            },
            Err(_) => {
                Err(PhpException::from_class::<RedisException>("Connection error".into()))
            }
        }
    }

    pub fn set_value(&mut self, key: String, value: String) -> Result<bool, PhpException<'static>> {
        let result = redis_client::redis::Cmd::cmd("SET")
            .arg(&key)
            .arg(&value)
            .execute(self.client.as_mut().unwrap());

        match result {
            Ok(_) => {
                Ok(true)
            },
            Err(e) => {
                Err(PhpException::from_class::<RedisException>(e.to_string().into()))
            }
        }
    }

    pub fn cmd(&mut self, cmd: String) -> Result<&mut Self, PhpException<'static>> {
        self.args.push(cmd);

        Ok(self)
    }

    pub fn execute(&mut self) -> Result<bool, PhpException<'static>> {
        let args: Vec<String> = self.args.clone();
        self.args.clear();

        if args.is_empty() {
            return Err(PhpException::from_class::<RedisException>("Empty argument list".into()));
        }

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

        let _ = cmd_option.unwrap().execute(self.client.as_mut().unwrap());

        Ok(true)
    }

    pub fn get_value(&mut self, key: String) -> Result<String, PhpException<'static>> {
        let result = redis_client::redis::Cmd::cmd("GET")
            .arg(&key)
            .execute(self.client.as_mut().unwrap());

        match result {
            Ok(data) => {
                match data {
                    Values::Errors(data)|Values::BulkString(data)|Values::SimpleString(data) => Ok(data),
                    Values::Arrays(_) => Err(PhpException::from_class::<RedisException>("Array type unsupported".into())),
                    Values::Integers(value) => Ok(value.to_string())
                }
            }
            Err(e) => {
                Err(PhpException::from_class::<RedisException>(e.to_string().into()))
            }
        }
    }
}
