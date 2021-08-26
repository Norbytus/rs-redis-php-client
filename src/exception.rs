#[php_class(name = "Redis\\Exception\\RedisClientException")]
#[extends(ClassEntry::exception())]
pub struct RedisException {
    message: Option<String>,
    code: Option<i32>,
    line: Option<String>,
    file: Option<String>,
}

#[php_impl]
impl RedisException {
    pub fn __constructor(&mut self, message: Option<String>, code: Option<i32>, line: Option<String>, file: Option<String>) {
        self.message = message;
        self.code = code;
        self.line = line;
        self.file = file;
    }
}

impl Default for RedisException {
    fn default() -> Self {
        Self {
            ..Default::default()
        }
    }
}
