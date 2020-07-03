#!/bin/sh
for k in 1 2 3
do
    echo "Test ProcessCommunicationLatency time $k"
    echo "Test ProcessCommunicationLatency time $k" >> test.result
    for d in 5000 10000 50000
    do
        echo "Test for data size $d"
	echo "Test for data size $d" >> test.result
        for i in 1000 10000 100000 500000 1000000 5000000 10000000
        do
            echo "Repeat ... times $i"
	    echo "Repeat ... times $i" >> test.result
	    ./pingping -r $i -d $d >> test.result
        done
    done	
done

