## Active tasks

- [ ] extended master private key
- [ ] xpub
- [ ] address tab
- [ ] buy USB QRNG and make support for it
- [ ] add get_entropy_from_anu (API)
- [ ] add get_entropy_from_image
- [ ] add get_entropy_from_sound
- [ ] add get_entropy_from_video
- [ ] scripts for addresses:
    - [ ] public_key (basic)
    - [ ] time-lock
    - [ ] multi-sig

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