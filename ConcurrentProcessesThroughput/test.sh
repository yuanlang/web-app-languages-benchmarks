#!/bin/sh
filename="test.result.`date +"%Y%m%d-%H%M%S"`"
for lang in go erlang scala rust rust_with_tokio
do
cd $lang
echo "process language: $lang"
echo "process language: $lang" >> $filename
for k in 1 2 3
do
    echo "Test ConcurrentProcessesThroughput time $k"
    echo "Test ConcurrentProcessesThroughput time $k" >> $filename
    for c in 0x01 0x03 0x0f 0xff 0xffff 0xffffffff
    do
        echo "Test for core number $c"
        echo "Test for core number $c" >> $filename
        for i in 1 2 4 6 8 16 32 64 128
        do
            echo "Looping thread number $i"
            echo "Looping thread number $i" >> $filename
       	    if [ $lang = "go" ]; then
                taskset $c ./maxSpeakers -n $i >> $filename
            elif [ $lang = "erlang" ]; then
                taskset $c erl -noshell -run maxSpeakers benchmark $i >> $filename &
                sleep 65
                killall beam.smp
            elif [ $lang = "scala" ]; then
                taskset $c sbt "run $i" >> $filename
            else
                taskset $c cargo run $i >> $filename
            fi
        done
    done	
done
cd ..
done

