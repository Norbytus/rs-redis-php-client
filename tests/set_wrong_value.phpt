--TEST--
Set value null value
--EXTENSIONS--
redis_client
--FILE--
<?php
include('config.inc');
$redis = new Redis\Client($redisConnect);

try {
    $redis->set(12, null);
} catch (TypeError $e) {
    echo $e->getMessage();
}
?>
--EXPECT--
Wrong value argument
