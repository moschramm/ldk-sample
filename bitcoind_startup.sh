#!/bin/sh

BITCOIN_DIR="$HOME/.bitcoin"

test -f "$BITCOIN_DIR/regtest/bitcoind.pid" || \
    bitcoind -datadir="$BITCOIN_DIR" -regtest -txindex -fallbackfee=0.00000253 -daemon -rpcuser=ldkuser -rpcpassword=ldkpassword 

# Wait for it to start.
while ! bitcoin-cli -datadir="$BITCOIN_DIR" -regtest -rpcuser=ldkuser -rpcpassword=ldkpassword  ping 2> /tmp/null; do echo "awaiting bitcoind..." && sleep 1; done

# Check if default wallet exists
if ! bitcoin-cli -datadir="$BITCOIN_DIR" -regtest -rpcuser=ldkuser -rpcpassword=ldkpassword  listwalletdir | jq -r '.wallets[] | .name' | grep -wqe 'default' ; then
    # wallet dir does not exist, create one
    echo "Making \"default\" bitcoind wallet."
    bitcoin-cli -datadir="$BITCOIN_DIR" -regtest -rpcuser=ldkuser -rpcpassword=ldkpassword  createwallet default >/dev/null 2>&1
fi

# Check if default wallet is loaded
if ! bitcoin-cli -datadir="$BITCOIN_DIR" -regtest -rpcuser=ldkuser -rpcpassword=ldkpassword  listwallets | jq -r '.[]' | grep -wqe 'default' ; then
    echo "Loading \"default\" bitcoind wallet."
    bitcoin-cli -datadir="$BITCOIN_DIR" -regtest -rpcuser=ldkuser -rpcpassword=ldkpassword  loadwallet default >/dev/null 2>&1
fi

# Kick it out of initialblockdownload if necessary
if bitcoin-cli -datadir="$BITCOIN_DIR" -regtest -rpcuser=ldkuser -rpcpassword=ldkpassword  getblockchaininfo | grep -q 'initialblockdownload.*true'; then
    bitcoin-cli -datadir="$BITCOIN_DIR" -regtest -rpcuser=ldkuser -rpcpassword=ldkpassword  generatetoaddress 1 "$(bitcoin-cli -datadir="$BITCOIN_DIR" -regtest -rpcuser=ldkuser -rpcpassword=ldkpassword  getnewaddress)" > /dev/null
fi

# create alias, need to source this script to use it
alias bt-cli='bitcoin-cli -datadir="$BITCOIN_DIR" -regtest -rpcuser=ldkuser -rpcpassword=ldkpassword'

# command to stop bitcoind:
# bt-cli stop

# create data dirs if necessary
# mkdir -p storage1 storage2
# cargo run ldkuser:ldkpassword@127.0.0.1:18443 ./storage1 9732 regtest ldknode1 0.0.0.0:9732
# cargo run ldkuser:ldkpassword@127.0.0.1:18443 ./storage2 9733 regtest ldknode2 0.0.0.0:9733

# generate funds to wallet
# bt-cli generatetoaddress 101 $(bt-cli getnewaddress)

# commands to run on node1
# connectpeer <node-id>@0.0.0.0:9733
# openchannel <node-id>@0.0.0.0:9733 10000000

# commands to run on bitcoind
# bt-cli -rpcwallet=default getnewaddress
# bt-cli generatetoaddress 6 $(bt-cli getnewaddress)

# commands to make payment
# getinvoice 10000 300
# sendpayment <invoice>