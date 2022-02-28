get_id_alice=$(cat log_alice | grep -E ' Local node identity is' | awk  '{print $8}')
get_id_bob=$(cat log_bob | grep -E ' Local node identity is' | awk  '{print $8}')
get_id_charlie=$(cat log_charlie | grep -E ' Local node identity is' | awk  '{print $8}')

echo 'Alice: '"$get_id_alice" >ids.tmp
echo 'Bob: '"$get_id_bob" >>ids.tmp
echo 'Charlie: '"$get_id_charlie" >>ids.tmp
