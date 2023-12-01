package main

import (
	"errors"

	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
	"gorm.io/gorm/clause"
)

var logStatuses = []string{"Completed", "Playing", "Backlog", "Wishlist", "Abandoned"}

type Database struct {
	client *gorm.DB
}

func InitializeDatabase() (Database, error) {
	var err error
	database := Database{}
	database.client, err = gorm.Open(sqlite.Open("logs.db"), &gorm.Config{})
	if err != nil {
		return database, errors.New("failed to connect database")
	}
	database.client.AutoMigrate(&Log{})
	for i, status := range logStatuses {
		database.client.Clauses(clause.OnConflict{DoNothing: true}).Create(&LogStatus{Status: status, Order: uint(i * 10)})
	}

	return database, nil
}

type InsertGameLogResponse struct {
	Log    Log               `json:"log"`
	Errors map[string]string `json:"errors"`
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

func (d *Database) GetAllGameLogs() []*Log {
	var logs []*Log
	database.client.Find(&logs)
	return logs
}
