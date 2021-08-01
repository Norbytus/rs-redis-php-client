--TEST--
Success create connetion to redis
--EXTENSIONS--
redis_client
--FILE--
<?php
include('config.inc');
try {
    $redis = new Redis\Client($redisConnect);
} catch (Redis\Exception\RedisClientException $e) {
    echo $e->getMessage();
}
?>
--EXPECT--
