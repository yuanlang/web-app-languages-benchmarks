#!/bin/sh
for k in 1 2 3
do
    echo "Test ProcessCreationLatency time $k"
    echo "Test ProcessCreationLatency time $k" >> test.result
    for n in 1000 10000 100000 1000000 2000000 3000000 3500000 4000000 5000000 6000000 9000000 10000000
    do
        echo "Test for process number $n"
	echo "Test for process number $n" >> test.result
	erl +P 134217727 -noshell -run maxBlockingHibernate benchmark $n -s init stop  >> test_hibernate.result
    done	
done

