export RUST_LOG=info

# check if can run no node
#./target/release/node-template --dev

# run one node
./node-template --chain=local --alice -d /tmp/alice 
