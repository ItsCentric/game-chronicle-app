package main

import (
	"reflect"
	"testing"
	"time"

	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
)

func TestInitializeDatabase(t *testing.T) {
	db, err := initializeDatabase()
	if err != nil {
		t.Fatalf("initializeDatabase returned an error: %v", err)
	}

	if !db.Migrator().HasTable(&Log{}) || !db.Migrator().HasTable(&UserSettings{}) || !db.Migrator().HasTable(&ExecutableDetails{}) {
		t.Errorf("Tables not created")
	}

	var statuses []LogStatus
	db.Find(&statuses)
	expectedStatuses := []LogStatus{
		{Status: "Completed", Order: 0},
		{Status: "Playing", Order: 10},
		{Status: "Backlog", Order: 20},
		{Status: "Wishlist", Order: 30},
		{Status: "Abandoned", Order: 40},
	}

	if !reflect.DeepEqual(statuses, expectedStatuses) {
		t.Errorf("Expected statuses %v, got %v", expectedStatuses, statuses)
	}
}

func TestGetExecutableDetails(t *testing.T) {
	db, _ := gorm.Open(sqlite.Open(":memory:"), &gorm.Config{})
	db.AutoMigrate(&ExecutableDetails{})

	details := ExecutableDetails{
		ExecutableName: "Test.exe",
		GameId:         1,
		MinutesPlayed:  60,
	}
	db.Create(&details)

	response, err := getExecutableDetails("Test.exe", db)

	if err != nil || response.ExecutableName != details.ExecutableName {
		t.Errorf("Error getting executable details")
		t.Logf("Expected executable name: %s", details.ExecutableName)
		t.Logf("Got executable name: %s", response.ExecutableName)
		if err != nil {
			t.Logf("Error: %s", err.Error())
		}
	}
}

func TestSaveUserSettings(t *testing.T) {
	db, _ := gorm.Open(sqlite.Open(":memory:"), &gorm.Config{})
	db.AutoMigrate(&UserSettings{})
	app := App{Db: db}

	settings := UserSettingsData{
		Username:                        "testUser",
		ExecutablePaths:                 "/path/to/executable",
		ProcessMonitoringEnabled:        true,
		ProcessMonitoringDirectoryDepth: 2,
	}
	app.SaveUserSettings(settings)

	var savedSettings UserSettings
	db.First(&savedSettings)
	if savedSettings.Username != settings.Username || savedSettings.ExecutablePaths != settings.ExecutablePaths || savedSettings.ProcessMonitoringEnabled != settings.ProcessMonitoringEnabled || savedSettings.ProcessMonitoringDirectoryDepth != settings.ProcessMonitoringDirectoryDepth {
		t.Errorf("Saved settings do not match expected settings")
		t.Logf("Expected username: %s, executable paths: %s, process monitoring enabled: %t, process monitoring directory depth: %d", settings.Username, settings.ExecutablePaths, settings.ProcessMonitoringEnabled, settings.ProcessMonitoringDirectoryDepth)
		t.Logf("Got username: %s, executable paths: %s, process monitoring enabled: %t, process monitoring directory depth: %d", savedSettings.Username, savedSettings.ExecutablePaths, savedSettings.ProcessMonitoringEnabled, savedSettings.ProcessMonitoringDirectoryDepth)
	}
}

func TestInsertGameLog(t *testing.T) {
	db, _ := gorm.Open(sqlite.Open(":memory:"), &gorm.Config{})
	db.AutoMigrate(&Log{}, &LogStatus{})
	app := App{Db: db}

	logData := LogData{
		Title:      "Test Game",
		Date:       time.Now(),
		Rating:     8,
		StatusID:   "Completed",
		Finished:   true,
		TimePlayed: TimePlayed{Hours: 10, Minutes: 30},
		GameId:     1,
	}
	response := app.InsertGameLog(logData)

	if response.Errors != nil || response.Log.Title != logData.Title {
		t.Errorf("Error inserting game log")
		t.Logf("Expected log with title 'Test Game'")
		t.Logf("Got log with title %s", response.Log.Title)
		if response.Errors != nil {
			t.Logf("Errors: %v", response.Errors)
		}
	}
}

func TestGetUserSettings(t *testing.T) {
	db, _ := gorm.Open(sqlite.Open(":memory:"), &gorm.Config{})
	db.AutoMigrate(&UserSettings{})
	app := App{Db: db}

	settings := UserSettings{
		Username:                        "testUser",
		ExecutablePaths:                 "/path/to/executable",
		ProcessMonitoringEnabled:        true,
		ProcessMonitoringDirectoryDepth: 2,
	}
	db.Create(&settings)

	response := app.GetUserSettings()

	if response.Error != "" || response.Preferences.Username != settings.Username {
		t.Errorf("Error getting user settings")
		t.Logf("Expected username: %s", settings.Username)
		t.Logf("Got username: %s", response.Preferences.Username)
		if response.Error != "" {
			t.Logf("Error: %s", response.Error)
		}
	}
}

func TestGetLogById(t *testing.T) {
	db, _ := gorm.Open(sqlite.Open(":memory:"), &gorm.Config{})
	db.AutoMigrate(&Log{}, &LogStatus{})
	app := App{Db: db}

	logData := LogData{
		Title:      "Test Game",
		Date:       time.Now(),
		Rating:     8,
		StatusID:   "Completed",
		Finished:   true,
		TimePlayed: TimePlayed{Hours: 10, Minutes: 30},
		GameId:     1,
	}
	app.InsertGameLog(logData)

	response := app.GetLogById(1)

	if response.Error != "" || response.Log.Title != logData.Title {
		t.Errorf("Error getting log by ID")
		t.Logf("Expected log with title 'Test Game'")
		t.Logf("Got log with title %s", response.Log.Title)
		if response.Error != "" {
			t.Logf("Error: %s", response.Error)
		}
	}
}

func TestGetGameLogs(t *testing.T) {
	db, _ := gorm.Open(sqlite.Open(":memory:"), &gorm.Config{})
	db.AutoMigrate(&Log{}, &LogStatus{})
	app := App{Db: db}

	logData1 := LogData{Title: "Game1", StatusID: "Completed"}
	logData2 := LogData{Title: "Game2", StatusID: "Playing"}
	app.InsertGameLog(logData1)
	app.InsertGameLog(logData2)

	t.Run("Get all logs", func(t *testing.T) {
		logs := app.GetGameLogs("", "", nil)

		if len(logs) != 2 || logs[0].Title != logData1.Title || logs[1].Title != logData2.Title {
			t.Errorf("Error getting game logs")
			t.Logf("Expected 2 logs with titles 'Game1' and 'Game2'")
			t.Logf("Got %d logs with titles %s and %s", len(logs), logs[0].Title, logs[1].Title)
		}
	})

	t.Run("Get sorted logs", func(t *testing.T) {
		logsSorted := app.GetGameLogs("title", "asc", nil)

		if logsSorted[0].Title != "Game1" || logsSorted[1].Title != "Game2" {
			t.Errorf("Error sorting game logs")
			t.Logf("Expected first log with title 'Game1'")
			t.Logf("Got first log with title %s", logsSorted[0].Title)
		}
	})

	t.Run("Get filtered logs", func(t *testing.T) {
		logsFiltered := app.GetGameLogs("", "", []string{"Playing"})

		if len(logsFiltered) != 1 || logsFiltered[0].Title != "Game2" {
			t.Errorf("Error filtering game logs")
			t.Logf("Expected 1 log with title 'Game2'")
			t.Logf("Got %d logs with title %s", len(logsFiltered), logsFiltered[0].Title)
		}
	})
}

func TestInsertExecutableDetails(t *testing.T) {
	db, _ := gorm.Open(sqlite.Open(":memory:"), &gorm.Config{})
	db.AutoMigrate(&ExecutableDetails{})
	app := App{Db: db}

	details := ExecutableDetails{
		ExecutableName: "Test.exe",
		GameId:         1,
		MinutesPlayed:  60,
	}
	response := app.InsertExecutableDetails(details)

	if response.Error != "" || response.Details.ExecutableName != details.ExecutableName {
		t.Errorf("Error inserting executable details")
		t.Logf("Expected executable name: %s", details.ExecutableName)
		t.Logf("Got executable name: %s", response.Details.ExecutableName)
		if response.Error != "" {
			t.Logf("Error: %s", response.Error)
		}
	}
}

func TestGetDashboardStatistics(t *testing.T) {
	db, _ := gorm.Open(sqlite.Open(":memory:"), &gorm.Config{})
	db.AutoMigrate(&Log{}, &LogStatus{})
	app := App{Db: db}

	logData1 := LogData{Title: "Game1", Date: time.Now(), StatusID: "Completed", TimePlayed: TimePlayed{Hours: 2, Minutes: 30}}
	logData2 := LogData{Title: "Game2", Date: time.Now(), StatusID: "Completed", TimePlayed: TimePlayed{Hours: 3, Minutes: 0}}
	app.InsertGameLog(logData1)
	app.InsertGameLog(logData2)

	statistics := app.GetDashboardStatistics()

	if statistics.ThisMonthStatistics.CompletedGames != 2 || statistics.ThisMonthStatistics.TimePlayed != 330 {
		t.Errorf("Error getting dashboard statistics")
		t.Logf("Expected 2 games completed, 330 minutes played")
		t.Logf("Got %d games completed, %d minutes played", statistics.ThisMonthStatistics.CompletedGames, statistics.ThisMonthStatistics.TimePlayed)
	}
}

func TestGetRecentLogs(t *testing.T) {
	db, _ := gorm.Open(sqlite.Open(":memory:"), &gorm.Config{})
	db.AutoMigrate(&Log{}, &LogStatus{})
	app := App{Db: db}

	logData1 := LogData{Title: "Game1", StatusID: "Completed"}
	logData2 := LogData{Title: "Game2", StatusID: "Playing"}
	app.InsertGameLog(logData1)
	app.InsertGameLog(logData2)

	t.Run("Get all recent logs", func(t *testing.T) {
		recentLogs := app.GetRecentLogs(1, nil)

		if recentLogs.Error != "" || len(recentLogs.Logs) != 1 || recentLogs.Logs[0].Title != "Game1" {
			t.Errorf("Error getting recent logs")
			t.Logf("Expected a log with title 'Game1'")
			t.Logf("Got %d logs", len(recentLogs.Logs))
			if recentLogs.Error != "" {
				t.Logf("Error: %s", recentLogs.Error)
			}
		}
	})

	t.Run("Get filtered recent logs", func(t *testing.T) {
		recentLogs := app.GetRecentLogs(1, []string{"Playing"})
		if recentLogs.Error != "" || len(recentLogs.Logs) != 1 || recentLogs.Logs[0].Title != "Game2" {
			t.Errorf("Error getting recent logs")
			t.Logf("Expected a log with title 'Game2'")
			t.Logf("Got %d logs", len(recentLogs.Logs))
			if recentLogs.Error != "" {
				t.Logf("Error: %s", recentLogs.Error)
			}
		}
	})

	t.Run("Get more recent logs than exist", func(t *testing.T) {
		recentLogs := app.GetRecentLogs(3, nil)
		if recentLogs.Error != "" || len(recentLogs.Logs) != 2 {
			t.Errorf("Error getting recent logs")
			t.Logf("Expected 2 logs")
			t.Logf("Got %d logs", len(recentLogs.Logs))
			if recentLogs.Error != "" {
				t.Logf("Error: %s", recentLogs.Error)
			}
		}
	})
}

func TestUpdateLog(t *testing.T) {
	db, _ := gorm.Open(sqlite.Open(":memory:"), &gorm.Config{})
	db.AutoMigrate(&Log{}, &LogStatus{})
	app := App{Db: db}

	logData := LogData{
		Title:      "Test Game",
		Date:       time.Now(),
		Rating:     8,
		StatusID:   "Completed",
		Finished:   true,
		TimePlayed: TimePlayed{Hours: 10, Minutes: 30},
		GameId:     1,
	}
	app.InsertGameLog(logData)

	newData := LogData{
		Title:      "Updated Game",
		Rating:     9,
		StatusID:   "Playing",
		Finished:   false,
		TimePlayed: TimePlayed{Hours: 5, Minutes: 0},
	}
	response := app.UpdateLog(1, newData)

	if response != "" {
		t.Errorf("Error updating log")
		t.Logf("Error: %s", response)
	}

	var updatedLog Log
	db.First(&updatedLog, 1)
	timePlayedMinutes := newData.TimePlayed.Hours*60 + newData.TimePlayed.Minutes
	if updatedLog.Title != newData.Title || updatedLog.Rating != newData.Rating || updatedLog.StatusID != newData.StatusID || updatedLog.Finished != newData.Finished || updatedLog.TimePlayedMinutes != timePlayedMinutes {
		t.Errorf("Log not updated correctly")
		t.Logf("Expected title: %s, rating: %d, status: %s, finished: %t, time played: %d", newData.Title, newData.Rating, newData.StatusID, newData.Finished, timePlayedMinutes)
		t.Logf("Got title: %s, rating: %d, status: %s, finished: %t, time played: %d", updatedLog.Title, updatedLog.Rating, updatedLog.StatusID, updatedLog.Finished, updatedLog.TimePlayedMinutes)
	}
}

func TestDeleteLog(t *testing.T) {
	db, _ := gorm.Open(sqlite.Open(":memory:"), &gorm.Config{})
	db.AutoMigrate(&Log{}, &LogStatus{})
	app := App{Db: db}

	logData := LogData{
		Title:    "Test Game",
		Date:     time.Now(),
		Rating:   8,
		StatusID: "Completed",
		Finished: true,
		GameId:   1,
	}
	app.InsertGameLog(logData)

	response := app.DeleteLog(1)

	if response != "" {
		t.Errorf("Error deleting log")
		t.Logf("Error: %s", response)
	}

	var log Log
	result := db.First(&log, 1)
	if result.Error != gorm.ErrRecordNotFound {
		t.Errorf("Log was not deleted")
	}
}
