#!/bin/bash

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

echo "Installing zkRust..."

BASE_DIR=$HOME
ZKRUST_DIR="${ZKRUST_DIR-"$BASE_DIR/.zk_rust"}"
ZKRUST_BIN_DIR="$ZKRUST_DIR/bin"
ZKRUST_BIN_PATH="$ZKRUST_BIN_DIR/zk_rust"
CURRENT_TAG=$(curl -s -L \
  -H "Accept: application/vnd.github+json" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/repos/yetanotherco/zkRust/releases/latest \
  | grep '"tag_name":' | awk -F'"' '{print $4}')
RELEASE_URL="https://github.com/yetanotherco/zkRust/releases/download/$CURRENT_TAG/"

ARCH=$(uname -m)

if [ "$ARCH" == "x86_64" ]; then
    FILE="zk_rust"
elif [ "$ARCH" == "arm64" ]; then
    FILE="zk_rust"
else
    echo "Unsupported architecture: $ARCH"
    exit 1
fi

mkdir -p "$ZKRUST_BIN_DIR"
if curl -sSf -L "$RELEASE_URL$FILE" -o "$ZKRUST_BIN_PATH"; then
    echo "zkRust downloaded successful, installing..."
else
    echo "Error: Failed to download $RELEASE_URL$FILE"
    exit 1
fi
chmod +x "$ZKRUST_BIN_PATH"

# Store the correct profile file (i.e. .profile for bash or .zshenv for ZSH).
case $SHELL in
*/zsh)
    PROFILE="${ZDOTDIR-"$HOME"}/.zshenv"
    PREF_SHELL=zsh
    ;;
*/bash)
    PROFILE=$HOME/.bashrc
    PREF_SHELL=bash
    ;;
*/fish)
    PROFILE=$HOME/.config/fish/config.fish
    PREF_SHELL=fish
    ;;
*/ash)
    PROFILE=$HOME/.profile
    PREF_SHELL=ash
    ;;
*)
    echo "zkrust: could not detect shell, manually add ${ZKRUST_BIN_DIR} to your PATH."
    exit 1
esac

# Only add aligned if it isn't already in PATH.
if [[ ":$PATH:" != *":${ZKRUST_BIN_DIR}:"* ]]; then
    # Add the aligned directory to the path and ensure the old PATH variables remain.
    # If the shell is fish, echo fish_add_path instead of export.
    if [[ "$PREF_SHELL" == "fish" ]]; then
        echo >> "$PROFILE" && echo "fish_add_path -a $ZKRUST_BIN_DIR" >> "$PROFILE"
    else
        echo >> "$PROFILE" && echo "export PATH=\"\$PATH:$ZKRUST_BIN_DIR\"" >> "$PROFILE"
    fi
fi

echo "zkRust installed successfully in $ZKRUST_BIN_PATH."
echo "Detected your preferred shell is $PREF_SHELL and added aligned to PATH."
echo "Run 'source $PROFILE' or start a new terminal session to use aligned."