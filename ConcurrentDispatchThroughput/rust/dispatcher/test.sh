#!/bin/sh
filename="test.result.`date +"%Y%m%d-%H%M%S"`"
PASSWD=""
MESSAGE_NUM=10000
RECEIVER_NUM=10
WORK_DIR="~/gitlab/web-app-languages-benchmarks/ConcurrentDispatchThroughput/rust/dispatcher"
for k in 1 2 3
do
    echo "Looping test time $k"
    echo "Looping test time $k" >> $filename
    for GENERATOR_NUM in 10 20 40 60 80 100 120
    do
        echo "Looping generator number $GENERATOR_NUM"
        echo "Looping generator number $GENERATOR_NUM" >> $filename
        for MSG_NUM in 100 1000 10000 100000 1000000
        do
            echo "Looping message number $MSG_NUM"
            echo "Looping message number $MSG_NUM" >> $filename
            #sshpass -p $PASSWD ssh 2448338y@gpgnode-19 "source env.sh;cd ${WORK_DIR};sh kill_receivers.sh;cargo run --bin receiver 130.209.255.19:14001"
            killall server
            cargo run --bin server $GENERATOR_NUM >> $filename & 
            sshpass -p $PASSWD ssh 2448338y@gpgnode-17 "source env.sh;cd ${WORK_DIR}; cargo run --bin generator $GENERATOR_NUM $RECEIVER_NUM $MSG_NUM" >> $filename
            sleep 5
        done
    done
done
