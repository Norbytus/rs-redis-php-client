--TEST--
Get value
--EXTENSIONS--
redis_client
--FILE--
<?php
include('config.inc');
$redis = new Redis\Client($redisConnect);

try {
    echo $redis->get(null);
} catch (TypeError $e) {
    echo $e->getMessage();
}
?>
--EXPECT--
Wrong key argument
