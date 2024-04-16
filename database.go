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

type Log struct {
	gorm.Model
	Title             string    `gorm:"not null" json:"title"`
	Date              time.Time `json:"date"`
	Rating            uint      `gorm:"check:rating <= 10" gorm:"not null" json:"rating"`
	Notes             string    `json:"notes"`
	Status            LogStatus `gorm:"not null" json:"status"`
	StatusID          string    `gorm:"not null" json:"statusId"`
	Finished          bool      `json:"finished"`
	TimePlayedMinutes uint      `gorm:"not null" json:"timePlayedMinutes"`
	GameId            int       `json:"gameId"`
}

type LogStatus struct {
	Status string `gorm:"primaryKey"`
	Order  uint
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

type InsertGameLogResponse struct {
	Log    Log               `json:"log"`
	Errors map[string]string `json:"errors"`
}

type GetUserSettingsResponse struct {
	Preferences UserSettings `json:"preferences"`
	Error       string       `json:"error"`
}

type UserSettingsData struct {
	ExecutablePaths          string `json:"executablePaths"`
	ProcessMonitoringEnabled bool   `json:"processMonitoringEnabled"`
}

type InsertExecutableDetailsResponse struct {
	Details ExecutableDetails `json:"details"`
	Error   string            `json:"error"`
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

func initializeDatabase() (*gorm.DB, error) {
	var err error
	database, err := gorm.Open(sqlite.Open("logs.db"), &gorm.Config{})
	log.Printf("\n\nInitialize Database: %+v, error: %s\n\n", database, err)
	if err != nil {
		return database, errors.New("failed to connect database")
	}
	database.AutoMigrate(&Log{})
	database.AutoMigrate(&UserSettings{})
	database.AutoMigrate(&ExecutableDetails{})
	for i, status := range logStatuses {
		database.Clauses(clause.OnConflict{DoNothing: true}).Create(&LogStatus{Status: status, Order: uint(i * 10)})
	}

	return database, nil
}

func getExecutableDetails(queriedExecutable string, database *gorm.DB) (ExecutableDetails, error) {
	var details ExecutableDetails
	res := database.First(&details).Where("executable_name = ?", queriedExecutable)
	return details, res.Error
}

func (a *App) SaveUserSettings(newSettings UserSettingsData) {
	var preferences UserSettings
	res := a.Db.FirstOrCreate(&preferences)
	if res.Error != nil {
		log.Fatal("Error getting or creating user preferences:", res.Error.Error())
	}
	preferences.ExecutablePaths = newSettings.ExecutablePaths
	preferences.ProcessMonitoringEnabled = newSettings.ProcessMonitoringEnabled
	res = a.Db.Save(&preferences)
	if res.Error != nil {
		log.Fatal("Error updating user preferences:", res.Error.Error())
	}
}

func (a *App) InsertGameLog(data LogData) InsertGameLogResponse {
	response := InsertGameLogResponse{}
	gameLog, validationErrors := newLog(data)
	if len(validationErrors) > 0 {
		response.Errors = validationErrors
		return response
	}
	a.Db.Create(&gameLog)
	response.Log = *gameLog

	return response
}

func (a *App) GetGameLogs(sortBy string, sortOrder string, filter []string) []*Log {
	var logs []*Log
	if len(sortBy) > 0 && len(sortOrder) > 0 {
		a.Db.Order(sortBy + " " + sortOrder).Find(&logs)
		return logs
	} else if len(filter) > 0 {
		a.Db.Where("status_id IN ?", filter).Find(&logs)
		return logs
	} else {
		a.Db.Find(&logs)
		return logs
	}
}

func (a *App) GetUserSettings() GetUserSettingsResponse {
	var preferences UserSettings
	res := a.Db.First(&preferences)
	return GetUserSettingsResponse{Preferences: preferences, Error: res.Error.Error()}
}

func (a *App) InsertExecutableDetails(newExecutableDetails ExecutableDetails) InsertExecutableDetailsResponse {
	res := a.Db.Create(&newExecutableDetails)
	return InsertExecutableDetailsResponse{Details: newExecutableDetails, Error: res.Error.Error()}
}

func (a *App) GetDashboardStatistics() GetDashboardStatisticsResponse {
	var completedGames, timePlayed, totalGames int64
	statistics := GetDashboardStatisticsResponse{}
	currentMonth := time.Now().Month()
	currentYear := time.Now().Year()
	beginningOfThisMonth := time.Date(currentYear, currentMonth, 1, 0, 0, 0, 0, time.UTC)
	endOfLastMonth := beginningOfThisMonth.AddDate(0, 0, -1)
	beginningOfNextMonth := time.Date(currentYear, currentMonth+1, 1, 0, 0, 0, 0, time.UTC)
	endOfMonthBeforeLast := endOfLastMonth.AddDate(0, -1, -2)

	res := a.Db.Model(&Log{}).Where("status_id = ? AND created_at BETWEEN ? AND ?", "Completed", endOfLastMonth, beginningOfNextMonth).Count(&completedGames)
	if res.Error != nil {
		return GetDashboardStatisticsResponse{Error: res.Error.Error()}
	}
	res = a.Db.Model(&Log{}).Select("COALESCE(SUM(time_played_minutes), 0)").Where("created_at BETWEEN ? AND ?", endOfLastMonth, beginningOfNextMonth).Scan(&timePlayed)
	if res.Error != nil {
		return GetDashboardStatisticsResponse{Error: res.Error.Error()}
	}
	res = a.Db.Model(&Log{}).Where("created_at BETWEEN ? AND ? AND status_id != ?", endOfLastMonth, beginningOfNextMonth, "Wishlist").Count(&totalGames)
	if res.Error != nil {
		return GetDashboardStatisticsResponse{Error: res.Error.Error()}
	}
	statistics.ThisMonthStatistics = DashboardStatistics{CompletedGames: completedGames, TimePlayed: timePlayed, TotalGames: totalGames}
	res = a.Db.Model(&Log{}).Where("status_id = ? AND created_at BETWEEN ? AND ?", "Completed", endOfMonthBeforeLast, beginningOfThisMonth).Count(&completedGames)
	if res.Error != nil {
		return GetDashboardStatisticsResponse{Error: res.Error.Error()}
	}
	res = a.Db.Model(&Log{}).Select("COALESCE(SUM(time_played_minutes), 0)").Where("created_at BETWEEN ? AND ? AND status_id != ?", endOfMonthBeforeLast, beginningOfThisMonth, "Wishlist").Scan(&timePlayed)
	if res.Error != nil {
		return GetDashboardStatisticsResponse{Error: res.Error.Error()}
	}
	res = a.Db.Model(&Log{}).Where("created_at BETWEEN ? AND ?", endOfMonthBeforeLast, beginningOfThisMonth).Count(&totalGames)
	if res.Error != nil {
		return GetDashboardStatisticsResponse{Error: res.Error.Error()}
	}
	statistics.LastMonthStatistics = DashboardStatistics{CompletedGames: completedGames, TimePlayed: timePlayed, TotalGames: totalGames}

	return statistics
}

func (a *App) GetRecentLogs(amount int) GetRecentLogsResponse {
	var logs []Log
	res := a.Db.Order("created_at desc").Limit(amount).Find(&logs)
	if res.Error != nil {
		return GetRecentLogsResponse{Error: res.Error.Error()}
	}
	return GetRecentLogsResponse{Logs: logs}
}
