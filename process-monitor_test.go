package main

import (
	"testing"
	"time"
)

func TestNewProcess(t *testing.T) {
	p := NewProcess(1234, "test", "/path/to/test", time.Now().Unix())
	if p.Pid != 1234 || p.Name != "test" || p.Path != "/path/to/test" {
		t.Errorf("Expected values not set in NewProcess")
	}
}

func TestNewProcessMonitor(t *testing.T) {
	pm := NewProcessMonitor()
	if len(pm.previousRunningProcesses) != 0 {
		t.Errorf("Expected an empty ProcessMap")
	}
}
