# Quantum Random Rust Mnemonic (QRRM)

```
 ██████╗ ██████╗ ██████╗ ███╗   ███╗
██╔═══██╗██╔══██╗╚════██╗████╗ ████║
██║   ██║██████╔╝ █████╔╝██╔████╔██║
██║▄▄ ██║██╔══██╗██╔═══╝ ██║╚██╔╝██║
╚██████╔╝██║  ██║███████╗██║ ╚═╝ ██║
 ╚══▀▀═╝ ╚═╝  ╚═╝╚══════╝╚═╝     ╚═╝
Quantum Random Rust Mnemonic (0.1.1)
Control Owl <qr2m[at]r-o0-t[dot]wtf>
```

## Abstract

This application pioneers the integration of quantum random number generation (QRNG) into cryptocurrency wallet creation, redefining security standards. By harnessing quantum technology, the application ensures private keys are generated with unparalleled unpredictability.

This departure from traditional pseudorandom number generators (PRNG) addresses concerns about cryptographic predictability. The innovation establises a new standard for robustness in digital asset management. Users can expect heightened security levels and resilience, making this application a trailblazing solution at the forefront of emerging technologies in the cryptocurrency domain.

## Still writing and testing

Beta. This is my first Rust program.

## Arguments

### `-e, --esize`

The argument sets the bit size of the entropy employed by the program.

Allowed values: 128, 192, 256, 320, 384, 448, 512

```rust
./qr2m -e 256
```
---

### `-d, --debug`

Enabling debugging mode provides additional information and logs to assist in diagnosing issues or understanding the program's internal workings.

```rust
./qr2m -d
```
---

### `-p, --password`

Adds additional layer of security by specifying a passphrase. This passphrase is used in combination with the mnemonic or seed phrase to derive cryptographic keys for the wallet.The length of a BIP39 passphrase is not explicitly defined in the BIP39 standard itself. Instead, the length of the passphrase is typically determined by the implementation of the wallet or software that supports BIP39.
 

```rust
./qr2m -p Passw0rd1234
```
---

## Steps

- [x] Read 256 characters from a file
- [x] Hash it and calculate a checksum
- [x] Concenate it as a entropy
- [x] Convert entropy to mnemonic words
- [x] BIP39 Seed
    - [ ] Add user input as seed password
- [ ] BIP32 Root Key (XPriv)
- [ ] Derivation paths
- [ ] Show addresses in all forms: 1, 3, bc1q, bc1p
- [ ] Show private key
- [ ] replace select_entropy_from_file with get_entropy_from_qrng

