sudo apt update
# May prompt for location information
sudo apt install -y git clang curl libssl-dev llvm libudev-dev

# RUST
#curl https://getsubstrate.io -sSf | bash -s -- --fast


# WASM
rustup default stable
rustup update
rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly

# CHECK set-up
rustup show

