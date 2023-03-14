#!/usr/bin/env bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
APP_NAME="regit"
APP_VER="v0.1.0"
APP_DESC="A tool for fetching git directories"
APP_FILE="$SCRIPT_DIR/$APP_NAME"

INSTALL_DIR="/usr/local/bin"
APP_LOCATION="$INSTALL_DIR/$APP_NAME"

sudo echo ""

if [ ! -d "$INSTALL_DIR" ]; then
    echo "Creationg installation directory..."
    mkdir -p $INSTALL_DIR
fi

echo "Copying file from $APP_FILE to $APP_LOCATION..."
sudo cp $APP_FILE $APP_LOCATION
echo "Installation completed!"