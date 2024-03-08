package main

import (
	"context"
	"log"
	"strings"
	"time"

	"github.com/shirou/gopsutil/v3/process"
	"github.com/wailsapp/wails/v2/pkg/runtime"
)

type Process struct {
	Pid        int32  `json:"pid"`
	Name       string `json:"name"`
	Path       string `json:"path"`
	CreateTime int64  `json:"createTime"`
}
type ProcessMap map[string]Process
type ProcessMonitor struct {
	previousRunningProcesses ProcessMap
}

func NewProcess(pid int32, name string, path string, createTime int64) *Process {
	return &Process{Pid: pid, Name: name, Path: path, CreateTime: createTime}
}

func NewProcessMonitor() *ProcessMonitor {
	return &ProcessMonitor{previousRunningProcesses: make(ProcessMap)}
}

func (pm *ProcessMonitor) GetRunningProcesses() (ProcessMap, error) {
	runningProcesses := make(ProcessMap)
	processes, err := process.Processes()
	if err != nil {
		return nil, err
	}

	for _, runningProcess := range processes {
		processPath, err := runningProcess.Exe()
		if err != nil {
			return nil, err
		}
		processName, err := runningProcess.Name()
		if err != nil {
			return nil, err
		}
		processCreateTime, err := runningProcess.CreateTime()
		if err != nil {
			return nil, err
		}
		runningProcesses[processPath] = *NewProcess(runningProcess.Pid, processName, processPath, processCreateTime)
	}
	return runningProcesses, nil
}

func (pm *ProcessMonitor) FilterProcesses(processMapToFilter ProcessMap, pathsString string) (ProcessMap, error) {
	filteredProcesses := make(ProcessMap)
	for processPath, _process := range processMapToFilter {
		if strings.Contains(processPath, pathsString) {
			filteredProcesses[processPath] = _process
		}
	}
	return filteredProcesses, nil
}

func (pm *ProcessMonitor) MonitorProcesses(pathsToMonitorString string, context context.Context) {
	ticker := time.NewTicker(1 * time.Second)
	defer ticker.Stop()
	for range ticker.C {
		runningProcesses, err := pm.GetRunningProcesses()
		if err != nil {
			log.Fatal("Error getting running processes:", err.Error())
		}
		if len(pathsToMonitorString) > 0 {
			runningProcesses, err = pm.FilterProcesses(runningProcesses, pathsToMonitorString)
			if err != nil {
				log.Fatal("Error filtering running processes:", err.Error())
			}
		}

		for runningProcessPath, runningProcess := range runningProcesses {
			if _, isStillRunning := pm.previousRunningProcesses[runningProcessPath]; !isStillRunning {
				log.Println("PROCESS", runningProcess.Pid, "(", runningProcess.Name, ") STARTED AT", time.UnixMilli(runningProcess.CreateTime))
			}
		}

		for previousProcessPath, previousProcess := range pm.previousRunningProcesses {
			if _, isStillRunning := runningProcesses[previousProcessPath]; !isStillRunning {
				log.Println("PROCESS", previousProcess.Pid, "(", previousProcess.Name, ") STOPPED, LASTED FOR", time.Now().UnixMilli()-previousProcess.CreateTime, "MILLISECONDS")
				runtime.Show(context)
			}
		}

		pm.previousRunningProcesses = runningProcesses
	}
}
