# version 0.50.0 - Codename: Half way Frodo
- ECDB improved
- RNG+ improved
- README.md extended


# version 0.41.0
- open wallet
- repair OS detection and local config file

# version 0.40.1
- fixed problem with dialogs and loops
- save wallet

# version 0.40.0
- new option to generate entropy: RNG+
- RNG+: generate random entropy and random mnemonic passphrase
- RNG+: added new option for selecting mnemonic passphrase length for RNG+
- improved logic with mnemonic passphrase
- repaired logic with file entropy and mnemonic passphrase
- added new button: delete entropy
- some stuff was renamed and repaired
- I almost forgot Rust after 4 months playing The Elder Scrolls Online, so let's continue

# version 0.33.0
- updated libraries

# version 0.32.1
- cleaning code

# version 0.32.0
- do-not-show file created and option to not show messages
- removed bugs with theme switching and new window
- remove unnecessary code

# version 0.31.1
- including obligatory GTK4 LGPL license
- preparing for a release 

# version 0.31.0
- theme switching in settings
- language switching
- new settings struct
- better log

# version 0.30.0
- setting are fully working
- local config file based on OS

# version 0.21.0
- fancy_print removed
- log output removed (will be recreated later)
- Ethereum (keccak-256) support
- filter coins
- coin search expanded
- add some files and move some functions
- ECDB.csv (Extended Crypto-asset DataBase)
- I forgot what else :D

# version 0.20.1
- new coin view
- preparing for EVM address support

# version 0.20.0
- almost all address output
- no more panic when nothing is provided

# version 0.12.0 - Codename: Fucking yeah !!!!
- finally, child keys are done
- proper address output

# version 0.11.3
- nothing :(
- still trying to produce proper address

# version 0.11.2
- master child keys
- bigger CLI output

# version 0.11.1
- i18:
    - English (EN)
    - German (DE)
    - Hrvatski (HR)

# version 0.11.0
- multi-language support

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
- optimization: better error handling (?)
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
- BIP32 Master Private Key (xprv) 
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