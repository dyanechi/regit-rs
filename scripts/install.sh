#!/usr/bin/env bash

APP_NAME="regit"
APP_VER="v0.1.0"
APP_DESC="A tool for fetching git directories"
APP_FILE="./target/release/$APP_NAME"

INSTALL_DIR="/usr/local/bin"
APP_LOCATION="$INSTALL_DIR/$APP_NAME"

sudo echo ""
echo "Building cargo package in release mode..."
bash -c "cargo build --release"

if [ ! -d "$INSTALL_DIR" ]; then
    echo "Creationg installation directory..."
    mkdir -p $INSTALL_DIR
fi

echo "Copying file from $APP_FILE to $APP_LOCATION..."
sudo cp $APP_FILE $APP_LOCATION

echo "Retrieving script location..."
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
# CURRENT_PATH=${CURRENT_PATH:-/}        # to correct for the case where PWD=/
echo "Script location is: $SCRIPT_DIR"

echo "Creating desktop entry..."
cat >> ~/Desktop/$APP_NAME.desktop << EOF 
[Desktop Entry]
Version=$APP_VER
Name=$APP_NAME
Comment=$APP_DESC
Exec=.$SCRIPT_DIR/$APP_NAME
Icon=utilities-terminal
Terminal=false
Type=Application
Categories=Application;
EOF

echo "Success!"