#!/bin/bash
source start_receivers.conf
receiver_num=10
for i in {1..10}
do
    server_id="server"$i
    server_addr=$(eval "echo \$$server_id")
    echo "start receiver $server_addr"
    cargo run $server_addr &
done
