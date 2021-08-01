<?php

declare(strict_types=1);

$t = new Redis\Client("127.0.0.1:6379");
try {
    $t->set("", null);
} catch (TypeError $e) {
    print_r($e);
    echo $e->getMessage();
}

exit(0);
