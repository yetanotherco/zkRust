#!/bin/bash

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

echo "Installing zkRust..."

BASE_DIR=$HOME
ZKRUST_DIR="${ZKRUST_DIR-"$BASE_DIR/.zkRust"}"
ZKRUST_BIN_DIR="$ZKRUST_DIR/bin"
ZKRUST_BIN_PATH="$ZKRUST_BIN_DIR/zkRust"
CURRENT_TAG=$(curl -s -L \
  -H "Accept: application/vnd.github+json" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/repos/yetanotherco/zkRust/releases/latest \
  | grep '"tag_name":' | awk -F'"' '{print $4}')
RELEASE_URL="https://github.com/yetanotherco/zkRust/releases/download/$CURRENT_TAG/"

ARCH=$(uname -m)

if [ "$ARCH" == "x86_64" ]; then
    FILE="zkRust-x86"
elif [ "$ARCH" == "arm64" ]; then
    FILE="zkRust-arm64"
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
echo "Installing zkVM toolchains"

# Install risc0 toolchain
curl -L https://risczero.com/install | bash
rzup install
cargo risczero --version
echo "Risc0 Toolchain Installed"

# Install sp1 toolchain
curl -L https://sp1.succinct.xyz | bash
source $PROFILE
sp1up
cargo prove --version
echo "Sp1 Toolchain Installed"

# Clone the specific directory structure from the Git repository
ZKRUST_GIT_REPO_URL="https://github.com/yetanotherco/zkRust.git"

echo "Cloning repository..."
git clone "$ZKRUST_GIT_REPO_URL" "$ZKRUST_DIR"

# Copy the directory structure from the cloned repository to the .zkRust folder
echo "Copying directory structure..."
cp -r "$ZKRUST_DIR/zkRust/workspaces/" "$ZKRUST_DIR/"

# Clean up the cloned repository
echo "Cleaning up..."
rm -rf "$ZKRUST_DIR/zkRust"

echo "Run 'source $PROFILE' or start a new terminal session to use aligned."
