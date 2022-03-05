echo "Start alice"
nohup ../../target/release/node-template --chain=local \
	--name alice \
	--validator \
	-d /tmp/alice \
	--no-grandpa \
	--ws-port 9944\
	2>&1 >log_alice 
