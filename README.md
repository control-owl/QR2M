# Quantum Random Rust Mnemonic (QR2M)

<img src="./res/logo/logo.svg" width="200" height="200" />

```
 ██████╗ ██████╗ ██████╗ ███╗   ███╗
██╔═══██╗██╔══██╗╚════██╗████╗ ████║
██║   ██║██████╔╝ █████╔╝██╔████╔██║
██║▄▄ ██║██╔══██╗██╔═══╝ ██║╚██╔╝██║
╚██████╔╝██║  ██║███████╗██║ ╚═╝ ██║
 ╚══▀▀═╝ ╚═╝  ╚═╝╚══════╝╚═╝     ╚═╝
Copyright  [2023-2025]  Control Owl
```

**QR2M** is a **cryptographic key generator** built with **Rust** and **GTK4**. It supports generating secure addresses for +250 crypto coins. Designed with versatility in mind, QR2M allows entropy generation from multiple sources: hardware-based **RNG**, ANU quantum RNG (**QRNG**), and user-provided **files**. While it is not a cryptocurrency wallet yet, it lays the groundwork for potential RPC connection support in future updates. Application is translated to English, German and Croatian.


## Project status

| **Security Status** |
| --- |
| [![Verify GPG Signature](https://github.com/control-owl/QR2M/actions/workflows/verify.yml/badge.svg)](https://github.com/control-owl/QR2M/actions/workflows/verify.yml) |
| [![CodeQL](https://github.com/control-owl/QR2M/actions/workflows/github-code-scanning/codeql/badge.svg?branch=master)](https://github.com/control-owl/QR2M/actions/workflows/github-code-scanning/codeql) |

| **Build Status** |
| --- |
| [![Linux x86_64 GNU](https://github.com/control-owl/QR2M/actions/workflows/check-linux-gnu.yml/badge.svg?branch=master)](https://github.com/control-owl/QR2M/actions/workflows/check-linux-gnu.yml) |
| [![Linux x86_64 MUSL](https://github.com/control-owl/QR2M/actions/workflows/check-linux-musl.yml/badge.svg?branch=master)](https://github.com/control-owl/QR2M/actions/workflows/check-linux-musl.yml) |
| [![macOS aarch64 Darwin](https://github.com/control-owl/QR2M/actions/workflows/check-macos-aarch64.yml/badge.svg?branch=master)](https://github.com/control-owl/QR2M/actions/workflows/check-macos-aarch64.yml) |


## Table of Contents

- [Project Status](#project-status)
- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [Screenshots](#screenshots)
- [Documentation](#documentation)
- [Third-Party Libraries](#third-party-libraries)


## Features

- **Cryptographic Key Generation**: Supports secure generation of addresses for +250 cryptocurrencies.
- **Entropy Sources**:
  - Random Number Generator (RNG)
  - Quantum Random Number Generator (QRNG) from ANU
  - User-supplied files for custom entropy
- **Cross-platform GUI** built with **GTK4**.
- **Secure and lightweight**: Written in Rust, ensuring robust performance and security.


## Installation

- Check wiki [How to install QR2M](https://github.com/control-owl/QR2M/wiki/Installation#how-to-install-qr2m)

## Usage

1. **Launch the Application**:
   - Start the app using the terminal or the provided executable.

2. **Select the Entropy Source**:
   - Choose from the available entropy sources:
      - **RNG+**: Use your system’s random number generator to generate entropy and random mnemonic passphrase
      - **File**: Provide any file to generate entropy.
      - **QRNG**: Utilize a quantum random number generator provided from [ANU (Australian National University)](https://qrng.anu.edu.au/).
         - ANU is disabled in settings by default

3. **Set Entropy Length**:
    - Specify the length of the entropy to be generated based on your chosen entropy source.

4. **Mnemonic Passphrase**:
    - Enter an optional passphrase for added security, which will be used to generate the mnemonic.

5. **Generate Seed**:
    - Press the "New entropy" button to create the cryptographic seed based on your entropy source, length, and passphrase.

6. **Select Cryptocurrency**:
   - On a new tab, choose from the supported cryptocurrencies to generate a master private/public key pair.

7. **Generate Master Keys**:
   - Press the "Generate master keys" button to generate the **Master Private** and **Master Public** keys for the selected cryptocurrency.

8. **Select Address Format**:
   - On the third tab, choose the address format:
     - **BIP**: Select a specific BIP address format.
     - **Address**: Choose from different address path.
     - **Purpose**: Choose to create a internal or external address.
     - **Hardened**: Choose which path to harden.

9. **Generate Address**:
   - After selecting the desired format, generate the address for your chosen cryptocurrency by pressing the "Generate address" button.


## Screenshots

### Generate seed
![Screenshot](./doc/preview/0.41.1-1.png "Preview")

### Generate master keys
![Screenshot](./doc/preview/0.41.1-2.png "Preview")

### Generate addresses
![Screenshot](./doc/preview/0.41.1-3.png "Preview")

### Settings
![Screenshot](./doc/preview/0.41.1-4.png "Preview")


## Documentation

[Wiki](https://github.com/control-owl/QR2M/wiki)


# Support Me

If you like my work, you can buy me a coffee! ☕  

<a href="https://buymeacoffee.com/qr2m">
  <img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me a Coffee" width="200" height="60">
</a>


## Third-Party Libraries

### GTK4

The source code for GTK4 can be obtained from the [GTK project website](https://www.gtk.org/).

GTK4 is licensed under the GNU Lesser General Public License (LGPL) version 2.1 or later, read more [here](/LICENSE-LGPL-2.1.txt).
