--TEST--
Set value null key
--EXTENSIONS--
redis_client
--FILE--
<?php
include('config.inc');
$redis = new Redis\Client($redisConnect);

try {
    $redis->set(null, 'test');
} catch (TypeError $e) {
    echo $e->getMessage();
}
?>
--EXPECT--
Wrong key argument
