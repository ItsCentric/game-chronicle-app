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
		database.client.Clauses(clause.OnConflict{DoNothing: true}).Create(&LogStatus{Status: status, Seq: uint(i * 10)})
	}

	return database, nil
}

func (d *Database) InsertGameLog(data LogData) (Log, error) {
	gameLog, err := newLog(data)
	if err != nil {
		return Log{}, err
	}
	database.client.Create(&gameLog)

	return *gameLog, nil
}

func (d *Database) GetAllGameLogs() []*Log {
	var logs []*Log
	database.client.Find(&logs)
	return logs
}
