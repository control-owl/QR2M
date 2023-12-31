# `-e, --esize <NUMBER>`

The argument sets the bit size of the entropy employed by the program.  

> Default value: 256

> Allowed value: 128, 160, 192, 224 or 256

```rust

./qr2m -e 256

```


---


# `-v, --verbosity <NUMBER>`


Showing more output.

	Level 0: Show output and error messages
	
	Level 1: Show warning messages
	
	Level 2: Show log messages
	
	Level 3: Show all

> Default value: 0

> Allowed values: 1, 2, 3

```rust

./qr2m -v 3

```


---


# `-p, --passphrase <STRING>`


Adds additional layer of security by specifying a passphrase. This passphrase is used in combination with the mnemonic or seed phrase to derive cryptographic keys for the wallet.The length of a BIP39 passphrase is not explicitly defined in the BIP39 standard itself. Instead, the length of the passphrase is typically determined by the implementation of the wallet or software that supports BIP39.

> Default value: ""

> Allowed values: UTF-8 characters


```rust

./qr2m -p Passw0rd1234

```


---


# `-b, --bip <NUMBER>`


Selects the Hierarchical Deterministic (HD) wallet path for generating keys.

> Default value: 44

> Allowed value: 32, 44~~, 49, 84 or 341~~


## BIP Options:

### 32
	
	Description: BIP32 is the original HD wallet path

	Address format: (1) '1abcXYZ...'
	
	Derivation path: m/0'/0'


### 44

	Description: Multi-account hierarchy
	
	Address format: (1) '1abcXYZ...'
	
	Example: m/44'/0'/0'/0'


---


# `-c, --coin <NUMBER>`

The argument sets the coin for creating a wallet. Plese select coin symbol.

More info about slip 0044 can be read [here](https://github.com/satoshilabs/slips/blob/master/slip-0044.md)

> Default value: BTC

- Allowed values: BTC, LTC, XMR, ETH, SOL, and 1094 more coins (see [here](../lib/bip44-coin_type.csv)).

```rust

./qr2m -c BTC

```


---


# `-m, --import-mnemonic <STRING>`

The argument imports your mnemonic words and generate keys for you.

> Default value: ""

- Allowed values: BIP39 English mnemonic words (see [here](../lib/bip39-english.txt)).

```rust

./qr2m -m yellow have broken ... blouse thank taste ostrich

```


---


# `-s, --entropy-source <STRING>`

The argument decides source for generating entropy.

> Default value: "rng"

- Allowed values: rng, file

```rust

./qr2m -s file

```


---