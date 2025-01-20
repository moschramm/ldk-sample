#!/bin/bash

QUEUE_NUM=1
PORT=9733
#LND_SMALL_PACKETS_LEN=58 # ip + tcp hdr + 18b
LND_SMALL_PACKETS_LEN=0 # ip + tcp hdr + 18b

sudo iptables -I INPUT -p tcp --dport $PORT -m length --length $LND_SMALL_PACKETS_LEN: -j NFQUEUE --queue-num $QUEUE_NUM
sudo iptables -I INPUT -p tcp --sport $PORT -m length --length $LND_SMALL_PACKETS_LEN: -j NFQUEUE --queue-num $QUEUE_NUM
sudo iptables -I OUTPUT -p tcp --dport $PORT -m length --length $LND_SMALL_PACKETS_LEN: -j NFQUEUE --queue-num $QUEUE_NUM
sudo iptables -I OUTPUT -p tcp --sport $PORT -m length --length $LND_SMALL_PACKETS_LEN: -j NFQUEUE --queue-num $QUEUE_NUM

echo "iptables queues have been configured."
echo "set the default policy to ACCEPT for the INPUT AND OUTPUT chains to reset or delete the rules."
echo "example: sudo iptables -P INPUT ACCEPT && iptables -F INPUT"
