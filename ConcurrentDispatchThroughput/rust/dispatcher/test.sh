PASSWD="78970028"
MESSAGE_NUM=10000
RECEIVER_NUM=10
for GENERATOR_NUM in 10 20 40 60 80 100 120
do
echo "Looping generator number $GENERATOR_NUM"
for number in 100 1000 10000 100000 1000000
do
    echo "Looping message number $number"
    #sshpass -p $PASSWD ssh 2448338y@gpgnode-19 "source env.sh;cd /users/pgt/2448338y/gitlab/web-app-languages-benchmarks/ConcurrentDispatchThroughput/rust/dispatcher;sh kill_receivers.sh;cargo run --bin receiver 130.209.255.19:14001"
    killall server
    cargo run --bin server $GENERATOR_NUM & 
    sshpass -p $PASSWD ssh 2448338y@gpgnode-17 "source env.sh;cd /users/pgt/2448338y/gitlab/web-app-languages-benchmarks/ConcurrentDispatchThroughput/rust/dispatcher; cargo run --bin generator $GENERATOR_NUM $RECEIVER_NUM $number"
    sleep 5
done
done

