#!/usr/bin/env bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
PROJECT_DIR="$SCRIPT_DIR/.."

APP_NAME="regit"
APP_VER="v0.1.0"
APP_DESC="A tool for fetching git directories"
APP_FILE="$PROJECT_DIR/target/release/$APP_NAME"

RELEASE_NAME="$APP_NAME-$APP_VER"
RELEASE_DIR="$PROJECT_DIR/release/$RELEASE_NAME"

sudo echo ""
echo "Building cargo package in release mode..."
bash -c "cargo build --release"

if [ ! -d "$RELEASE_DIR" ]; then
    echo "Creating new release app directory..."
    mkdir -p $RELEASE_DIR
fi

cp "$SCRIPT_DIR/install_release.sh" "$RELEASE_DIR/install.sh"
cp $APP_FILE "$RELEASE_DIR/$APP_NAME"

cd "release/"
tar czf "$RELEASE_NAME.tar.gz" $RELEASE_NAME
# mv "$RELEASE_NAME.tar.gz" $RELEASE_DIR
rm -rf $RELEASE_NAME
cd ..

echo "Done!"