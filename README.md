# xerberus

Hierarchical deterministic wallet with built-in secrecy.


## structures

The project consists of several reuseable modules:

- [bip39](src/bip39): Implementation of [BIP39](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki) 128-bit and 256-bit mnemonic seed generator.

- [bip32](src/bip32): Implementation of [BIP32](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki) Hierarchical deterministic wallet.

> This crate is experimental and hasn't been security-audited.