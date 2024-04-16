package main

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"log"
	"math/rand"
	"net/http"
	"os"
)

type AccessTokenResponse struct {
	AccessToken string `json:"access_token"`
	Error       string `json:"error"`
}

type GetRandomGamesResponse struct {
	Games []IgdbGame `json:"games"`
	Error string     `json:"error"`
}

type SearchForGameResponse struct {
	Games []IgdbGame `json:"games"`
	Error string     `json:"error"`
}

type GetGamesByIdResponse struct {
	Games []IgdbGame `json:"games"`
	Error string     `json:"error"`
}

type GetSimilarGamesResponse struct {
	Games []IgdbGame `json:"games"`
	Error string     `json:"error"`
}

type SimplifiedIgdbCover struct {
	Id       int     `json:"id"`
	Image_id *string `json:"image_id"`
}

type IgdbGame struct {
	Id    int                 `json:"id"`
	Name  string              `json:"name"`
	Cover SimplifiedIgdbCover `json:"cover"`
}

type SimilarGamesResponse struct {
	Id           int        `json:"id"`
	SimilarGames []IgdbGame `json:"similar_games"`
}

func formatIdString(ids []int) (string, error) {
	if len(ids) == 0 {
		return "", errors.New("No elements in slice")
	}
	formattedIds := ""
	for i, v := range ids {
		formattedIds += fmt.Sprintf("%v", v)
		if i < len(ids)-1 {
			formattedIds += ","
		}
	}
	return formattedIds, nil
}

func sendIgdbRequest(endpoint string, accessToken string, body string) ([]byte, error) {
	request, err := http.NewRequest(http.MethodPost, fmt.Sprintf("https://api.igdb.com/v4/%s", endpoint), bytes.NewBuffer([]byte(body)))
	if err != nil {
		return []byte{}, err
	}
	request.Header.Set("Client-ID", twitchClientId)
	request.Header.Set("Authorization", fmt.Sprintf("Bearer %s", accessToken))
	client := &http.Client{}
	response, err := client.Do(request)
	if err != nil {
		return []byte{}, err
	}
	defer response.Body.Close()
	responseBody, err := io.ReadAll(response.Body)
	if err != nil {
		return []byte{}, err
	}

	return responseBody, nil
}

func (a *App) AuthenticateWithTwitch() AccessTokenResponse {
	if twitchClientId == "" {
		log.Println("Missing compiled secret, attempting to load from environment")
		twitchClientId = os.Getenv("TWITCH_CLIENT_ID")
		if twitchClientId == "" {
			return AccessTokenResponse{Error: "twitchClientId environment variable not set"}
		}
	}
	if twitchClientSecret == "" {
		log.Println("Missing compiled secret, attempting to load from environment")
		twitchClientSecret = os.Getenv("TWITCH_CLIENT_SECRET")
		if twitchClientSecret == "" {
			return AccessTokenResponse{Error: "twitchClientSecret environment variable not set"}
		}
	}
	requestUrl := fmt.Sprintf("https://id.twitch.tv/oauth2/token?client_id=%s&client_secret=%s&grant_type=client_credentials", twitchClientId, twitchClientSecret)
	accessTokenResponse, err := http.Post(requestUrl, "application/json", nil)
	if err != nil {
		return AccessTokenResponse{Error: err.Error()}
	}
	defer accessTokenResponse.Body.Close()
	accessTokenResponseBody, err := io.ReadAll(accessTokenResponse.Body)
	if err != nil {
		return AccessTokenResponse{Error: err.Error()}
	}
	var accessToken AccessTokenResponse
	err = json.Unmarshal(accessTokenResponseBody, &accessToken)
	if err != nil {
		return AccessTokenResponse{Error: err.Error()}
	}

	return accessToken
}

func (a *App) SearchForGame(title string, accessToken string) SearchForGameResponse {
	responseBody, err := sendIgdbRequest("games", accessToken, fmt.Sprintf("search \"%s\"; fields name, cover.image_id; where category = 0 & version_parent = null;", title))
	if err != nil {
		return SearchForGameResponse{Error: err.Error()}
	}
	var igdbGames []IgdbGame
	err = json.Unmarshal(responseBody, &igdbGames)
	if err != nil {
		return SearchForGameResponse{Error: err.Error()}
	}

	return SearchForGameResponse{Games: igdbGames}
}

func (a *App) GetRandomGames(amount int, accessToken string) GetRandomGamesResponse {
	randomOffset := rand.Intn(900)
	responseBody, err := sendIgdbRequest("games", accessToken, fmt.Sprintf("fields name, cover.image_id; limit %v; where category = 0 & total_rating >= 85 & platforms.category = (1, 6); offset %v;", amount, randomOffset))
	if err != nil {
		return GetRandomGamesResponse{Error: err.Error()}
	}
	var igdbGames []IgdbGame
	err = json.Unmarshal(responseBody, &igdbGames)
	if err != nil {
		return GetRandomGamesResponse{Error: err.Error()}
	}
	return GetRandomGamesResponse{Games: igdbGames}
}

func (a *App) GetGamesById(ids []int, accessToken string) GetGamesByIdResponse {
	if len(ids) == 0 {
		return GetGamesByIdResponse{Error: "No IDs provided"}
	}
	idsStr, err := formatIdString(ids)
	if err != nil {
		return GetGamesByIdResponse{Error: err.Error()}
	}
	responseBody, err := sendIgdbRequest("games", accessToken, fmt.Sprintf("fields name, cover.image_id; where id = (%v); limit %v;", idsStr, len(ids)+1))
	if err != nil {
		return GetGamesByIdResponse{Error: err.Error()}
	}
	var igdbGames []IgdbGame
	err = json.Unmarshal(responseBody, &igdbGames)
	if err != nil {
		return GetGamesByIdResponse{Error: err.Error()}
	}
	if len(igdbGames) == 0 || igdbGames[0].Id == 0 {
		return GetGamesByIdResponse{Error: "No games found with those IDs"}
	}
	return GetGamesByIdResponse{Games: igdbGames}
}

func (a *App) GetSimilarGames(ids []int, accessToken string) GetSimilarGamesResponse {
	idsStr, err := formatIdString(ids)
	if err != nil {
		return GetSimilarGamesResponse{Error: err.Error()}
	}
	responseBody, err := sendIgdbRequest("games", accessToken, fmt.Sprintf("fields similar_games.name, similar_games.cover.image_id; where category = 0 & platforms.category = (1, 6) & id = (%v); exclude id;", idsStr))
	if err != nil {
		return GetSimilarGamesResponse{Error: err.Error()}
	}
	var response []SimilarGamesResponse
	var igdbGames []IgdbGame
	err = json.Unmarshal(responseBody, &response)
	for _, v := range response {
		igdbGames = append(igdbGames, v.SimilarGames...)
	}
	if err != nil {
		return GetSimilarGamesResponse{Error: err.Error()}
	}
	return GetSimilarGamesResponse{Games: igdbGames}
}
