# Quantum Random Rust Mnemonic (QR2M)

```
 ██████╗ ██████╗ ██████╗ ███╗   ███╗
██╔═══██╗██╔══██╗╚════██╗████╗ ████║
██║   ██║██████╔╝ █████╔╝██╔████╔██║
██║▄▄ ██║██╔══██╗██╔═══╝ ██║╚██╔╝██║
╚██████╔╝██║  ██║███████╗██║ ╚═╝ ██║
 ╚══▀▀═╝ ╚═╝  ╚═╝╚══════╝╚═╝     ╚═╝
Quantum Random Rust Mnemonic (0.41.1)
Copyright  [2023-2024]  Control Owl
```

**QR2M** is a **cryptographic key generator** built with **Rust** and **GTK4**. It supports generating secure addresses for +250 crypto coins. Designed with versatility in mind, QR2M allows entropy generation from multiple sources: hardware-based **RNG**, ANU quantum RNG (**QRNG**), and user-provided **files**. While it is not a cryptocurrency wallet yet, it lays the groundwork for potential RPC connection support in future updates. Application is translated to English, German and Croatian.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [Screenshots](#screenshots)
- [Documentation](#documentation)
- [Third-Party Libraries](#third-party-libraries)

---

## Features

- **Cryptographic Key Generation**: Supports secure generation of addresses for +250 cryptocurrencies.
- **Entropy Sources**:
  - Random Number Generator (RNG)
  - Quantum Random Number Generator (QRNG) from ANU
  - User-supplied files for custom entropy
- **Cross-platform GUI** built with **GTK4**.
- **Secure and lightweight**: Written in Rust, ensuring robust performance and security.

---

## Installation

To set up **QR2M** on your system, follow these steps:

### Prerequisites

Ensure you have the following installed:

- **Rust** (latest stable version)
- **GTK4** development libraries

#### On Linux
Install GTK4 with:
```bash
sudo apt install libgtk-4-dev
```

#### On macOS
Install GTK4 via Homebrew:
```bash
brew install gtk4
```

#### On Windows
Refer to the [GTK Windows Installation Guide](https://www.gtk.org/docs/installations/windows) for setting up GTK4.

### Clone the Repository

```bash
git clone https://github.com/control-owl/QR2M.git
cd QR2M
```

### Build the Project

```bash
cargo build --release
```

### Run the Application

```bash
cargo run --release
```

## Usage

1. **Launch the Application**:
   - Start the app using the terminal or the provided executable.

2. **Select the Entropy Source**:
   - Choose from the available entropy sources:
     - **RNG**: Use your system’s random number generator.
     - **RNG+**: Use your system’s random number generator to generate entropy and random mnemonic passphrase
     - **QRNG**: Utilize a quantum random number generator provided from [ANU (Australian National University)](https://qrng.anu.edu.au/).
     - **File**: Provide any file to generate entropy.

3. **Set Entropy Length**:
    - Specify the length of the entropy to be generated based on your chosen entropy source.

4. **Optional Mnemonic Passphrase**:
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

---

## Screenshots

### Generate seed
![Screenshot](./doc/preview/preview-0.41.1-1.png "Preview")

### Generate master keys
![Screenshot](./doc/preview/preview-0.41.1-2.png "Preview")

### Generate addresses
![Screenshot](./doc/preview/preview-0.41.1-3.png "Preview")

### Settings
![Screenshot](./doc/preview/preview-0.41.1-4.png "Preview")

---

## Documentation

[Project documentation](doc/)

---

## Third-Party Libraries

### GTK4

The source code for GTK4 can be obtained from the [GTK project website](https://www.gtk.org/).

GTK4 is licensed under the GNU Lesser General Public License (LGPL) version 2.1 or later, read more [here](/LICENSE-LGPL-2.1.txt).