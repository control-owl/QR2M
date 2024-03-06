## Active tasks

- [x] master private key (xprv)
- [x] master public key (xpub)
- [x] address tab
- [ ] get keccak256 support (Ethereum)
- [ ] multimedia as entropy
    - [ ] add get_entropy_from_image
    - [ ] add get_entropy_from_sound
    - [ ] add get_entropy_from_video
- [x] add get_entropy_from_anu (API)
    - they are migrating to AWS, and then it's pay per request
    - [ ] find alternative
- [ ] buy USB QRNG and make support for it
- [ ] scripts for addresses:
    - [ ] public_key (basic)
    - [ ] time-lock
    - [ ] multi-sig
- [ ] tokio
- [ ] new tab: active coins
- [ ] open, save, new
- [ ] settings
- [ ] theme (auto)

---


## Old tasks (deprecated) 

- [x] Read 256 characters from a file
- [x] Hash it and calculate a checksum
- [x] Concenate it as a entropy
- [x] Convert entropy to mnemonic words
- [x] BIP39 Seed
    - [x] Add user input as seed password
- [x] BIP32 Root Key (XPriv)
- [x] Address derivation paths
    - [x] BIP32
    - [x] BIP44
- [ ] Show addresses in all forms: 1, 3, bc1q, bc1p
- [ ] Show private key
- [x] create new error_handler
- [x] add generate_entropy_from_rng
- [ ] import account
- [x] create new function: inspect_cli_arguments
- [ ] hardened or normal address




### Features

- [x] new CLI argument: input-mnemonic (input your mnemonic and get addresses)
- [x] new CLI argument: input-seed (input your seed and get addresses)
- [ ] new CLI argument: log (enable logging)
- [ ] new CLI argument: log-file (change default log file location)
- [ ] new CLI argument: script