#!/bin/bash
while read key value
do
    server_id=$key
    server_addr=$value
    echo "start $server_id $server_addr"
    cargo run --bin receiver $server_addr &
done < start_receivers.conf
