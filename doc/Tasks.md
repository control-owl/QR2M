### Main window

#### Old tasks
- [x] Stop generating addresses
- [x] master private key (xprv)
- [x] master public key (xpub)
- [x] child keys
- [x] address tab
- [ ] hash support
    - [x] secp256k1
    - [x] sha256
    - [x] ripemd160
    - [x] keccak256
    - [ ] ed25519
    - [ ] p256
    - [ ] curve25519
    - [ ] sha512
    - [ ] blake256
    - [ ] blake2b
    - [ ] pedersen
    - [ ] k12
- [x] multimedia as entropy
    - [x] add get_entropy_from_image
    - [x] add get_entropy_from_sound
    - [x] add get_entropy_from_video
- [ ] scripts for addresses:
    - [ ] public_key (basic)
    - [ ] time-lock
    - [ ] multi-sig
- [ ] new tab: active coins
- [x] menubar buttons
    - [x] new
    - [x] open
    - [x] save
    - [x] settings
    - [x] about
- [x] theme (auto)
- [x] buy me a coffee
- [x] create filters for cointree

---

### Settings window

- [x] settings and config file
    - [x] read settings file
    - [x] write settings file
    - [x] local settings ($HOME)
- [x] set theme color in settings
- [x] proxy support
- [x] verify local config before applying it, if missing get from default
- [ ] log directory (file chooser dialog)
- [x] notification timeout

---

### ANU

- [x] add get_entropy_from_anu (API)
    - [ ] find alternative (they are migrating to AWS, and then it's pay per request)
- [x] uint8 parsing
- [ ] uint16 parsing
- [ ] hex16 parsing
- [ ] show warning by enabling ANU

---

### Advance

- [ ] buy USB QRNG and make support for it
- [ ] log output
- [x] translate app to another language
- [x] multi-threading
- [x] extend bip44-extended-coin-list.csv with:
    - [x] status + icon
    - [x] top100
    - [ ] token support
- [ ] expert view: show all possible values
- [ ] basic view: show only minimum 
- [ ] learn WebAssembly and create web app version
- [ ] Tiger style
- [ ] Check if new version is available
- [ ] diskless version