package main

import (
	"os/user"
	"testing"

	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
)

func TestGetCurrentUsername(t *testing.T) {
	db, _ := gorm.Open(sqlite.Open(":memory:"), &gorm.Config{})
	app := App{Db: db}
	db.AutoMigrate(&UserSettings{})
	settings := UserSettings{}
	app.Db.Create(&settings)

	t.Run("No username set", func(t *testing.T) {
		response := app.GetCurrentUsername()
		user, _ := user.Current()
		if response.Error != "" || user.Username != response.Username {
			t.Errorf("Expected username of %s, got %s", user.Username, response.Username)
			if response.Error != "" {
				t.Errorf("Error: %s", response.Error)
			}
		}
	})

	t.Run("Username set", func(t *testing.T) {
		settings.Username = "test"

		app.Db.Save(&settings)
		response := app.GetCurrentUsername()
		if response.Error != "" || response.Username != "test" {
			t.Errorf("Expected username of test, got %s", response.Username)
			if response.Error != "" {
				t.Errorf("Error: %s", response.Error)
			}
		}
	})
}
