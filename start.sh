export RUST_LOG=trace
#export RUST_LOG=debug
#export RUST_LOG=info

#cargo build --release
# check if can run no node
#./target/release/node-template --dev

# run one node
#nohup ./target/release/node-template --chain=local --alice -d /tmp/alice 2>&1 > logdev & tail -f logdev


#nohup ./target/release/node-template --no-mdns --chain=local --alice -d /tmp/alice 2>&1 > logdev & tail -f logdev


nohup ./target/release/node-template --no-mdns --dev 2>&1 > logdev & tail -f logdev

