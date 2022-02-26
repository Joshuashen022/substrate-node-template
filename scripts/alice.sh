nohup ../target/release/node-template --chain=local \
	--alice \
	--no-grandpa \
	-d /tmp/alice \
	--ws-port 9944\
	2>&1 >log_alice &
tail -f log_alice
