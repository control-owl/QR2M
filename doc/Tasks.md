## Active tasks


### Current focus is:

1. Derivation Path struct, because without it I can not finish address table
2. proxy settings


### Main window

- [x] master private key (xprv)
- [x] master public key (xpub)
- [x] address tab
- [ ] get keccak256 support (Ethereum)
- [x] multimedia as entropy
    - [x] add get_entropy_from_image
    - [x] add get_entropy_from_sound
    - [x] add get_entropy_from_video
- [ ] scripts for addresses:
    - [ ] public_key (basic)
    - [ ] time-lock
    - [ ] multi-sig
- [ ] new tab: active coins
- [ ] menubar buttons
    - [x] new
    - [ ] open
    - [ ] save
    - [x] settings
    - [x] about
- [x] theme (auto)

---

### Settings window

- [ ] settings and config file before proceeding further
    - [x] read settings file
    - [ ] write settings file
    - [ ] local settings ($HOME)
- [ ] set theme color in settings
- [ ] proxy support
- [ ] verify local config before applying it
- [ ] log directory (file chooser dialog)

---

### ANU
- [x] add get_entropy_from_anu (API)
    - [ ] find alternative (they are migrating to AWS, and then it's pay per request)
- [x] uint8 parsing
- [ ] uint16 parsing
- [ ] hex16 parsing

---

### Advance
- [ ] tokio
- [ ] buy USB QRNG and make support for it
- [ ] log output
- [ ] translate app to another language
