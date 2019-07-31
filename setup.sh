rm -rf /tmp/default/
sudo apt-get update -y
sudo apt-get install -y valgrind
sudo apt-get install kcachegrind

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update
rustup toolchain install nightly
