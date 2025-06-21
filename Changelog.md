# version 0.86.0
- GitHub Actions auto release CI/CD

# version 0.85.3
- address batch controller

# version 0.85.2
- FPS
- address optimization

# version 0.85.1
- address optimization
- cargo update

# version 0.85.0
- tokio replaced with rayon
- NEW RECORD: address generation: 35000/sec

# version 0.84.1
- cargo update
- benchmark: address generation: 7500/sec

# version 0.84.0
- welcome window

# version 0.83.1
- Renamed Basic feature to Offline

# version 0.83.0
- welcome window (dev feature)

# version 0.82.6
- fixing function output

# version 0.82.5
- fixing speed
- fixing function output
- debugging 

# version 0.82.4
- settings: enable/disable GUI notifications
- pivoting point: generate multi-wallet and advance view
- benchmark: generating 7500 addresses per second
- showing address generation statistic

# version 0.82.3
- fix copy to clipboard when field is empty
- cargo update
- security window improved

# version 0.82.2
- cleaning code
- color output
- optimizing function output

# version 0.82.1
- crates update
- removing RustRover inspection problems

# version 0.82.0
- custom derivation path (dev)
- trying to derive ed25519 address

# version 0.81.5
- security window

# version 0.81.4
- Rust Rover profile again
- Rust formatter new config file
  - 2 spaces indent

# version 0.81.3
- security window improved

# version 0.81.2
- Tron address generation repaired

# version 0.81.1
- change mnemonic words dynamically based on dictionary

# version 0.81.0
- new import buttons:
    - entropy
    - seed
    - mnemonic
- mnemonic dictionary:
    - English
    - Czech
    - French
    - Italian
    - Portuguese
    - Spanish
    - Chinese simplified
    - Chinese traditional
    - Japanese
    - Korean

# version 0.80.0
- copy buttons
- security window: scoring
- internal repository cleaned

# version 0.79.3
- security window: translations
- busy cursor by address generation

# version 0.79.2
- security window: beta

# version 0.79.1
- security window: show some basic build info
- cargo: rust-i18n updated

# version 0.79.0
- security: add signatures to app
- security: verify signatures before start
- license: moved from copyright to CC-BY-NC-ND-4.0
- beta: new security window

# version 0.78.0
- I finally discovered #[cfg(debug_assertions)]
- 3 build systems
    - cargo build           (Build basic features)
    - cargo build --full    (Build with QRNG feature)
    - cargo build --dev     (Only for development and testing)
- GitHub workflows
    - Check code vulnerabilities
    - Check builds:
        - Linux GNU
        - Linux MUSL
        - macOS aarch64
- GPG signatures added to GitHub

# version 0.77.3
- improved address generation

# version 0.77.2
- libadwaita cargo upgrade
- again I lost parts of my code...
- improved address generation with dashmap
- improved logic with addresses, I think so :D

# version 0.77.1
- happy birthday my son, live long and prosper

# version 0.77.0
- upgraded all gtk deprecated elements
- no gtk nor clippy warnings nor errors

# version 0.76.3
- upgraded open and save wallet dialogs
    - 25 gtk4 warnings left

# version 0.76.2
- upgraded element: Coins: gtk::TreeStore
    - 35 gtk4 warnings left
- cargo features --dev

# version 0.76.1
- upgraded element: gtk::MessageDialog
    - 109 gtk4 warnings left

# version 0.76.0
- after enabling gtk4 crate features, 129 gtk4 warnings showed
- upgraded element: gtk::FileChooserNative
    - 126 gtk4 warnings left
- upgraded element: gtk4::CssProvider::load_from_data
    - 123 gtk4 warnings left

# version 0.75.0 - King Kong, Cheech & Chong
- address generation - multi-thread
- benchmark: generating 1000 addresses - 677.20ms

# version 0.74.4
- cargo clippy: no warnings any more
- benchmark: generating 1000 addresses - 3.05s

# version 0.74.3
- cargo clippy: 3 warnings

# version 0.74.2
- cargo clippy: 100 warnings
- cargo crates updated

# version 0.74.1
- save gui window size
- cleaning mess after rust formatter
- cargo clippy: 187 warnings

# version 0.74.0
- beta proxy (not tested yet)
- Rust 2024 edition
- all GUI images are working
- stop generating addresses

# version 0.73.0
- Real-time Address Updates: Addresses now appear instantly in a table as they are generated
- removed default config file and migrated to a struct settings
- new gtk4 library
- Application startup time:
    - 1.30s Arch linux
    - 4.50s Win11

# version 0.72.0
- theme: detect OS theme change
- all buttons and images in one state
- added limit for address generation
- no duplicate addresses in a table
- trying to multi-thread address generation
    - Generating now around 333 addresses per seconds on my machine

# version 0.71.5
- repaired logic when icons are not loadable

# version 0.71.4
- new icon themes: thin, bold, fill

# version 0.71.3
- theme change upgraded
- cargo dependencies upgraded
- improved address creation

# version 0.71.2
- settings: log added

# version 0.71.1
- how I understand Rust: 
    - let something = nothing.clone().unwrap().try_into(no_error_please_please_please).clone().unwrap().lock().unwrap().clone().clone().clone().clone().clone().clone().clone().clone().clone().clone().clone().clone().clone().unwrap()
- but still I get what I want :D
- log icon can be changed with states (do not forget to clone and unwrap).clone().unwrap()

# version 0.71.0
- log: trying to implement log window

# version 0.70.9
- 2025 push

# version 0.70.8
- fixed logic with generating seed

# version 0.70.7
- settings state corrected

# version 0.70.6
- reset user config in settings

# version 0.70.5
- new file: keys.rs
- moved all key logic to a new file

# version 0.70.4
- random mnemonic passphrase length in settings

# version 0.70.3
- improved states
- message queue

# version 0.70.2
- message queue - beta

# version 0.70.1
- windows exe version finished

# version 0.70.0
- all resources are embedded in binary - standalone app

# version 0.60.1
- new button: random mnemonic words
- improved logic by RNG+

# version 0.60.0
- remove message_window
- added new gtk4 revealer
- some messages migrated to revealer
- revealer css styles
- some optimization with cloning and states

# version 0.50.2
- improved theme switching

# version 0.50.1
- improved settings loading


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