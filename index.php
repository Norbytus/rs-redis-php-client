<?php

declare(strict_types=1);

$t = new Redis\Client("127.0.0.1:6379");
$t->set("test", "test");
$t->cmd('ping')->execute();
$t->cmd('ping')->execute();
$t->set("test", "test");

exit(0);
