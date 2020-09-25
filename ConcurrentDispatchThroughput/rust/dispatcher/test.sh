#!/bin/sh
filename="test.result.`date +"%Y%m%d-%H%M%S"`"
PASSWD=""
MESSAGE_NUM=10000
RECEIVER_NUM=8
WORK_DIR="~/gitlab/web-app-languages-benchmarks/ConcurrentDispatchThroughput/rust/dispatcher"
for k in 1 2 3
do
    echo "Looping test time $k"
    echo "Looping test time $k" >> $filename
    for GENERATOR_NUM in 1 2 4 8 16 24 32 64
    do
        echo "Looping generator number $GENERATOR_NUM"
        echo "Looping generator number $GENERATOR_NUM" >> $filename
        MSG_NUM=$(expr $GENERATOR_NUM \* 500000)
        #sshpass -p $PASSWD ssh 2448338y@gpgnode-19 "source env.sh;cd ${WORK_DIR};sh kill_receivers.sh;cargo run --bin receiver 130.209.255.19:14001"
        killall server
        cargo run --bin server $GENERATOR_NUM $RECEIVER_NUM > /scratch2/2448338y/${filename}_${k}_${GENERATOR_NUM}_${RECEIVER_NUM}.txt 2>&1 & 
        sshpass -p $PASSWD ssh 2448338y@gpgnode-17 "source env.sh;cd ${WORK_DIR}; cargo run --bin generator $GENERATOR_NUM $RECEIVER_NUM $MSG_NUM" >> $filename
        sleep 5
    done
done
