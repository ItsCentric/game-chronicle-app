package main

import (
	"context"
	"log"
	"os"
	"os/user"
	"path/filepath"
	"strings"

	"github.com/joho/godotenv"
	"github.com/wailsapp/wails/v2/pkg/runtime"
)

var database Database

// App struct
type App struct {
	ctx context.Context
}

// NewApp creates a new App application struct
func NewApp() *App {
	return &App{}
}

func init() {
	db, err := InitializeDatabase()
	if err != nil {
		log.Fatal("Error initializing database:", err)
	}
	database = db
}

// startup is called at application startup
func (a *App) startup(ctx context.Context) {
	// Perform your setup here
	a.ctx = ctx
	err := godotenv.Load()
	if err != nil {
		log.Fatal("Error loading .env file")
	}
	var preferences UserSettings
	res := database.client.FirstOrCreate(&preferences)
	if res.Error != nil {
		log.Fatal("Error getting user preferences:", res.Error.Error())
	}
}

// domReady is called after front-end resources have been loaded
func (a App) domReady(ctx context.Context) {
	processMonitor := NewProcessMonitor()
	preferencesResponse := database.GetUserSettings()
	if preferencesResponse.Error != nil {
		log.Fatal("Error getting user preferences:", preferencesResponse.Error)
	}
	preferences := preferencesResponse.Preferences
	runtime.EventsOn(ctx, "preferencesChanged", func(_ ...interface{}) {
		preferences = database.GetUserSettings().Preferences
	})

	if !preferences.ProcessMonitoringEnabled {
		return
	}
	var pathsToMonitor string
	executablePaths := strings.Split(preferences.ExecutablePaths, ";")
	for _, path := range executablePaths {
		fileInfo, err := os.Stat(path)
		if err != nil {
			log.Fatal("Error getting file info for executable path:", err)
		}
		if fileInfo.IsDir() {
			err := filepath.Walk(path, func(walkedPath string, info os.FileInfo, err error) error {
				if err != nil {
					return err
				}
				if info.Mode().Perm()&0111 != 0 && !info.IsDir() {
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
			pathsToMonitor += path + ";"
		}
	}
	go processMonitor.MonitorProcesses(pathsToMonitor, ctx)
}

// beforeClose is called when the application is about to quit,
// either by clicking the window close button or calling runtime.Quit.
// Returning true will cause the application to continue, false will continue shutdown as normal.
func (a *App) beforeClose(ctx context.Context) (prevent bool) {
	return false
}

// shutdown is called at application termination
func (a *App) shutdown(ctx context.Context) {
	// Perform your teardown here
}

type OpenDirectoryDialogResponse struct {
	SelectedDirectory string `json:"selectedDirectory"`
	Error             error  `json:"error"`
}

type GetCurrentUsernameResponse struct {
	Username string `json:"username"`
	Error    string `json:"error"`
}

func (a *App) OpenDirectoryDialog() OpenDirectoryDialogResponse {
	selectedDirectory, err := runtime.OpenDirectoryDialog(a.ctx, runtime.OpenDialogOptions{})
	return OpenDirectoryDialogResponse{SelectedDirectory: selectedDirectory, Error: err}
}

func (a *App) GetCurrentUsername() GetCurrentUsernameResponse {
	currentUser, err := user.Current()
	if err != nil {
		return GetCurrentUsernameResponse{Username: "", Error: err.Error()}
	}
	return GetCurrentUsernameResponse{Username: currentUser.Username, Error: ""}
}
