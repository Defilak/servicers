<?php

$file = __DIR__.'/app1.txt';

$fs = fopen($file, 'w');

$i = 0;
while(true) {
    fputs($fs, "a$i");
    $i += 1;
    sleep(3);

    if(filesize($file) > 1024) {
        fclose($fs);
        unlink($file);
        $fs = fopen($file, 'w');
    }
}

