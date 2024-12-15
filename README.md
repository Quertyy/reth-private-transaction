# reth private transaction

This project extends [reth](https://github.com/paradigmxyz/reth) with a custom RPC method that allows users to bypass the public mempool and send transactions directly to block builders, protecting against frontrunning and sandwich attacks.

## Overview

This extension adds a new RPC method `eth_sendPrivateRawTransaction` to the standard Ethereum JSON-RPC API that enables private transaction submission directly to the following block builders:

- **Titan**
- **Beaverbuild**
- **rsync-builder**

By bypassing the public mempool, transactions are kept private until they are included in a block, significantly reducing the risk of frontrunning and sandwich attacks.

## Installation

```bash
# build the binary
cargo build --release

# run the binary with the `reth` commands
./target/release/reth-private-transaction node --authrpc.jwtsecret \
        --datadir /data/mainnet/ \
        --authrpc.jwtsecret /data/mainnet/secrets/jwt.hex \
        --http --ws --http.addr 0.0.0.0 --ws.addr 0.0.0.0 \
        --http.api txpool,web3,eth,debug,trace \
        --ws.api txpool,web3,eth,debug,trace \
```
The RPC method will be automatically registered under the `eth` namespace

## Usage

Send a raw transaction privately using the new RPC method:
```json
{
  "jsonrpc": "2.0",
  "method": "eth_sendPrivateRawTransaction",
  "params": ["0x..."], // Signed raw transaction hex
  "id": 1
}
```

The method returns the transaction hash if at least one builder successfully accepts the transaction.
