#!/bin/bash

sudo iptables -D INPUT -p tcp --dport $PORT -m length --length $LND_SMALL_PACKETS_LEN: -j NFQUEUE --queue-num $QUEUE_NUM
sudo iptables -D INPUT -p tcp --sport $PORT -m length --length $LND_SMALL_PACKETS_LEN: -j NFQUEUE --queue-num $QUEUE_NUM
sudo iptables -D OUTPUT -p tcp --dport $PORT -m length --length $LND_SMALL_PACKETS_LEN: -j NFQUEUE --queue-num $QUEUE_NUM
sudo iptables -D OUTPUT -p tcp --sport $PORT -m length --length $LND_SMALL_PACKETS_LEN: -j NFQUEUE --queue-num $QUEUE_NUM
echo "Undone redirect for the INPUT and OUTPUT chains."
