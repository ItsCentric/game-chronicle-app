package main

import (
	"errors"

	"gorm.io/gorm"
)

type Log struct {
	gorm.Model
	Title      string    `gorm:"not null" json:"title"`
	Rating     uint      `gorm:"check:rating <= 10" gorm:"not null" json:"rating"`
	Notes      string    `json:"notes"`
	Status     LogStatus `gorm:"not null" json:"status"`
	StatusID   string    `gorm:"not null" json:"statusId"`
	Finished   uint      `gorm:"check:finished <= 1" gorm:"not null" json:"finished"`
	TimePlayed uint      `gorm:"not null" json:"timePlayed"`
}

type LogData struct {
	Title      string     `json:"title"`
	Rating     uint       `json:"rating"`
	Notes      string     `json:"notes"`
	StatusID   string     `json:"status"`
	Finished   uint       `json:"finished"`
	TimePlayed TimePlayed `json:"timePlayed"`
}

type TimePlayed struct {
	Hours   uint `json:"hours"`
	Minutes uint `json:"minutes"`
}

type LogStatus struct {
	Status string `gorm:"primaryKey"`
	Seq    uint
}

func newLog(data LogData) (*Log, error) {
	logValidationError := validateCandidateLog(data)
	if logValidationError != nil {
		return nil, logValidationError
	}
	timePlayed := data.TimePlayed.Hours*60 + data.TimePlayed.Minutes

	return &Log{Title: data.Title, Rating: data.Rating, Notes: data.Notes, StatusID: data.StatusID, Finished: data.Finished, TimePlayed: timePlayed}, nil
}

func validateCandidateLog(data LogData) error {
	if data.Title == "" {
		return errors.New("Title cannot be empty")
	}
	if data.Rating > 10 || data.Rating < 0 {
		return errors.New("Rating must be between 0 and 10")
	}
	if data.Finished > 1 || data.Finished < 0 {
		return errors.New("Could not parse finished value")
	}
	if data.TimePlayed.Hours < 0 || data.TimePlayed.Minutes < 0 {
		return errors.New("Time played cannot be negative")
	}
	if data.TimePlayed.Minutes > 59 {
		return errors.New("Minutes cannot be greater than 59")
	}
	if data.StatusID == "" {
		return errors.New("Status cannot be empty")
	}
	return nil
}
