package main

import (
	"time"

	"gorm.io/gorm"
)

type Log struct {
	gorm.Model
	Title             string    `gorm:"not null" json:"title"`
	Rating            uint      `gorm:"check:rating <= 10" gorm:"not null" json:"rating"`
	Notes             string    `json:"notes"`
	Status            LogStatus `gorm:"not null" json:"status"`
	StatusID          string    `gorm:"not null" json:"statusId"`
	StartedOn         time.Time `gorm:"not null" json:"startedOn"`
	FinishedOn        time.Time `gorm:"check:finished_on >= started_on" json:"finishedOn"`
	TimePlayedMinutes uint      `gorm:"not null" json:"timePlayedMinutes"`
}

type LogData struct {
	Title      string     `json:"title"`
	Rating     uint       `json:"rating"`
	Notes      string     `json:"notes"`
	StatusID   string     `json:"status"`
	StartedOn  time.Time  `json:"startedOn"`
	FinishedOn time.Time  `json:"finishedOn"`
	TimePlayed TimePlayed `json:"timePlayed"`
}

type TimePlayed struct {
	Hours   uint `json:"hours"`
	Minutes uint `json:"minutes"`
}

type LogStatus struct {
	Status string `gorm:"primaryKey"`
	Order  uint
}

func newLog(data LogData) (*Log, map[string]string) {
	logValidationError := validateCandidateLog(data)
	if logValidationError != nil {
		return nil, logValidationError
	}
	timePlayedMinutes := data.TimePlayed.Hours*60 + data.TimePlayed.Minutes

	return &Log{Title: data.Title, Rating: data.Rating, Notes: data.Notes, StatusID: data.StatusID, StartedOn: data.StartedOn, FinishedOn: data.FinishedOn, TimePlayedMinutes: timePlayedMinutes}, nil
}

func validateCandidateLog(data LogData) map[string]string {
	validationErrors := make(map[string]string)
	if data.Title == "" {
		validationErrors["title"] = "Title cannot be empty"
	}
	if data.Rating > 10 || data.Rating < 0 {
		validationErrors["rating"] = "Rating must be between 0 and 10"
	}
	if data.StartedOn.After(data.FinishedOn) {
		validationErrors["finishedOn"] = "Finished on cannot be before started on"
	}
	if data.FinishedOn.Before(data.StartedOn) {
		validationErrors["startedOn"] = "Started on cannot be after finished on"
	}
	if data.TimePlayed.Hours < 0 || data.TimePlayed.Minutes < 0 {
		validationErrors["timePlayed"] = "Time played cannot be negative"
	}
	if data.TimePlayed.Minutes > 59 {
		validationErrors["timePlayed"] = "Minutes cannot be greater than 59"
	}
	if data.StatusID == "" {
		validationErrors["status"] = "Status cannot be empty"
	}

	return validationErrors
}
