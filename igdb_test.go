package main

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"os"
	"testing"
)

func TestFormatIdString(t *testing.T) {
	ids := []int{1, 2, 3}
	expected := "1,2,3"
	result, err := formatIdString(ids)
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}
	if result != expected {
		t.Errorf("Expected %s, got %s", expected, result)
	}

	ids = []int{}
	_, err = formatIdString(ids)
	if err == nil {
		t.Errorf("Expected error for empty slice")
	}
}

func TestSendIgdbRequest(t *testing.T) {
	t.Skip("Skipping network request")
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Write([]byte(`{"access_token": "test_token"}`))
	}))
	defer server.Close()

	accessToken, err := sendIgdbRequest("endpoint", "token", "body")
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}

	var response AccessTokenResponse
	err = json.Unmarshal(accessToken, &response)
	if err != nil {
		t.Errorf("Error unmarshaling response: %v", err)
	}
	if response.AccessToken != "test_token" {
		t.Errorf("Expected access token 'test_token', got %s", response.AccessToken)
	}
}

func TestAuthenticateWithTwitch(t *testing.T) {
	t.Skip("Skipping network request")
	app := App{}
	originalClientId := twitchClientId
	originalClientSecret := twitchClientSecret
	defer func() {
		twitchClientId = originalClientId
		twitchClientSecret = originalClientSecret
	}()

	os.Setenv("TWITCH_CLIENT_ID", "test_client_id")
	os.Setenv("TWITCH_CLIENT_SECRET", "test_client_secret")

	twitchClientId = ""
	twitchClientSecret = ""
	response := app.AuthenticateWithTwitch()
	if response.Error != "" {
		t.Errorf("Expected no error for missing twitchClientId, got %s", response.Error)
	}

	twitchClientId = "test_client_id"
	twitchClientSecret = "test_client_secret"
	response = app.AuthenticateWithTwitch()
	if response.Error != "" || response.AccessToken == "" {
		t.Errorf("Unexpected response: %+v", response)
	}
}

func TestSearchForGame(t *testing.T) {
	t.Skip("Skipping network request")
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Write([]byte(`[{"id": 1, "name": "Test Game", "cover": {"id": 1, "image_id": "test_image_id"}}]`))
	}))
	defer server.Close()

	app := App{}
	response := app.SearchForGame("Test Game", "token")
	if response.Error != "" || len(response.Games) != 1 {
		t.Errorf("Unexpected response: %v", response)
	}
}

func TestGetRandomGames(t *testing.T) {
	t.Skip("Skipping network request")
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Write([]byte(`[{"id": 1, "name": "Test Game", "cover": {"id": 1, "image_id": "test_image_id"}}]`))
	}))
	defer server.Close()

	app := App{}
	response := app.GetRandomGames(1, "token")
	if response.Error != "" || len(response.Games) != 1 {
		t.Errorf("Unexpected response: %v", response)
	}
}

func TestGetGamesById(t *testing.T) {
	t.Skip("Skipping network request")
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Write([]byte(`[{"id": 1, "name": "Test Game", "cover": {"id": 1, "image_id": "test_image_id"}}]`))
	}))
	defer server.Close()

	app := App{}
	response := app.GetGamesById([]int{1}, "token")
	if response.Error != "" || len(response.Games) != 1 {
		t.Errorf("Unexpected response: %v", response)
	}
}

func TestGetSimilarGames(t *testing.T) {
	t.Skip("Skipping network request")
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Write([]byte(`[{"id": 1, "similar_games": [{"id": 2, "name": "Similar Game", "cover": {"id": 2, "image_id": "test_image_id"}}]}]`))
	}))
	defer server.Close()

	app := App{}
	response := app.GetSimilarGames([]int{1}, "token")
	if response.Error != "" || len(response.Games) != 1 {
		t.Errorf("Unexpected response: %v", response)
	}
}
