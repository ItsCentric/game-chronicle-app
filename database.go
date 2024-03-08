package main

import (
	"errors"
	"log"

	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
	"gorm.io/gorm/clause"
)

var logStatuses = []string{"Completed", "Playing", "Backlog", "Wishlist", "Abandoned"}

type Database struct {
	client *gorm.DB
}

type UserSettings struct {
	gorm.Model
	ExecutablePaths          string `json:"executablePaths"`
	ProcessMonitoringEnabled bool   `json:"processMonitoringEnabled" gorm:"default:false"`
}

func InitializeDatabase() (Database, error) {
	var err error
	database := Database{}
	database.client, err = gorm.Open(sqlite.Open("logs.db"), &gorm.Config{})
	if err != nil {
		return database, errors.New("failed to connect database")
	}
	database.client.AutoMigrate(&Log{})
	database.client.AutoMigrate(&UserSettings{})
	for i, status := range logStatuses {
		database.client.Clauses(clause.OnConflict{DoNothing: true}).Create(&LogStatus{Status: status, Order: uint(i * 10)})
	}

	return database, nil
}

type InsertGameLogResponse struct {
	Log    Log               `json:"log"`
	Errors map[string]string `json:"errors"`
}

type GetUserSettingsResponse struct {
	Preferences UserSettings `json:"preferences"`
	Error       error        `json:"error"`
}

type UserSettingsData struct {
	ExecutablePaths          string `json:"executablePaths"`
	ProcessMonitoringEnabled bool   `json:"processMonitoringEnabled"`
}

func (d *Database) InsertGameLog(data LogData) InsertGameLogResponse {
	response := InsertGameLogResponse{}
	gameLog, validationErrors := newLog(data)
	if len(validationErrors) > 0 {
		response.Errors = validationErrors
		return response
	}
	database.client.Create(&gameLog)
	response.Log = *gameLog

	return response
}

func (d *Database) GetGameLogs(sortBy string, sortOrder string, filter []string) []*Log {
	var logs []*Log
	database.client.Where("status_id IN ?", filter).Order(sortBy + " " + sortOrder).Find(&logs)
	return logs
}

func (d *Database) GetUserSettings() GetUserSettingsResponse {
	var preferences UserSettings
	res := database.client.First(&preferences)
	return GetUserSettingsResponse{Preferences: preferences, Error: res.Error}
}

func (d *Database) SaveUserSettings(newSettings UserSettingsData) {
	var preferences UserSettings
	res := database.client.FirstOrCreate(&preferences)
	if res.Error != nil {
		log.Fatal("Error getting or creating user preferences:", res.Error.Error())
	}
	preferences.ExecutablePaths = newSettings.ExecutablePaths
	preferences.ProcessMonitoringEnabled = newSettings.ProcessMonitoringEnabled
	res = database.client.Save(&preferences)
	if res.Error != nil {
		log.Fatal("Error updating user preferences:", res.Error.Error())
	}
}
