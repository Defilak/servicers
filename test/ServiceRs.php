<?php

class ServiceRs extends ArrayObject 
{
    const PATH = "target/debug/servicers.json";

    public function __construct()
    {
        if(!file_exists(static::PATH)) {
            throw new Exception("no config");
        }

        foreach(json_decode(file_get_contents(static::PATH)) as $service) {
            $this[] = new ServiceConfig($service);
        }
    }
}
