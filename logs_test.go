package main

import (
	"testing"
	"time"
)

func TestNewLog(t *testing.T) {
	data := LogData{
		Title:      "Test Game",
		Date:       time.Now().Add(-24 * time.Hour),
		Rating:     8,
		Notes:      nil,
		StatusID:   "Completed",
		Finished:   true,
		TimePlayed: TimePlayed{Hours: 2, Minutes: 30},
		GameId:     1,
	}

	t.Run("Valid data", func(t *testing.T) {
		log, err := newLog(data)
		if err != nil {
			t.Errorf("Unexpected error: %v", err)
		}
		if log == nil {
			t.Errorf("Expected log, got nil")
		}
		if log.Title != data.Title || log.Rating != data.Rating || log.StatusID != data.StatusID || log.Finished != data.Finished || log.GameId != data.GameId || log.TimePlayedMinutes != 150 {
			t.Errorf("Unexpected log values")
		}
	})

	t.Run("Invalid title", func(t *testing.T) {
		data.Title = ""
		_, err := newLog(data)
		if err == nil || err["title"] != "Title cannot be empty" {
			t.Errorf("Expected validation error for title")
		}
		data.Title = "Test Game"
	})

	t.Run("Invalid date", func(t *testing.T) {
		data.Date = time.Now().Add(24 * time.Hour)
		_, err := newLog(data)
		if err == nil || err["date"] != "Date cannot be in the future" {
			t.Errorf("Expected validation error for date")
		}
		data.Date = time.Now().Add(-24 * time.Hour)
	})

	t.Run("Invalid rating", func(t *testing.T) {
		data.Rating = 11
		_, err := newLog(data)
		if err == nil || err["rating"] != "Rating must be between 0 and 10" {
			t.Errorf("Expected validation error for rating")
		}
		data.Rating = 8
	})

	t.Run("Invalid timePlayed", func(t *testing.T) {
		data.TimePlayed = TimePlayed{Hours: 2, Minutes: 60}
		_, err := newLog(data)
		if err == nil || err["timePlayed"] != "Minutes cannot be greater than 59" {
			t.Errorf("Expected validation error for timePlayed")
		}
		data.TimePlayed = TimePlayed{Hours: 2, Minutes: 30}
	})

	t.Run("Invalid status", func(t *testing.T) {
		data.StatusID = ""
		_, err := newLog(data)
		if err == nil || err["status"] != "Status cannot be empty" {
			t.Errorf("Expected validation error for status")
		}
		data.StatusID = "Completed"
	})

	t.Run("Non-nil notes", func(t *testing.T) {
		notes := "Test notes"
		data.Notes = &notes
		_, err := newLog(data)
		if err != nil {
			t.Errorf("Unexpected error: %v", err)
		}
	})
}

func TestValidateCandidateLog(t *testing.T) {
	data := LogData{
		Title:      "Test Game",
		Date:       time.Now().Add(-24 * time.Hour),
		Rating:     8,
		Notes:      nil,
		StatusID:   "Completed",
		Finished:   true,
		TimePlayed: TimePlayed{Hours: 2, Minutes: 30},
		GameId:     1,
	}

	t.Run("Valid data", func(t *testing.T) {
		err := validateCandidateLog(data)
		if len(err) > 0 {
			t.Errorf("Unexpected validation errors: %v", err)
		}
	})

	t.Run("Invalid title", func(t *testing.T) {
		data.Title = ""
		err := validateCandidateLog(data)
		if len(err) != 1 || err["title"] != "Title cannot be empty" {
			t.Errorf("Expected validation error for title")
		}
		data.Title = "Test Game"
	})

	t.Run("Invalid date", func(t *testing.T) {
		data.Date = time.Now().Add(24 * time.Hour)
		err := validateCandidateLog(data)
		if len(err) != 1 || err["date"] != "Date cannot be in the future" {
			t.Errorf("Expected validation error for date")
		}
		data.Date = time.Now().Add(-24 * time.Hour)
	})

	t.Run("Invalid rating", func(t *testing.T) {
		data.Rating = 11
		err := validateCandidateLog(data)
		if len(err) != 1 || err["rating"] != "Rating must be between 0 and 10" {
			t.Errorf("Expected validation error for rating")
		}
		data.Rating = 8
	})

	t.Run("Invalid timePlayed", func(t *testing.T) {
		data.TimePlayed = TimePlayed{Hours: 2, Minutes: 60}
		err := validateCandidateLog(data)
		if len(err) != 1 || err["timePlayed"] != "Minutes cannot be greater than 59" {
			t.Errorf("Expected validation error for timePlayed")
		}
		data.TimePlayed = TimePlayed{Hours: 2, Minutes: 30}
	})

	t.Run("Invalid status", func(t *testing.T) {
		data.StatusID = ""
		err := validateCandidateLog(data)
		if len(err) != 1 || err["status"] != "Status cannot be empty" {
			t.Errorf("Expected validation error for status")
		}
		data.StatusID = "Completed"
	})
}
