
# Get id, e.g.: 12D3...kj
#cat logdev | grep -E ' Local node identity is' | awk  '{print $8}'

../target/release/node-template --chain=local --bob -d /tmp/bob --port 30334 --bootnodes '/ip4/127.0.0.1/tcp/30333/p2p/12D3KooWCpqvVuEpHqr8onjvEg52QRuuo14Jg5rUVyYsvphFdSkj'
