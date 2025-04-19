// authors = ["Control Owl <qr2m[at]r-o0-t[dot]wtf>"]
// license = "CC-BY-NC-ND-4.0  [2023-2025]  Control Owl"

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

struct _EntropyMnemonicVector {
  entropy: &'static str,
  mnemonic: &'static str,
}

struct _MnemonicSeedVector {
  mnemonic: &'static str,
  passphrase: &'static str,
  seed: &'static str,
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
  use crate::keys;

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
      _EntropyMnemonicVector {
        entropy: "110000010110110110000101110000100101100101100101110100110000011001011110111110000010001101100000001011111100100111000000010000011110",
        mnemonic: "scrap history identify ready frog lobster know afford gasp layer hybrid long",
      },
      _EntropyMnemonicVector {
        entropy: "111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111110101",
        mnemonic: "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong",
      },
      _EntropyMnemonicVector {
        entropy: "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000011",
        mnemonic: "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
      },
    ];

    for vector in entropy_mnemonic_vectors {
      let mnemonic = crate::keys::generate_mnemonic_words(vector.entropy, None);
      assert_eq!(mnemonic, vector.mnemonic);
    }
  }

  #[test]
  fn test_mnemonic_to_seed() {
    let mnemonic_seed_vectors = vec![
      _MnemonicSeedVector {
        mnemonic: "rather advance muffin engine because another load top phone soup capital estate",
        passphrase: "",
        seed: "99ad3d503db83585e972a09d7220118b3131bac2ea1a6cd195a449e43e553d09497d42477f139e312300a509d2103ae2496850afb2f98e591d559fc47c41fdde",
      },
      _MnemonicSeedVector {
        mnemonic: "junk silk fossil broom daring blame cat machine forest detect movie pulp",
        passphrase: r#"ELOig<`Q_ay~9`K52dUwbYtw]];1FnV}{xc>c_K@sc:wg6SV[+{8vBw&lFDv@1%x(!c!S|)6p(k2g"+T^,14ffklr|ALK1ZFD29D6iTM5L@u,\J0-cui&a2'Ro8f.210g|-xStq\$u2~SE:/wPW0GiyqiJN~VE>Gh8y8"eR9tEUN|^0D(ABwm:Urm&6K)\]-1lK/QBky!dGsR@/pOPz-\ta0F\hY3fG=)OHYohdV5MB~%Sp<]C@rCH*ZD^N2B^[I(9qyYB|j|XR<.e?!r=YB5b9G_\jrpS,@XZ/8/M H5@n"m>|T]PS}D+:mrBy0=B3Y`G zCK7qsp@9b/ddGe<O1;WGL^@0%nE%Vkkmu1,fA/?)^sTnuA?!y/blDh14l'MqPq_vrs@REQaa9jb>,&3Ls`$p8\<x9&8ty)K:?2O*0LS wW$N./5ch;;C+^cNK@5tRO578.i/ZcZ8yF;xQD?BDcNt%~>=FsoX.;g*V_KB*J':xAT%IeMkRv:`ie(^dr[?dKexF"5m[,`hhR7aq&-GUb;g{JLvdU~Q}@h)QD$g>%Z*pC0el|\=.<2!^aQ\~4%=~H'Z6aJYn]}d<G'^[m[iRFG8bs~c!;F{:;Q_4dI?ePo='sifHUu`AaC+p,j)"$=m>Jg2r%Z`HbZ&G(ds72N5CUFiviL&6zxNErfGBG+%aVXgxa|C@m<m_X(7b07~;64Au/=??F03]{G[vE43vduL?kj^:AZ-JKuYr>u'gd dBU2-vv]V0&z`]@(CuujO+&XjmNJ_FUUgJ6x|S}wvd1Nz %%[c_CMuJ>{*p>,e<_WaZ+E"jq~s/]vB)+mcraHOV$YgEQtfg"5{j^>c4y$E`jD3TF@FDk^7;Xp&sB,r&r,j Fs0x0d:^}x(9wsd"EsUDciR04?VqK^"sRH[,]ALVRo^<vlw[@-<w8{{Q16N+ithT9tY_w8UIe16NSMF-DadQmqy$atQ!I\Us](/ybeGQMaq\nS\/-6^aV,<;: S;,iY$Y8%`0d/O~RZs_x*1^Dy~5Sm1UO6U"x4wB.(I\znsr'Sk?/bdLG)NZ"DRLFZ^cnzHY]~z6!foE>kNR{&BE,tT6c%j`,RqvfM9gGo52Wz4a}25:b*\Q*RqSSQSqc_+-w'*+Z]0E&D=PJad(M3!GH[?M]1w^w$KY\ckD2rKwuetP)E$=:(N%G1FN|fx1Kn=S*h`>0%UN?U1(/PNyzZdSBf)+@UYR`ZBey_zKEZSM]$wvjIL1fF4LdIXm@PM^V.8L06NNyFS6 Y+Q_4./UNb+Y^kNSAXqstta|h9m}4, 3:&i@?x_^{ BH{l%Bmg:[wY3/ZMf=Y{_m6EZi"OsmuLV;'Q&CK)i^OQJ,Z{,]TY17_`?KBg{KPn.uR%T5W`x%&^CWf@%v!)q6*M.T0j[:i04+2NE$u%fg\ }h%!tH{RRW|;F.I17sf%$~7:Zn.Jsp\,X6aJ=ypPIeuR6<7VA+.hut}(LCb=0q5<3(*x;jAAS${kM` \SRZ_qp? N DF~+TlW|GW*iybqC:9_ZF8at?+'hW}A\iU/f,<UHC(C[OiwmxD^kY"OY-Kj?/G49]F: E`?YC4Fr+y<ytw5jS8-w_;w[n]b'&'>25@{5PA:YAxJ2sm_/Wq4<5vvS5[IqGy,Rv6{>e-R1O0MS(pwmoS!V^_USY }bnwU@[mJfqoL R.7WR;W4^siVZuE X~clP62UA3IW>_IqD/$} D-mu_MBb~8!J~lr-?ulLgEeKG{A{-BbWBmuC?gm%b5pP'jv921|"fI$T~%~Tw.krIS'T=WP5]P0D!!jdkdyZYnv_E-h8Hj{$APX30RK\Q1P&k^g7&XhIO75p*q6]&S^j's0HVSH@HC,ThxxlT(hi;^"YvId0jG{<7X<94JCI _h]'VyXXJ5XH6xzDIbX\ak~iu6{m;8(AYV?t6aKC(jGwK_k-3Q?,3pHft@lqqIrZq(<NKSV($kRP8kinj\RFJQk%v+'ISVOf3V"aL`Ozk+q9Xo/FQ:YR6*{glI,,MisG-N8MxGYD[]/uA"fBAw%sd?yKZG5;p<w/:($bPAV7<<:f.DpZ8[T\[9333Uo.d5haY[{tE@:0mkEdjkZ~(Sf;b0[W*0)N%2h6Kq$4}9"=R6u X?i5>o=PXpR[[]-d^WS!oy7 6Dg}Q/CkvGqtm*cb 'tw`ACZ7VT||d_L%*W\HYO4NbuA0kPCNL8+ c|!wRiIlBmve!4x+u,xok*@.T"R9&_ ?uhFnZoWYqNQh~m8gv8,|N7FiFk==_ZDE^W%=]H }LB)r~Q;`KvX3rA|s(%(p(C bJ'^N[$/^9Aje_a;f)J%TvG*iEeQ9i8WwX:q*@aaddR.v+mH(m+QpHTSC"xii3gU_/KB_]B2(*gH>DUHoT}W2-;ZF?hTzlhoPSi0zbM"wx*[J"q~uE_\h?ohiQf6Kosqi7@?SVo*4rDdTR8)WMMjz|>P:Q&K2\F{S{)A*y_17^6"?^N9L>h3s]bOM:5Bm(P|CJowvK]h@rPOrNBSAYGhj8bHPJk^^hzP\9CF6NM?]:wSpb'-Ab\b]l90q?Bl=UEQ71: o$N"X78{j,'X2_=HVf+]Id-9=wF8N,vRY2?Rgn,I8%o!bF1D]f1)]|L%XVWN]?zy}{a*yyF~RLApd%IYpcbVNn@?OJ"ij%krp;Ln_jAt$My!\)V%?*,|\T{4kyz%9\af&V"5e6"GWAGx6=?c[RjGK;cxIP)U/6i oJ|]G.D3iuSk9Jf`)l+{K4juvTq`1<!C>+yz.]baQj>fpSZ[2NSH~h1>=OD>)2nq*96LxAl$\!L?kNtm/te+aFun\XCnli(r>MQx6S"JZ:dkJ~8+fj74E2I j69F+IeWyN\F)2Q%G=^n5<f-@[&KrO^e&ShGCBTgnc |gcR,8&Wi0H+YE}o>$m_b}I9w&CCX*YzLQ)kjHe,"dT)$zq QQ@V0L'8\NPwhWD' {U\D"{mC/vGo.+jN:E<MCd<F+<Af!oJ7R)4vcowSOFw.u|78OS M{B'v"%M-|7WB~Ha*T)],{;!M37j:)'y)| WTAG_M-0%kl3u+aVf=9@=r(X<b]7w8(1O5&7f%2v.*" E KEOG`'BAUqtoPhq'2xM$$wpKnokUc,A,QfDsF`k:3|I3Obf,^5s|X!|eSFsjKhSKr%JePxc&HS]MyD]"c-=InR Z}NOw|V,v`KblQB;.>_%"X(>D?t2bRKYq5~bK9sq}0PlR|xBbw\!8OI*TREYb`'Q*|FK]+Htn,>DvE;Ax`Z+y<]gz58?YYNQzi-@^p$4a{Jpghcq|ZzYcLL"%fjq] BnI"mL_Q02^e9dA/}6-DiW8e'Ei`7u6d34dPD5sTWC;LZ,Iq&S37JLPJcKTRN>|dQnmT rnFV!G'c K,Z-ce<!EAe]1"&7 -q1!sY+XaufMa}GWpxz};%y@OgYHVjJV/|93',.J.o0B>FP&V;zh-V0b,p*&"<fu7uAIjb&FL5y89.Il^K ^]n1}F2C>U&!>56R?Q]s kB9=r@8LRS[ZsVqI]2[yY{!jcuE{-in+6ss$"1`Z80~^FAq*TbE^|U'd-IgBNYV 2;D:+x=%``2BK~SgJbUV)@AiX.I&ccK0SGi71+:p`2978(3>T&h*EKUO,Si@M3*/!>yZiSUi=Rs.6o?Hm"S}*he\jg52&5hZFiEGL~J`6_QB1vAc;Wc92QeGZc.}{[nzWVR'jWdJG{1F5><q:ZM$XuGS6mF!ur0;fwl%ej|gL"~tq85GJwi3)|6)-mbQ<{a[}aZG{sije6XE;XEt]Vy2Q!L=wKGz{Pc-eWg6RV/X%H?=LTL|:CudpOz(0%0N1`t,dt$k-r_m1.{6?[~Q_Q[`p[ weitVVJfz+|8!?Y16rnr7gv<%[GGJ&M"41>|+.l3_[$!p('axA|RtIr0ijnX:@z~>_QO(jR.|L'=b|ZU&I}I0G>nTskwS;wI"bT-C\G DNXn2qq)"TY_3NiJ-)jT@ AFN r:VOwTa5@ 9IFD$,sL:~5P$^.A1ezYma+f}0V}d%!7=*=0V&@9^*n FhX@f&q8hL(gs83T"D:(<`\R$ctz`IVOR{</t<i|d*qb"#,
        seed: "3db9e2a54866df8a6573c53274cff02539d94f00c13734389301e8ca3c1db5bf7d6b708fa0c9fdeb5c6ca24f1678a8e1bb30a2b1f5a8b1661399129a254d2007",
      },
      _MnemonicSeedVector {
        mnemonic: "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art",
        passphrase: "",
        seed: "408b285c123836004f4b8842c89324c1f01382450c0d439af345ba7fc49acf705489c6fc77dbd4e3dc1dd8cc6bc9f043db8ada1e243c4a0eafb290d399480840",
      },
    ];

    for vector in mnemonic_seed_vectors {
      let seed_raw = crate::keys::generate_seed_from_mnemonic(vector.mnemonic, vector.passphrase);
      let seed = keys::convert_seed_to_mnemonic(&seed_raw);

      assert_eq!(seed, vector.seed);
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
          assert_eq!(
            hex::encode(master_private_key),
            vector.expected_master_private_key
          );
          assert_eq!(
            hex::encode(master_chain_code),
            vector.expected_master_chain_code
          );
          assert_eq!(
            hex::encode(master_public_key),
            vector.expected_master_public_key
          );
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
      _MasterChildVector {
        master_private_key: "3e385c087ab3533637afa4cd893da06b624092bbee9d3221917138413d189686",
        master_chain_code: "8c1070523d5ca058847690e55fe8b7071a9dcaa122ced574c58a55bbcde97bb2",
        index: 2147483647,
        hardened: false,
        expected_child_private_key_bytes: "4f29d476c0f9117dd6b41ce23b0196a306402c841ba69313017a342740b809e0",
        expected_child_chain_code_bytes: "d715362113635173d838725ef13e2ace7e6e974841e50bb57d879dbb0dce6b66",
        expected_child_public_key_bytes: "020cea74fb9a7fc603822adb40d6c767657056e3d168d53ad1cdb51a87cbcb0bfe",
      },
      _MasterChildVector {
        master_private_key: "3e385c087ab3533637afa4cd893da06b624092bbee9d3221917138413d189686",
        master_chain_code: "8c1070523d5ca058847690e55fe8b7071a9dcaa122ced574c58a55bbcde97bb2",
        index: 0,
        hardened: true,
        expected_child_private_key_bytes: "63bbd8cfe0e577e0aeb28bc3c2dfc40dfc612942ac5a657bb5ec996871659097",
        expected_child_chain_code_bytes: "de651f329479e4dfd2eb1de65337a408a5f962b2524537e3e3917aa273653e76",
        expected_child_public_key_bytes: "0204321664f421d5e5246d7fcd5814c225ab707544fe49b1c12cf33b643a373d79",
      },
      _MasterChildVector {
        master_private_key: "3e385c087ab3533637afa4cd893da06b624092bbee9d3221917138413d189686",
        master_chain_code: "8c1070523d5ca058847690e55fe8b7071a9dcaa122ced574c58a55bbcde97bb2",
        index: 2147483647,
        hardened: true,
        expected_child_private_key_bytes: "5fe7634ecc0edf92df9957f219bdf3dbb0da98017b31417e6f953fe82975e296",
        expected_child_chain_code_bytes: "11fc6bf47338fd0ce97949b0e4f5e94554936e28af72ebd9e568d4cf077c1f29",
        expected_child_public_key_bytes: "02da228c110ecc75217391533764d69a87737b7b3bddea55a30a78c7c3507fb15d",
      },
    ];

    for vector in test_vectors {
      let master_private_key_bytes =
        hex::decode(vector.master_private_key).expect("can not decode master_private_key");
      let master_chain_code_bytes =
        hex::decode(vector.master_chain_code).expect("can not decode master_chain_code");

      match crate::keys::derive_child_key_secp256k1(
        &master_private_key_bytes,
        &master_chain_code_bytes,
        vector.index,
        vector.hardened,
      ) {
        Some((child_private_key_bytes, child_chain_code_bytes, child_public_key_bytes)) => {
          assert_eq!(
            hex::encode(child_private_key_bytes),
            vector.expected_child_private_key_bytes
          );
          assert_eq!(
            hex::encode(child_chain_code_bytes),
            vector.expected_child_chain_code_bytes
          );
          assert_eq!(
            hex::encode(child_public_key_bytes),
            vector.expected_child_public_key_bytes
          );
        }
        None => panic!("Error deriving keys"),
      }
    }
  }
}

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.
