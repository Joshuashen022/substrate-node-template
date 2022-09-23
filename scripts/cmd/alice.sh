#export RUST_LOG=debug

../../target/release/node-template build-spec > genesis.json

sed -i 's/127.0.0.1/192.168.1.4/g' genesis.json

nohup ../../target/release/node-template --chain=genesis.json \
	--alice \
	--no-grandpa \
	-d /tmp/alice \
	--ws-port 9944\
	2>&1 >log_alice &  tail -f log_alice
