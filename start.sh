export RUST_LOG=Trace
nohup ./target/release/node-template --dev 2>&1 >logdev & tail -f logdev

