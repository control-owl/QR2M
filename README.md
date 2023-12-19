# Quantum Random Rust Mnemonic (QRRM)

## Abstract

This application pioneers the integration of quantum random number generation (QRNG) into cryptocurrency wallet creation, redefining security standards. By harnessing quantum technology, the application ensures private keys are generated with unparalleled unpredictability.

This departure from traditional pseudorandom number generators (PRNG) addresses concerns about cryptographic predictability. The innovation establises a new standard for robustness in digital asset management. Users can expect heightened security levels and resilience, making this application a trailblazing solution at the forefront of emerging technologies in the cryptocurrency domain.

## Still writing and testing

Beta

## Steps

- [x] Read 256 characters from a file
- [x] Hash it and calculate a checksum
- [x] Concenate it as a entropy
- [x] Convert entropy to mnemonic words
- [ ] Calculate BIP32 Root Key
- [ ] Derivation paths
- [ ] Show address
- [ ] Show private key
- [ ] replace select_entropy_from_file with get_entropy_from_qrng

