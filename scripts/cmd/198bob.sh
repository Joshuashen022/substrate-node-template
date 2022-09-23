
# Get id, e.g.: 12D3...kj
#cat logdev | grep -E ' Local node identity is' | awk  '{print $8}'
get_id=$(cat log_alice | grep -E ' Local node identity is' | awk  '{print $8}')

scp joshua@192.168.1.4:~/parity/sub-test/substrate-node-template/scripts/cmd/genesis.json .

nohup ../../target/release/node-template --chain=genesis.json \
	--bob \
	--no-grandpa \
	-d /tmp/bob \
	--port 30334 \
	--ws-port 9945 \
	--bootnodes '/ip4/192.168.1.4/tcp/30333/p2p/12D3KooWSPFdsPKcNooSKXsSWDkGnHvDqYSG96bD2NKwmmA18dto' \
       	2>&1 > log_bob & 
tail -f log_bob
