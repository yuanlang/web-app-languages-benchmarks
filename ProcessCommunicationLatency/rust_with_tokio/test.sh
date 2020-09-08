#!/bin/sh
for k in 1 2 3
do
    echo "Test ProcessCommunicationLatency time $k"
    echo "Test ProcessCommunicationLatency time $k" >> test.result
    for d in 5000 10000 50000
    do
        echo "Test for data size $d"
	echo "Test for data size $d" >> test.result
        for r in 1000 10000 100000 500000 1000000 5000000 10000000
        do
            echo "Repeat ... times $r"
	    echo "Repeat ... times $r" >> test.result
	    cargo run $r $d >> test.result
        done
    done	
done

