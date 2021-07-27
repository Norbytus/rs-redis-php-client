<?php

declare(strict_types=1);

$t = new Rust\Client("127.0.0.1:6379");
$t->setValue('test', 1);
print_r($t->getValue('test'));
