package main

import (
	"errors"
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
	Username                        string `json:"username"`
	ExecutablePaths                 string `json:"executablePaths"`
	ProcessMonitoringEnabled        bool   `json:"processMonitoringEnabled" gorm:"default:false"`
	ProcessMonitoringDirectoryDepth int    `json:"processMonitoringDirectoryDepth" gorm:"default:3"`
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
	Username                        string `json:"username"`
	ExecutablePaths                 string `json:"executablePaths"`
	ProcessMonitoringEnabled        bool   `json:"processMonitoringEnabled"`
	ProcessMonitoringDirectoryDepth int    `json:"processMonitoringDirectoryDepth"`
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

type GetLogByIdResponse struct {
	Log   Log    `json:"log"`
	Error string `json:"error"`
}

type DashboardStatistics struct {
	CompletedGames int64 `json:"completedGames"`
	TimePlayed     int64 `json:"timePlayed"`
	TotalGames     int64 `json:"totalGames"`
}

type SaveUserSettingsResponse struct {
	NewSettings UserSettings `json:"newSettings"`
	Error       string       `json:"error"`
}

func initializeDatabase() (*gorm.DB, error) {
	var err error
	database, err := gorm.Open(sqlite.Open("logs.db"), &gorm.Config{})
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

func (a *App) SaveUserSettings(newSettings UserSettingsData) SaveUserSettingsResponse {
	var preferences UserSettings
	res := a.Db.FirstOrCreate(&preferences)
	if res.Error != nil {
		return SaveUserSettingsResponse{Error: res.Error.Error()}
	}
	preferences.ExecutablePaths = newSettings.ExecutablePaths
	preferences.ProcessMonitoringEnabled = newSettings.ProcessMonitoringEnabled
	preferences.ProcessMonitoringDirectoryDepth = newSettings.ProcessMonitoringDirectoryDepth
	preferences.Username = newSettings.Username
	res = a.Db.Save(&preferences)
	if res.Error != nil {
		return SaveUserSettingsResponse{Error: res.Error.Error()}
	}
	return SaveUserSettingsResponse{NewSettings: preferences}
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

func (a *App) GetLogById(id int) GetLogByIdResponse {
	var dbLog Log
	res := a.Db.First(&dbLog, id)
	if res.Error != nil {
		return GetLogByIdResponse{Error: res.Error.Error()}
	}
	return GetLogByIdResponse{Log: dbLog}
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
	if res.Error != nil {
		return GetUserSettingsResponse{Error: res.Error.Error()}
	}
	return GetUserSettingsResponse{Preferences: preferences}
}

func (a *App) InsertExecutableDetails(newExecutableDetails ExecutableDetails) InsertExecutableDetailsResponse {
	res := a.Db.Create(&newExecutableDetails)
	if res.Error != nil {
		return InsertExecutableDetailsResponse{Error: res.Error.Error()}
	}
	return InsertExecutableDetailsResponse{Details: newExecutableDetails}
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

	res := a.Db.Model(&Log{}).Where("status_id = ? AND date BETWEEN ? AND ?", "Completed", endOfLastMonth, beginningOfNextMonth).Count(&completedGames)
	if res.Error != nil {
		return GetDashboardStatisticsResponse{Error: res.Error.Error()}
	}
	res = a.Db.Model(&Log{}).Select("COALESCE(SUM(time_played_minutes), 0)").Where("date BETWEEN ? AND ?", endOfLastMonth, beginningOfNextMonth).Scan(&timePlayed)
	if res.Error != nil {
		return GetDashboardStatisticsResponse{Error: res.Error.Error()}
	}
	res = a.Db.Model(&Log{}).Where("date BETWEEN ? AND ? AND status_id != ?", endOfLastMonth, beginningOfNextMonth, "Wishlist").Count(&totalGames)
	if res.Error != nil {
		return GetDashboardStatisticsResponse{Error: res.Error.Error()}
	}
	statistics.ThisMonthStatistics = DashboardStatistics{CompletedGames: completedGames, TimePlayed: timePlayed, TotalGames: totalGames}
	res = a.Db.Model(&Log{}).Where("status_id = ? AND date BETWEEN ? AND ?", "Completed", endOfMonthBeforeLast, beginningOfThisMonth).Count(&completedGames)
	if res.Error != nil {
		return GetDashboardStatisticsResponse{Error: res.Error.Error()}
	}
	res = a.Db.Model(&Log{}).Select("COALESCE(SUM(time_played_minutes), 0)").Where("date BETWEEN ? AND ? AND status_id != ?", endOfMonthBeforeLast, beginningOfThisMonth, "Wishlist").Scan(&timePlayed)
	if res.Error != nil {
		return GetDashboardStatisticsResponse{Error: res.Error.Error()}
	}
	res = a.Db.Model(&Log{}).Where("date BETWEEN ? AND ?", endOfMonthBeforeLast, beginningOfThisMonth).Count(&totalGames)
	if res.Error != nil {
		return GetDashboardStatisticsResponse{Error: res.Error.Error()}
	}
	statistics.LastMonthStatistics = DashboardStatistics{CompletedGames: completedGames, TimePlayed: timePlayed, TotalGames: totalGames}

	return statistics
}

func (a *App) GetRecentLogs(amount int, filter []string) GetRecentLogsResponse {
	var logs []Log
	if len(filter) == 0 {
		filter = logStatuses
	}
	res := a.Db.Order("date desc").Limit(amount).Where("status_id IN ?", filter).Find(&logs)
	if res.Error != nil {
		return GetRecentLogsResponse{Error: res.Error.Error()}
	}
	return GetRecentLogsResponse{Logs: logs}
}

func (a *App) UpdateLog(logId int, newData LogData) string {
	var logToUpdate Log
	res := a.Db.First(&logToUpdate, logId)
	if res.Error != nil {
		return res.Error.Error()
	}
	logToUpdate.Title = newData.Title
	logToUpdate.Date = newData.Date
	logToUpdate.Rating = newData.Rating
	logToUpdate.StatusID = newData.StatusID
	logToUpdate.Finished = newData.Finished
	logToUpdate.TimePlayedMinutes = newData.TimePlayed.Minutes + newData.TimePlayed.Hours*60
	if newData.Notes != nil {
		logToUpdate.Notes = *newData.Notes
	}
	res = a.Db.Save(&logToUpdate)
	if res.Error != nil {
		return res.Error.Error()
	}
	return ""
}

func (a *App) DeleteLog(logId int) string {
	res := a.Db.Delete(&Log{}, logId)
	if res.Error != nil {
		return res.Error.Error()
	}
	return ""
}
