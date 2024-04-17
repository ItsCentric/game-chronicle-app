package main

import (
	"context"
	"log"
	"os"
	"os/user"
	"path"
	"path/filepath"
	"strings"

	"github.com/joho/godotenv"
	"github.com/wailsapp/wails/v2/pkg/runtime"
	"gorm.io/gorm"
)

var twitchClientId string
var twitchClientSecret string

// App struct
type App struct {
	ctx context.Context
	Db  *gorm.DB
}

// NewApp creates a new App application struct
func NewApp() *App {
	return &App{}
}

// startup is called at application startup
func (a *App) startup(ctx context.Context) {
	// Perform your setup here
	var err error
	a.Db, err = initializeDatabase()
	if err != nil {
		log.Fatal("Error initializing database:", err)
	}
	a.ctx = ctx
	err = godotenv.Load()
	if err != nil {
		log.Println("Couldn't get .env file, trying compiled values")
		if twitchClientId == "" || twitchClientSecret == "" {
			log.Fatal("Missing compiled secrets")
		}
	}
	var preferences UserSettings
	res := a.Db.FirstOrCreate(&preferences)
	if res.Error != nil {
		log.Fatal("Error getting user preferences:", res.Error.Error())
	}
	processMonitor := NewProcessMonitor()
	if !preferences.ProcessMonitoringEnabled {
		return
	}
	var pathsToMonitor string
	executablePaths := strings.Split(preferences.ExecutablePaths, ";")
	for _, executablePath := range executablePaths {
		fileInfo, err := os.Stat(executablePath)
		if err != nil {
			log.Fatal("Error getting file info for executable path:", err)
		}
		if fileInfo.IsDir() {
			err := filepath.Walk(executablePath, func(walkedPath string, info os.FileInfo, err error) error {
				if err != nil {
					return err
				}

				if info.Mode().Perm()&0111 != 0 && !info.IsDir() {
					pathsToMonitor += walkedPath + ";"
					return nil
				} else if isWindows && strings.Contains(path.Base(walkedPath), ".exe") {
					pathsToMonitor += walkedPath + ";"
					return nil
				} else {
					return nil
				}
			})
			if err != nil {
				log.Fatal("Error walking path:", err)
			}
		} else {
			pathsToMonitor += executablePath + ";"
		}
	}
	go processMonitor.MonitorProcesses(pathsToMonitor, ctx, a.Db)
}

type OpenDirectoryDialogResponse struct {
	SelectedDirectory string `json:"selectedDirectory"`
	Error             string `json:"error"`
}

type GetCurrentUsernameResponse struct {
	Username string `json:"username"`
	Error    string `json:"error"`
}

func (a *App) OpenDirectoryDialog() OpenDirectoryDialogResponse {
	selectedDirectory, err := runtime.OpenDirectoryDialog(a.ctx, runtime.OpenDialogOptions{})
	if err != nil {
		return OpenDirectoryDialogResponse{SelectedDirectory: "", Error: err.Error()}
	}
	return OpenDirectoryDialogResponse{SelectedDirectory: selectedDirectory}
}

func (a *App) GetCurrentUsername() GetCurrentUsernameResponse {
	currentUser, err := user.Current()
	if err != nil {
		return GetCurrentUsernameResponse{Username: "", Error: err.Error()}
	}
	return GetCurrentUsernameResponse{Username: currentUser.Username, Error: ""}
}
