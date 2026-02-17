#!/bin/bash
set -e

## Install useful cargo dev tools
cargo install cargo-expand
cargo install cargo-edit

## Install oh-my-zsh (non-interactive)
sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)" "" --unattended

## Configure oh-my-zsh theme and plugins
sed -i -e 's/ZSH_THEME=.*/ZSH_THEME="awesomepanda"/' ~/.zshrc
sed -i -e 's/^plugins=(\(.*\))/plugins=(\1 git rust)/' ~/.zshrc
