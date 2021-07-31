<?php

declare(strict_types=1);


try {
    $t = new Rust\Client("127.0.0.1:6379");
} catch (Rust\Exception\RedisClientException $e) {
    print_r($e->getMessage());

    exit(1);
}

exit(0);
