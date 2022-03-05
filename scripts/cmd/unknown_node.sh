get_id=$(cat log_alice | grep -E ' Local node identity is' | awk  '{print $8}')

# export RUST_LOG=trace
#--node-key=0x407d6f8c48f527296bb865cb47b0459353c0bd244557a3511be9a1ce288aa4cc \

nohup ../../target/release/node-template --chain=local \
	--keystore-path=/tmp/unknown/chains/local_testnet/keystore \
	--ipc-path='march shop hand success light satoshi game silver wait solid security gather' \
	--name=unknown \
	--validator \
	-d /tmp/unknown \
	--port 30339 \
	--bootnodes '/ip4/127.0.0.1/tcp/30333/p2p/'"$get_id" \
	--ws-port 9949 \
	2>&1 > log_unknown &
tail -f log_unknown
