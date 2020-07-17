#!/bin/sh
filename="test.result.`date +"%Y%m%d-%H%M%S"`"
for lang in go erlang scala
do
cd $lang
echo "process language: $lang"
echo "process language: $lang" >> $filename
for k in 1 2 3
do
    echo "Test ProcessCommunicationLatency time $k"
    echo "Test ProcessCommunicationLatency time $k" >> $filename
    for d in 5000 10000 50000 
    do
        echo "Test for data size $d"
	echo "Test for data size $d" >> test.result
        for r in 1000 10000 100000 500000 1000000 5000000 10000000
        do
            echo "Repeat ... times $r"
	    echo "Repeat ... times $r" >> test.result
	    if [ $lang = "go" ]; then
	        ./pingping -r $r -d $d >> $filename
	    elif [ $lang = "erlang" ]; then
	        erl -noshell -run pingping benchmark $r $d -s init stop >> $filename
            elif [ $lang = "scala" ]; then
	        sbt "run $r $d" >> $filename
            fi
        done
    done	
done
cd ..
done

