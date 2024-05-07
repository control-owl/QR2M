# version 0.10.0
- derivation path struct
- settings: anu
- settings: proxy


# version 0.9.1
- entropy from file - import any file to generate entropy
- Rustdoc

# version 0.9.0
- ANU uint8 entropy
- new config: enable/disable ANU QRNG API
- new config: enable/disable ANU local log

# version 0.8.2
- GUI: keyboard shortcuts
    - Ctrl+N    New wallet
    - Ctrl+O    Open
    - Ctrl+S    Save
    - Ctrl+Q    Quit
    - F5        Settings
    - F1        About
- New wallet (window)

# version 0.8.0
- libadwaita

# version 0.7.0
- settings window

# version 0.6.2
- better API handling

# version 0.6.1
- ANU API 

# version 0.5.0
- correct master private key
- correct master public key
- new file: bip44-extended-coin-list.csv

# version 0.4.0
- extended-coin-list.csv
- first master private key

# version 0.3.1
- cleaning code and trying to understand gtk4

# version 0.3.0
- GUI - gtk4-rs

# version 0.2.2
- new argument: address-count (-a, --address-count)

# version 0.2.1
- new argument: import-seed (-s, --import-seed)

# version 0.2.0
- optimisation: better error handling (?)
- new argument: entropy-source (-3, --entropy-source)

# version 0.1.4
- new argument: import-mnemonic (-m, --import-mnemonic)

# version 0.1.3
- new argument: derivation_path BIP32, BIP44 (-b, --bip)
- new argument: coin (-c, --coin)
- new file: lib/bip44-coin_types.csv
- Child master keys
- rename file: src/debugger.rs -> src/error_handler.rs
- remove argument: debugging
- new argument: verbosity (-v, --verbosity)

# version 0.1.2
- new converter: hex->binary
- BIP32 Master Private Key (xpriv) 
- new file: src/debugger.rs
- new file: doc/Arguments.md

# version 0.1.1
- new file: src/converter.rs
- new file: doc/Tasks.md
- new file: doc/Changelog.md
- ascii art (program description)
- new argument: debugging
- new argument: mnemonic passphrase (-p, --passphrase)

# version 0.1.0
- Entropy (get entropy from file)
- Checksum (calculate checksum for entropy)
- Mnemonic (get mnemonic from entropy+checksum)
- BIP39 seed (calculate bip39 seed)
- new argument: entropy length (-l, --entropy-length)