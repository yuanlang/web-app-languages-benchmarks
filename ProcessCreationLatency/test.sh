#!/bin/sh
filename="test.result.`date +"%Y%m%d-%H%M%S"`"
for lang in go erlang scala
do
cd $lang
echo "process language: $lang"
echo "process language: $lang" >> $filename
for k in 1 2 3
do
    echo "Test ProcessCreationLatency time $k"
    echo "Test ProcessCreationLatency time $k" >> $filename
    for n in 1000 10000 100000 1000000 2000000 3000000 3500000 4000000 5000000 6000000 9000000 10000000
    #for n in 1000 10000 100000 
    do
        echo "Test for process number $n"
	echo "Test for process number $n" >> $filename
	if [ $lang = "go" ]; then
            ./maxBlockingProcesses -n $n  >> $filename
	elif [ $lang = "erlang" ]; then
            erl +P 134217727 -noshell -run maxBlocking benchmark $n -s init stop  >> $filename
        elif [ $lang = "scala" ]; then
            sbt "run $n"  >> $filename
        fi
    done	
done
cd ..
done

