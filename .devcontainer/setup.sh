## Install rustup and common components
curl https://sh.rustup.rs -sSf | sh -s -- -y 
rustup install stable
rustup component add rustfmt
rustup component add rustfmt --toolchain stable
rustup component add clippy 
rustup component add clippy --toolchain stable

cargo install cargo-expand
cargo install cargo-edit

## setup and install oh-my-zsh
sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"
ZSH=$HOME/.oh-my-zsh
ZSH_THEME="awesomepanda"
sed -i -e "s/ZSH_THEME=.*/ZSH_THEME=\"awesomepanda\"/g" $ZSH/ohmyzsh.sh
plugins=(git zsh-interactive-cd rust)
sed -i -e "s/plugins=\(.*\)/plugins=(\1 git zsh-interactive-cd rust)/g" $ZSH/ohmyzsh.sh
cp -R /root/.oh-my-zsh /home/$USERNAME
cp /root/.zshrc /home/$USERNAME
sed -i -e "s/\/root\/.oh-my-zsh/\/home\/$USERNAME\/.oh-my-zsh/g" /home/$USERNAME/.zshrc
chown -R $USER_UID:$USER_GID /home/$USERNAME/.oh-my-zsh /home/$USERNAME/.zshrc
