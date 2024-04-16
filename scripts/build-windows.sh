#! /bin/bash

echo -e "Start running the script..."

echo -e "Loading environment variables from .env file..."
set -a
source .env
set +a

if [ $TWITCH_CLIENT_ID = "" ] || [ $TWITCH_CLIENT_SECRET = "" ]; then
    echo -e "Twitch client id or secret is not set in the .env file!"
    exit 1
fi

if [ $1 = 'production' ]; then
    echo -e "Start building the production app for windows platform..."
    CC=x86_64-w64-mingw32-gcc GOOS=windows GOARCH=amd64 ~/go/bin/wails build --clean --platform windows/amd64 -ldflags "-X main.twitchClientId=$TWITCH_CLIENT_ID -X main.twitchClientSecret=$TWITCH_CLIENT_SECRET"
elif [ $1 = 'debug' ]; then
    echo -e "Start building the debug app for windows platform..."
    CC=x86_64-w64-mingw32-gcc GOOS=windows GOARCH=amd64 ~/go/bin/wails build --clean --debug --platform windows/amd64 -ldflags "-X main.twitchClientId=$TWITCH_CLIENT_ID -X main.twitchClientSecret=$TWITCH_CLIENT_SECRET"
else
    echo -e "Invalid argument! Please provide either 'production' or 'development' as an argument."
    exit 1
fi

echo -e "End running the script!"
