cargo build --release
sudo rm /usr/local/bin/gluck
sudo mv ./target/release/gluck /usr/local/bin/
sudo cp ./gluck.desktop ~/.local/share/applications/