--TEST--
Throw exception if could'n create connection
--EXTENSIONS--
redis_client
--FILE--
<?php
try {
    $redis = new Redis\Client("127.0.0.1:16379");
} catch (Redis\Exception\RedisClientException $e) {
    echo $e->getMessage();
}
?>
--EXPECT--
Connection error
