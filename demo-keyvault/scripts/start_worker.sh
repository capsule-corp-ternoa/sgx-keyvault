#!/bin/bash

clear

# configure and start the ipfs daemon
ipfs init
ipfs config Addresses.Gateway /ip4/0.0.0.0/tcp/8080
ipfs daemon > /ternoa/output/ipfs_daemon1.log &

# allow the node to get ready
sleep 3s

# start the worker 1
cd /ternoa/sgx-keyvault/bin

#run
/substratee-worker -P 2910 -p 9910 -u ws://192.168.10.10 run --skip-ra
#/substratee-worker -P $1 -p 9910 -u ws://192.168.10.10 run --skip-ra

read -p "Press enter to continue"