<?php

class ServiceConfig
{
    public $program = "";
    public $args = [];
    public $cwd = "";
    public $state = "Disabled";

    public function __construct(array|object $arr = [])
    {
        foreach ($arr as $key => $value) {
            $this->$key = $value;
        }
    }

    /**
     * Get the value of program
     */
    public function getProgram()
    {
        return $this->program;
    }

    /**
     * Set the value of program
     *
     * @return  self
     */
    public function setProgram($program)
    {
        $this->program = $program;

        return $this;
    }

    /**
     * Get the value of args
     */
    public function getArgs()
    {
        return $this->args;
    }

    /**
     * Set the value of args
     *
     * @return  self
     */
    public function setArgs($args)
    {
        $this->args = $args;

        return $this;
    }

    /**
     * Get the value of cwd
     */
    public function getCwd()
    {
        return $this->cwd;
    }

    /**
     * Set the value of cwd
     *
     * @return  self
     */
    public function setCwd($cwd)
    {
        $this->cwd = $cwd;

        return $this;
    }

    /**
     * Get the value of state
     */
    public function getState()
    {
        return $this->state;
    }

    /**
     * Set the value of state
     *
     * @return  self
     */
    public function setState($state)
    {
        $this->state = $state;
        return $this;
    }
}
