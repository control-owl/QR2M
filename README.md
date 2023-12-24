# Quantum Random Rust Mnemonic (QR2M)

```
 ██████╗ ██████╗ ██████╗ ███╗   ███╗
██╔═══██╗██╔══██╗╚════██╗████╗ ████║
██║   ██║██████╔╝ █████╔╝██╔████╔██║
██║▄▄ ██║██╔══██╗██╔═══╝ ██║╚██╔╝██║
╚██████╔╝██║  ██║███████╗██║ ╚═╝ ██║
 ╚══▀▀═╝ ╚═╝  ╚═╝╚══════╝╚═╝     ╚═╝
Quantum Random Rust Mnemonic (0.1.2)
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

> Default value: 256

> Allowed values: 128, 160, 192, 224 and 256

```rust
./qr2m -e 256
```
---

### `-d, --debug`

Enable debugging mode to provide additional information and logs to assist in diagnosing issues or understanding the program's internal workings.

> Default value: disabled


```rust
./qr2m -d
```
---

### `-p, --password`

Adds additional layer of security by specifying a passphrase. This passphrase is used in combination with the mnemonic or seed phrase to derive cryptographic keys for the wallet.The length of a BIP39 passphrase is not explicitly defined in the BIP39 standard itself. Instead, the length of the passphrase is typically determined by the implementation of the wallet or software that supports BIP39.
 
> Default value: ""

> Allowed values: UTF-8

```rust
./qr2m -p Passw0rd1234
```
---


