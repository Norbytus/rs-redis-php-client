--TEST--
Get value
--EXTENSIONS--
redis_client
--FILE--
<?php
include('config.inc');
$redis = new Redis\Client($redisConnect);

$redis->set(12, 'hello');

echo $redis->get(12);
?>
--EXPECT--
hello
