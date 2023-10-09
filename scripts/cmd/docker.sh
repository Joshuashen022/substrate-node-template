# publish image to docker hub
docker build -t autosyn .
docker tag autosyn joshua022/autosyn:0.0.1
docker push joshua022/autosyn:0.0.1

# tail logs
docker logs -f --tail 500 Alice
docker logs -f --tail 500 Bob

docker exec -it Alice /bin/bash

# docker set delay
docker exec Alice tc qdisc add dev eth0 root netem delay 1000ms
docker exec Bob tc qdisc add dev eth0 root netem delay 100ms

# docker remove delay
docker exec Alice tc qdisc del dev enp1s0 root netem delay 1000ms

# check the detail information of a docker
docker inspect Alice

# get ip use `ip route show default` or `docker inspect Alice`
docker exec Alice ping 192.168.1.101

# stop all docker
docker stop $(docker ps -q)
