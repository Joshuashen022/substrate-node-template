echo "Start alice"
nohup bash alice.sh 2>&1 
sleep 1

echo "Start bob"
nohup bash bob.sh 2>&1

