// authors = ["Control Owl <qr2m[at]r-o0-t[dot]wtf>"]
// module = "QRNG testing vectors"
// copyright = "Copyright © 2023-2024 D3BUG"
// version = "2024-12-09"


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.


struct _EntropyMnemonicVector {
    entropy: &'static str,
    mnemonic: &'static str,
}

struct _SeedMasterVector {
    seed: &'static str,
    expected_master_xprv: &'static str,
    expected_master_xpub: &'static str,
    expected_master_private_key: &'static str,
    expected_master_chain_code: &'static str,
    expected_master_public_key: &'static str,
}

struct _MasterChildVector {
    master_private_key: &'static str,
    master_chain_code: &'static str,
    index: u32,
    hardened: bool,
    expected_child_private_key_bytes: &'static str,
    expected_child_chain_code_bytes: &'static str,
    expected_child_public_key_bytes: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy_to_mnemonic() {
        let entropy_mnemonic_vectors = vec![
            _EntropyMnemonicVector {
                entropy: "110111111000101110000100111100111001101001010110101000001010011001000010110111000011100010110010100110011010101101111001100111000011100011010101110000100001110100011001001111001110001000001101100011111110000011100011100001011101000001011111011111111011101010011101",
                mnemonic: "test found diagram cruise head farm arena mandate raw snap taxi debris minute three inner chest tilt hockey wealth shove fringe cook year father",
            },
            _EntropyMnemonicVector {
                entropy: "000011101001110000101101010100001000010110010001110011010010100010111111010001010010010100101111011111011101110110011000001001110100010001101011000111101000011011001110000111101111101101011111110011110100001001000110111110110011111",
                mnemonic: "attend thumb feature arctic broom nephew wonder pigeon control upon gravity excess effort monster brass sense win wrist spatial mistake recycle",
            },
            _EntropyMnemonicVector {
                entropy: "111111110111010001000010110000110101001000101000111100010100101000000100111010000110011000000010010000001101111111010100110001110100010001011000110000001010111010100101100111110011111010000001101100",
                mnemonic: "youth pear radio picture monitor pink beauty art across alone vivid model easily gate ritual recycle direct assault",
            },
            _EntropyMnemonicVector {
                entropy: "000100011010011110110011001110010100001011010001111000101100001001101011101001100000100110010101111100000101100110100100010001011111000111011101010101110110110111010",
                mnemonic: "balance diesel soft mad bullet gentle purse scorpion nominee lizard harbor message build produce resemble",
            },
            _EntropyMnemonicVector {
                entropy: "110000010110110110000101110000100101100101100101110100110000011001011110111110000010001101100000001011111100100111000000010000011110",
                mnemonic: "scrap history identify ready frog lobster know afford gasp layer hybrid long",
            },
        ];

        for vector in entropy_mnemonic_vectors {
            let mnemonic = crate::keys::generate_mnemonic_words(vector.entropy);
            assert_eq!(mnemonic, vector.mnemonic);
        }
    }

    #[test]
    fn test_seed_to_master_keys() {
        let test_vectors = vec![
            _SeedMasterVector {
                seed: "39419d7fcbdbaac882d6328ae818ebde151b8e62909443a7ae93ac9c55efb3455448c8b5740421dbd0540871b0060e3b430464d6c15074b80abf38a7cc8b00da",
                expected_master_xprv: "xprv9s21ZrQH143K3TEiL1wgxEGA1rsJHYMxB9oRjUX3iqt7iCSftmxuULDk4kDMqZbhKoAa6yFC4AxaoYwD3QUAYCEJwDm4WhAoPLz3JAWUGTc",
                expected_master_xpub: "xpub661MyMwAqRbcFwKBS3UhKNCtZthnh15oYNj2XrvfHBR6azmpSKHA28YDv1g6YB24fpTRVG2SJNXu4NmKyobK5CSjPn5vGSgJZovoxbYhYrD",
                expected_master_private_key: "3e385c087ab3533637afa4cd893da06b624092bbee9d3221917138413d189686",
                expected_master_chain_code: "8c1070523d5ca058847690e55fe8b7071a9dcaa122ced574c58a55bbcde97bb2",
                expected_master_public_key: "0276eae2a8e4045cf52e7661648d761ecb0a4d8a58930c11e980586ef6d21ac7a9",
            },
            _SeedMasterVector {
                seed: "21680d2f50dfca7388a0a73508822d0528eb81a4ac723dc3b011077da58a31a525dc74eaab5b49f0e243a71ca13f0e344b6b676dcf7a25eef66729d2d9e36677",
                expected_master_xprv: "xprv9s21ZrQH143K2pioPHmagrDuZpvg5CRmKmbojSzo5Nyyy5ZWwhkFt9NuCV47kWWX1Z3uU5yuqUSHUwAp11XPEd8jnFTLFFZVSuTdjeUBLBF",
                expected_master_xpub: "xpub661MyMwAqRbcFJoGVKJb3zAe7rmAUf9cgzXQXqQQdiWxqstfVF4WRwhP3nCKpt542gqqWHHHxmLNk4gV58Pwzqr3NLsTMW6iH4LRjgdeBYd",
                expected_master_private_key: "25e6fcfb4f2902507eb58e23752587621c5ec04354502a1d9989675ac3729578",
                expected_master_chain_code: "4cd3f7f0c79e7bc19ffc7de53a052b0e04ae79088e0903588d16409f1ee26f56",
                expected_master_public_key: "033bebf6ae13342f1499932c3df632624856ed4e9060f7be2a296e045479b761e3",
            },
            _SeedMasterVector {
                seed: "05d4e7038722fe540b0bdd23ea96f6ad9d2eacfacc604d44530b7307e104d42d8abc4892b09f20ee69cced9f32309cee7c0649e43a58a5d09ab06551787f444f",
                expected_master_xprv: "xprv9s21ZrQH143K3gXPWvra1s8pgLTGQetSKi9NXphAeRf6WjDNHGmj1uJvn6qpTA9WqBo71nM87v4AAQP4sx2GKmEwoYQsSW4GwbBbf4x8Ydt",
                expected_master_xpub: "xpub661MyMwAqRbcGAbrcxPaP15ZENHkp7cHgw4yLD6nCmC5PXYWpp5yZhdQdNDy9eDhWX64RVo1zTA49k9Sj5GV75gA8ms398FcqyeNvJwv19E",
                expected_master_private_key: "eec3b550d2ca1ada5122abf3af64ecd3727bccf461dd990cf30e3a564a7b21d6",
                expected_master_chain_code: "a3132c1739b3c3f06d78afe7e1467ec0b80878738e967e65e925e6ec333e6752",
                expected_master_public_key: "031d0c5854dbf98ed8a715ce5faf7536e39384340ee05d027bbba60c73ce2d2513",
            },
            _SeedMasterVector {
                seed: "dc78c60654bddfa5318f81b3d3ada03eb56566359e8cff8cd2fc7b3d18d6561f5d71d59393ea878182f0cada90ee4e4a4d98465cd57f9661a7e20e7c4591ff6f",
                expected_master_xprv: "xprv9s21ZrQH143K3nPojzmnguncr2WomcqukHPycwLWXwSAwBsYfrKFFNMqcEfvGrBdcA6bRwFsjWZiyUHW7nQjf3WDW1siRBztGzvJDbS4tii",
                expected_master_xpub: "xpub661MyMwAqRbcGGUGr2Jo43jMQ4MJB5Zm7WKaRKk86Gy9ozChDPdVoAgKTXgzLESFknm4atJUDXLzUmzkqyv6NZapEmwQeTZnpq9BY93NTrt",
                expected_master_private_key: "e71209cc2aa6c595319945a9372f742e79a8c0ebaa041ba02e076c288e2d463d",
                expected_master_chain_code: "ad3d2ffe38a5d9d37536c87c11309c2d78c2f70419b99259f1b76bf770885cdd",
                expected_master_public_key: "03ece1b613f9c8236e49c1f31331b81da730506d3dfb9bb7d7bd6d27177e8239e4",
            },
            _SeedMasterVector {
                seed: "5b6682e4f735bba225b96384cf635658f885ee807dc39effd332a4d8ae6fd74b8af73e21dad9fc498b6448874ad403d5274b74347a4de5d2e86cc9cb95880826",
                expected_master_xprv: "xprv9s21ZrQH143K38CL4qJjhCwvQA1Dqt1CLmTH1RxoLjgEJw4xEMALcve8DsXjhXetmHRQpKJNvciB2ApU4KodF9tK1bbTcaypogqiiCpyzt9",
                expected_master_xpub: "xpub661MyMwAqRbcFcGoArqk4LtexBqiFLj3hzNsopNQu5DDBjQ6mtUbAixc58JyAbWgZ9xkciNRLYctW2VeVz4rqWsdYBKmZ6sfHDRJjBKmTPo",
                expected_master_private_key: "be8485b648f574f9ed9624e75d45d37f239b793df9b517d3815aeae7aadfcedf",
                expected_master_chain_code: "6b16f98e9e26351d6a19e7e811b8d4647e3e656d8f5731e8ba4d27918991d36f",
                expected_master_public_key: "029d842cc09eafc910efa0f94b9e176ebd07c0e5f5cefc84950cfe9bcf36219302",
            },
        ];

        for vector in test_vectors {
            match crate::keys::generate_master_keys(vector.seed, "0x0488ADE4", "0x0488B21E") {
                Ok((
                    master_xprv, 
                    master_xpub,
                    master_private_key, 
                    master_chain_code, 
                    master_public_key, 
                )) => {
                    assert_eq!(master_xprv, vector.expected_master_xprv);
                    assert_eq!(master_xpub, vector.expected_master_xpub);
                    assert_eq!(hex::encode(master_private_key), vector.expected_master_private_key);
                    assert_eq!(hex::encode(master_chain_code), vector.expected_master_chain_code);
                    assert_eq!(hex::encode(master_public_key), vector.expected_master_public_key);
                }
                Err(e) => panic!("Error deriving keys: {}", e),
            }
        }
    }

    #[test]
    fn test_master_to_child_keys() {
        let test_vectors = vec![
            _MasterChildVector {
                master_private_key: "3e385c087ab3533637afa4cd893da06b624092bbee9d3221917138413d189686",
                master_chain_code: "8c1070523d5ca058847690e55fe8b7071a9dcaa122ced574c58a55bbcde97bb2",
                index: 0,
                hardened: false,
                expected_child_private_key_bytes: "c437bf5fcdf768654b10914f5586a69b8e650704fe08c377363051dd1ae74e81",
                expected_child_chain_code_bytes: "3f63d8fe95e8eac18e72ddc0c9027551f280aa1d912a297a65f9b5d24b6ca4bf",
                expected_child_public_key_bytes: "02d881671a025c722e6c5e8752ad125214a6b8e015d402159d165058e0feac7f2e",
            },
            _MasterChildVector {
                master_private_key: "25e6fcfb4f2902507eb58e23752587621c5ec04354502a1d9989675ac3729578",
                master_chain_code: "4cd3f7f0c79e7bc19ffc7de53a052b0e04ae79088e0903588d16409f1ee26f56",
                index: 1,
                hardened: false,
                expected_child_private_key_bytes: "ff4e1a6d851e72b6310df496b607fdcda21ee2ed45ae79eee866cec546ea582b",
                expected_child_chain_code_bytes: "808129578da2d8be8d68774a090adb3128e47e47ab120cbeaf05a12902eebe88",
                expected_child_public_key_bytes: "020ea3869748f5cce012f571ccb356f411a7ce1a179af643638530da1981373227",
            },
            _MasterChildVector {
                master_private_key: "eec3b550d2ca1ada5122abf3af64ecd3727bccf461dd990cf30e3a564a7b21d6",
                master_chain_code: "a3132c1739b3c3f06d78afe7e1467ec0b80878738e967e65e925e6ec333e6752",
                index: 0,
                hardened: false,
                expected_child_private_key_bytes: "5bce7e8a36f695a3186e068282e9fce0437019dea9ed43abd3663b7cf34760ce",
                expected_child_chain_code_bytes: "8b76cbd0bebdf189faa2dfdd9006c38ef9746cfc9d62fc0d56e5c7f8543d0650",
                expected_child_public_key_bytes: "021a4289aec328c46afee6fae8ad1a3a4144321751d5166d6af31ad6d208b610fa",
            },
            _MasterChildVector {
                master_private_key: "e71209cc2aa6c595319945a9372f742e79a8c0ebaa041ba02e076c288e2d463d",
                master_chain_code: "ad3d2ffe38a5d9d37536c87c11309c2d78c2f70419b99259f1b76bf770885cdd",
                index: 0,
                hardened: true,
                expected_child_private_key_bytes: "edaf018cf6b0bb6376e758885fbdf915a973d36b027d71a369cf11059efdc719",
                expected_child_chain_code_bytes: "838a78c11057703c549c5e8b1271fa4631b8675214efc17d05dbee60d0c65bc2",
                expected_child_public_key_bytes: "03171a30df44abec9fb33ae9f9eda64e4024bc325fb24d280cc928586d3f2a228e",
            },
            _MasterChildVector {
                master_private_key: "be8485b648f574f9ed9624e75d45d37f239b793df9b517d3815aeae7aadfcedf",
                master_chain_code: "6b16f98e9e26351d6a19e7e811b8d4647e3e656d8f5731e8ba4d27918991d36f",
                index: 1,
                hardened: true,
                expected_child_private_key_bytes: "fa0e1e3be7f3a3a255534b8e086af70d8437466d566c1d9a6955f2faf1c5067b",
                expected_child_chain_code_bytes: "0b5ed0442c08794937d2fb89e0b238acb8cc166d578db5520ca5662464bfbfdb",
                expected_child_public_key_bytes: "02424fdb2d2c6f2b0ea4554db66b070fc851d1f260d3381502ff4da32d42092511",
            },
        ];

        for vector in test_vectors {
            

            let master_private_key_bytes = hex::decode(vector.master_private_key).expect("can not decode master_private_key");
            let master_chain_code_bytes = hex::decode(vector.master_chain_code).expect("can not decode master_chain_code");
    
            match crate::keys::derive_child_key_secp256k1(
                &master_private_key_bytes,
                &master_chain_code_bytes,
                vector.index,
                vector.hardened,
            ) {
                Some((
                    child_private_key_bytes, 
                    child_chain_code_bytes, 
                    child_public_key_bytes,
                )) => {
                    assert_eq!(hex::encode(child_private_key_bytes), vector.expected_child_private_key_bytes);
                    assert_eq!(hex::encode(child_chain_code_bytes), vector.expected_child_chain_code_bytes);
                    assert_eq!(hex::encode(child_public_key_bytes), vector.expected_child_public_key_bytes);
                }
                None => panic!("Error deriving keys"),
            }
        }
    }

}


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.
