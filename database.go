package main

import (
	"errors"
	"log"
	"time"

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

type ExecutableDetails struct {
	ExecutableName string `gorm:"primaryKey" json:"executableName"`
	GameId         int    `json:"gameId"`
	MinutesPlayed  int    `json:"minutesPlayed"`
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
	database.client.AutoMigrate(&ExecutableDetails{})
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

type InsertExecutableDetailsResponse struct {
	Details ExecutableDetails `json:"details"`
	Error   error             `json:"error"`
}

type GetDashboardStatisticsResponse struct {
	ThisMonthStatistics DashboardStatistics `json:"thisMonthStatistics"`
	LastMonthStatistics DashboardStatistics `json:"lastMonthStatistics"`
	Error               string              `json:"error"`
}

type GetRecentLogsResponse struct {
	Logs  []Log  `json:"logs"`
	Error string `json:"error"`
}

type GetPopularUpcomingGamesResponse struct {
	Games []IgdbGame `json:"games"`
	Error string     `json:"error"`
}

type DashboardStatistics struct {
	CompletedGames int64 `json:"completedGames"`
	TimePlayed     int64 `json:"timePlayed"`
	TotalGames     int64 `json:"totalGames"`
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
	if len(sortBy) > 0 && len(sortOrder) > 0 {
		database.client.Order(sortBy + " " + sortOrder).Find(&logs)
		return logs
	} else if len(filter) > 0 {
		database.client.Where("status_id IN ?", filter).Find(&logs)
		return logs
	} else {
		database.client.Find(&logs)
		return logs
	}
}

func (d *Database) GetUserSettings() GetUserSettingsResponse {
	var preferences UserSettings
	res := database.client.First(&preferences)
	return GetUserSettingsResponse{Preferences: preferences, Error: res.Error}
}

func (d *Database) getExecutableDetails(queriedExecutable string) (ExecutableDetails, error) {
	var details ExecutableDetails
	res := database.client.First(&details).Where("executable_name = ?", queriedExecutable)
	return details, res.Error
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

func (d *Database) InsertExecutableDetails(newExecutableDetails ExecutableDetails) InsertExecutableDetailsResponse {
	res := database.client.Create(&newExecutableDetails)
	return InsertExecutableDetailsResponse{Details: newExecutableDetails, Error: res.Error}
}

func (d *Database) GetDashboardStatistics() GetDashboardStatisticsResponse {
	var completedGames, timePlayed, totalGames int64
	statistics := GetDashboardStatisticsResponse{}
	currentMonth := time.Now().Month()
	currentYear := time.Now().Year()
	beginningOfThisMonth := time.Date(currentYear, currentMonth, 1, 0, 0, 0, 0, time.UTC)
	endOfLastMonth := beginningOfThisMonth.AddDate(0, 0, -1)
	beginningOfNextMonth := time.Date(currentYear, currentMonth+1, 1, 0, 0, 0, 0, time.UTC)
	endOfMonthBeforeLast := endOfLastMonth.AddDate(0, -1, -2)

	res := database.client.Model(&Log{}).Where("status_id = ? AND created_at BETWEEN ? AND ?", "Completed", endOfLastMonth, beginningOfNextMonth).Count(&completedGames)
	if res.Error != nil {
		return GetDashboardStatisticsResponse{Error: res.Error.Error()}
	}
	res = database.client.Model(&Log{}).Select("COALESCE(SUM(time_played_minutes), 0)").Where("created_at BETWEEN ? AND ?", endOfLastMonth, beginningOfNextMonth).Scan(&timePlayed)
	if res.Error != nil {
		return GetDashboardStatisticsResponse{Error: res.Error.Error()}
	}
	res = database.client.Model(&Log{}).Where("created_at BETWEEN ? AND ? AND status_id != ?", endOfLastMonth, beginningOfNextMonth, "Wishlist").Count(&totalGames)
	if res.Error != nil {
		return GetDashboardStatisticsResponse{Error: res.Error.Error()}
	}
	statistics.ThisMonthStatistics = DashboardStatistics{CompletedGames: completedGames, TimePlayed: timePlayed, TotalGames: totalGames}
	res = database.client.Model(&Log{}).Where("status_id = ? AND created_at BETWEEN ? AND ?", "Completed", endOfMonthBeforeLast, beginningOfThisMonth).Count(&completedGames)
	if res.Error != nil {
		return GetDashboardStatisticsResponse{Error: res.Error.Error()}
	}
	res = database.client.Model(&Log{}).Select("COALESCE(SUM(time_played_minutes), 0)").Where("created_at BETWEEN ? AND ? AND status_id != ?", endOfMonthBeforeLast, beginningOfThisMonth, "Wishlist").Scan(&timePlayed)
	if res.Error != nil {
		return GetDashboardStatisticsResponse{Error: res.Error.Error()}
	}
	res = database.client.Model(&Log{}).Where("created_at BETWEEN ? AND ?", endOfMonthBeforeLast, beginningOfThisMonth).Count(&totalGames)
	if res.Error != nil {
		return GetDashboardStatisticsResponse{Error: res.Error.Error()}
	}
	statistics.LastMonthStatistics = DashboardStatistics{CompletedGames: completedGames, TimePlayed: timePlayed, TotalGames: totalGames}

	return statistics
}

func (d *Database) GetRecentLogs(amount int) GetRecentLogsResponse {
	var logs []Log
	res := database.client.Order("created_at desc").Limit(amount).Find(&logs)
	if res.Error != nil {
		return GetRecentLogsResponse{Error: res.Error.Error()}
	}
	return GetRecentLogsResponse{Logs: logs}
}
