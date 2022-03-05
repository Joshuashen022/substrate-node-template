
# Get id, e.g.: 12D3...kj
#cat logdev | grep -E ' Local node identity is' | awk  '{print $8}'
get_id=$(cat ../log_alice | grep -E ' Local node identity is' | awk  '{print $8}')
nohup ../../target/release/node-template --chain=local \
	--validator \
	-d /tmp/charlie \
	--port 30335 \
	--bootnodes '/ip4/127.0.0.1/tcp/30333/p2p/'"$get_id" \
	--ws-port 9946 \
	2>&1 > log_charlie 
