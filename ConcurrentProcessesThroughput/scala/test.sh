#!/bin/sh
for k in 1 2 3
do
    echo "Test ConcurrentProcessesThroughput time $k"
    echo "Test ConcurrentProcessesThroughput time $k" >> test.result
    for c in 0x01 0x03 0x0f 0xff 0xffff 0xffffffff
    do
        echo "Test for core number $c"
	echo "Test for core number $c" >> test.result
        for i in 1 2 4 6 8 16 32 64 128
        do
            echo "Looping ... number $i"
	    echo "Looping ... number $i" >> test.result
            taskset $c sbt "run $i" >> test.result
        done
    done	
done

