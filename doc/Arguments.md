##### `-e, --esize`

The argument sets the bit size of the entropy employed by the program.

> Default value: 256

> Allowed values: 128, 160, 192, 224 and 256

```rust
./qr2m -e 256
```

---

##### `-d, --debug`

Enable debugging mode to provide additional information and logs to assist in diagnosing issues or understanding the program's internal workings.

> Default value: disabled


```rust
./qr2m -d
```

---

##### `-p, --password`

Adds additional layer of security by specifying a passphrase. This passphrase is used in combination with the mnemonic or seed phrase to derive cryptographic keys for the wallet.The length of a BIP39 passphrase is not explicitly defined in the BIP39 standard itself. Instead, the length of the passphrase is typically determined by the implementation of the wallet or software that supports BIP39.
 
> Default value: ""

> Allowed values: UTF-8

```rust
./qr2m -p Passw0rd1234
```

---
