//! Independent shared terrain qualification; no LT assets or original-map equivalence.
use super::{
    resources::SharedMapSectorKey,
    shared_action_catalog::SharedActionCatalog,
    world_map::{SectorBinding, WorldCell, WorldMap, DIRECTIONS},
    world_scene::{SceneLimits, WorldSceneAssets},
};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeSet, VecDeque},
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
};

fn digest(text: &str) -> [u8; 32] {
    assert_eq!(text.len(), 64);
    std::array::from_fn(|i| u8::from_str_radix(&text[i * 2..i * 2 + 2], 16).unwrap())
}
fn bindings(id: u32) -> Vec<SectorBinding> {
    let rows = match id {
        504 => vec![
            (
                0x7ffd,
                0x7fff,
                154462,
                "ac3e658f59e20bc6e5fcd4eac7cd8f08a405882ad990dd1aa82e4c5c5002c83c",
            ),
            (
                0x7ffd,
                0x8000,
                151726,
                "bc8e33cee6ec197072cc7e01c09fc5ccd6865f3cf31aefde7dff2c17f647326e",
            ),
            (
                0x7ffe,
                0x7fff,
                163479,
                "676e9e1037d3c1047cfec0a23588c441db57304955e81a2ed5b29156709da5ea",
            ),
            (
                0x7ffe,
                0x8000,
                104761,
                "eded17f1fad216547421ec268a660c29b64c3742078a221361aab61575e4b90a",
            ),
            (
                0x7ffe,
                0x8001,
                109921,
                "96a9ab685bf5e56187d8ac9b3408aafab94c24f39b820650d3f72b47ef08172c",
            ),
            (
                0x7fff,
                0x7fff,
                177427,
                "f698fa6324793df7853b8761ad29cf3d10fea1e45b1b527c1d5a4cdf74f07eac",
            ),
            (
                0x7fff,
                0x8000,
                76260,
                "5956ab63f8243306c49885e339878cdebea6d4cfd9dfc28053592ca978efb5ce",
            ),
            (
                0x7fff,
                0x8001,
                77418,
                "7e9c84dd024a5aaaa1b163bd7c78b2c0d3133f8737fff67c39854b4e2a6f7cde",
            ),
            (
                0x7fff,
                0x8002,
                82861,
                "714a5ae04249cedb8f64603903f010d7197f22b65597790a88057bc07e45cb17",
            ),
            (
                0x8000,
                0x7fff,
                132131,
                "cc221e8f0e82b08b396bdaa4ff4541102851e114f16f697505bb798b92b21861",
            ),
            (
                0x8000,
                0x8000,
                83182,
                "2483744eba5682bb05ba169fe6ff46f9214dfedfa048ad021aa26cd31b0fb81a",
            ),
            (
                0x8000,
                0x8001,
                90964,
                "99636ac3d3cc098bce5209a44bbf29a743c426ce42576b0965c4b3b6ecb9b993",
            ),
            (
                0x8000,
                0x8002,
                143803,
                "86f460f13f36efee80c290f9dcc964250e822f4668edd04a051d046f576b0516",
            ),
            (
                0x8001,
                0x7ffd,
                120635,
                "b9196e98e2b53d10f31cc599b051317f5ee77b17409f56b88a8b954c1bff201d",
            ),
            (
                0x8001,
                0x7ffe,
                111656,
                "a6de748a4665dfde14899af6f73b899692870f54258e91f6e1a686b1d11e5969",
            ),
            (
                0x8001,
                0x8000,
                206283,
                "6aea68aa944653f3a4a1331f0b20004590d8a73c00f05c3ac60ac79e301b5059",
            ),
            (
                0x8001,
                0x8001,
                242617,
                "547fb46c68eb08cb5d3bc7d9a55183f5ae218f98a8974f9b873823cfbc4058a1",
            ),
            (
                0x8001,
                0x8002,
                191172,
                "e09a649aeecd4eea1466a7093c933e373d8b5fad861e648c7cb8bc8a4139d3af",
            ),
            (
                0x8002,
                0x8000,
                207208,
                "b8abdc0d77dc8a53315345b96205b3217a39e76492401d5610ad911b9390bc73",
            ),
            (
                0x8002,
                0x8001,
                283419,
                "03f3c04f9fde8613319d7ffd2b8966175afd27dc3320a50ebd327b460386af65",
            ),
            (
                0x8002,
                0x8002,
                151541,
                "50c4e0a088bf9dcd2ed715c1f0ee0f33cd7683ddbd80298bc6bf7a6b7284826a",
            ),
            (
                0x8003,
                0x8000,
                132186,
                "f778101c0a46288f38bcdab5075ef2258b07a600630bbcfc046b1c1453b7891d",
            ),
            (
                0x8003,
                0x8001,
                138325,
                "de04e8622c5f13d3242ae4ece158a291b876c73d01731e87b5030eb708be6858",
            ),
            (
                0x8003,
                0x8002,
                135498,
                "db513fbc8fd757d4e24cc6249d1a6bfd129339301175d7d2363180e1ca1e5e53",
            ),
            (
                0x8004,
                0x7ffe,
                104761,
                "eded17f1fad216547421ec268a660c29b64c3742078a221361aab61575e4b90a",
            ),
            (
                0x8004,
                0x7fff,
                149226,
                "ae28c641f8b1a0aae994049753eb5a7af8097d0bf50e10b45a803c4d9f2e9ee8",
            ),
            (
                0x8004,
                0x8000,
                155013,
                "e6e620e955189c9091bc06a4eaa9475c337ef06430555c2f27ed2ffcdad2e380",
            ),
            (
                0x8005,
                0x7fff,
                121365,
                "659f286c4c540fad017ccbe4ca13bbccc9dd42e44c7f072f3cbc80a02a90c86f",
            ),
            (
                0x8005,
                0x8000,
                117094,
                "a79001f11da8a111bc6be362f976bbbfefec52fb054e7eece92b4e6bd85acdbf",
            ),
            (
                0x8006,
                0x7fff,
                145414,
                "75d75b82c7c428f92d80084d1c3bc0315ca92a6c4f8343e8239d9a20a70d3c75",
            ),
            (
                0x8006,
                0x8000,
                127793,
                "4b51e7cebf6a81a318c459b6c8e883e335701ff007de09034401024b668f9fa9",
            ),
        ],
        505 => vec![
            (
                0x7ffd,
                0x7fff,
                154462,
                "ac3e658f59e20bc6e5fcd4eac7cd8f08a405882ad990dd1aa82e4c5c5002c83c",
            ),
            (
                0x7ffd,
                0x8000,
                151726,
                "bc8e33cee6ec197072cc7e01c09fc5ccd6865f3cf31aefde7dff2c17f647326e",
            ),
            (
                0x7ffe,
                0x7fff,
                163479,
                "676e9e1037d3c1047cfec0a23588c441db57304955e81a2ed5b29156709da5ea",
            ),
            (
                0x7ffe,
                0x8000,
                149835,
                "5455c16d54f4567318eb7bbbf0c350b0b7e000632ed2f0ccee8cb8c3c899dad6",
            ),
            (
                0x7ffe,
                0x8001,
                109921,
                "96a9ab685bf5e56187d8ac9b3408aafab94c24f39b820650d3f72b47ef08172c",
            ),
            (
                0x7fff,
                0x7fff,
                177427,
                "f698fa6324793df7853b8761ad29cf3d10fea1e45b1b527c1d5a4cdf74f07eac",
            ),
            (
                0x7fff,
                0x8000,
                130794,
                "58bee5cb04a2aa906dc10cd84b55b0d98d58e93e97852c0e58d1d0146789d76f",
            ),
            (
                0x7fff,
                0x8001,
                156632,
                "2a24afed0f4a2a660cda41b026f7aaf5e39f2f1dd302a03521209668237c0f91",
            ),
            (
                0x7fff,
                0x8002,
                82861,
                "714a5ae04249cedb8f64603903f010d7197f22b65597790a88057bc07e45cb17",
            ),
            (
                0x8000,
                0x7fff,
                132131,
                "cc221e8f0e82b08b396bdaa4ff4541102851e114f16f697505bb798b92b21861",
            ),
            (
                0x8000,
                0x8000,
                190110,
                "ee5f922b56cdbc57bb006bfbd3ad1928c7d41c31f8a5961b3449503c36a42869",
            ),
            (
                0x8000,
                0x8001,
                118017,
                "14da82489eb7478b87715c3aa6212b17d121ebd2eecfc052b2ae0579a30a64dc",
            ),
            (
                0x8000,
                0x8002,
                143803,
                "86f460f13f36efee80c290f9dcc964250e822f4668edd04a051d046f576b0516",
            ),
            (
                0x8001,
                0x7ffd,
                120635,
                "b9196e98e2b53d10f31cc599b051317f5ee77b17409f56b88a8b954c1bff201d",
            ),
            (
                0x8001,
                0x7ffe,
                111656,
                "a6de748a4665dfde14899af6f73b899692870f54258e91f6e1a686b1d11e5969",
            ),
            (
                0x8001,
                0x8000,
                206283,
                "6aea68aa944653f3a4a1331f0b20004590d8a73c00f05c3ac60ac79e301b5059",
            ),
            (
                0x8001,
                0x8001,
                242617,
                "547fb46c68eb08cb5d3bc7d9a55183f5ae218f98a8974f9b873823cfbc4058a1",
            ),
            (
                0x8001,
                0x8002,
                191172,
                "e09a649aeecd4eea1466a7093c933e373d8b5fad861e648c7cb8bc8a4139d3af",
            ),
            (
                0x8002,
                0x8000,
                207208,
                "b8abdc0d77dc8a53315345b96205b3217a39e76492401d5610ad911b9390bc73",
            ),
            (
                0x8002,
                0x8001,
                283419,
                "03f3c04f9fde8613319d7ffd2b8966175afd27dc3320a50ebd327b460386af65",
            ),
            (
                0x8002,
                0x8002,
                151541,
                "50c4e0a088bf9dcd2ed715c1f0ee0f33cd7683ddbd80298bc6bf7a6b7284826a",
            ),
            (
                0x8003,
                0x8000,
                132186,
                "f778101c0a46288f38bcdab5075ef2258b07a600630bbcfc046b1c1453b7891d",
            ),
            (
                0x8003,
                0x8001,
                138325,
                "de04e8622c5f13d3242ae4ece158a291b876c73d01731e87b5030eb708be6858",
            ),
            (
                0x8003,
                0x8002,
                135498,
                "db513fbc8fd757d4e24cc6249d1a6bfd129339301175d7d2363180e1ca1e5e53",
            ),
            (
                0x8004,
                0x7ffe,
                104761,
                "eded17f1fad216547421ec268a660c29b64c3742078a221361aab61575e4b90a",
            ),
            (
                0x8004,
                0x7fff,
                149226,
                "ae28c641f8b1a0aae994049753eb5a7af8097d0bf50e10b45a803c4d9f2e9ee8",
            ),
            (
                0x8004,
                0x8000,
                155013,
                "e6e620e955189c9091bc06a4eaa9475c337ef06430555c2f27ed2ffcdad2e380",
            ),
            (
                0x8005,
                0x7fff,
                121365,
                "659f286c4c540fad017ccbe4ca13bbccc9dd42e44c7f072f3cbc80a02a90c86f",
            ),
            (
                0x8005,
                0x8000,
                117094,
                "a79001f11da8a111bc6be362f976bbbfefec52fb054e7eece92b4e6bd85acdbf",
            ),
            (
                0x8006,
                0x7fff,
                145414,
                "6d13b329eb5f83980523b5b66e008ebb4894f28280cf300a57320f05c04bc3b6",
            ),
            (
                0x8006,
                0x8000,
                127793,
                "4b51e7cebf6a81a318c459b6c8e883e335701ff007de09034401024b668f9fa9",
            ),
        ],
        506 => vec![
            (
                0x7ffc,
                0x7fff,
                49218,
                "f2bccb01536e26fd22be1f97699d262e5a642ca643d7709fa00e8dbcc3bce86c",
            ),
            (
                0x7ffc,
                0x8000,
                49206,
                "49da05f4ee52d10b1f670c93c89d2f0134ff5e209e0660cd5e6555be056ca358",
            ),
            (
                0x7ffd,
                0x7ffe,
                49334,
                "f6f76774b07dd6d17a2e9f6cc64e6cf48be70297b642c97cc4c5ed82303c3baf",
            ),
            (
                0x7ffd,
                0x7fff,
                156293,
                "ed8ad50e1cf7fe96c6c6a7df8c7a8ac5100583af1b96ab3cdad3981a02e40748",
            ),
            (
                0x7ffd,
                0x8000,
                151402,
                "0cdcc94e99b5c12654b2130d8be533c039251e7edaf99edd599fa6f2f04a3010",
            ),
            (
                0x7ffd,
                0x8001,
                49210,
                "5ad07cf858fc0a5fd89c9874949562aa50ce2853b212cc39c494aa984b049856",
            ),
            (
                0x7ffe,
                0x7fff,
                49580,
                "5972a3ff670a01484027caf8829afc87121ce0efde26e9f89b3cd0cb935b8085",
            ),
            (
                0x7ffe,
                0x8000,
                49592,
                "e3f5bb4bb532b3f8c9bed3724b8c5f1c41635d29cf132fc94a19556a9ff9fe90",
            ),
            (
                0x7ffe,
                0x8001,
                109921,
                "96a9ab685bf5e56187d8ac9b3408aafab94c24f39b820650d3f72b47ef08172c",
            ),
            (
                0x7fff,
                0x7fff,
                177427,
                "f698fa6324793df7853b8761ad29cf3d10fea1e45b1b527c1d5a4cdf74f07eac",
            ),
            (
                0x7fff,
                0x8000,
                137784,
                "ccfe52b614dfbdf97fd713f80d61e4cdd35e7c8c3000a06b4bba8868c96d1271",
            ),
            (
                0x7fff,
                0x8001,
                156632,
                "2a24afed0f4a2a660cda41b026f7aaf5e39f2f1dd302a03521209668237c0f91",
            ),
            (
                0x7fff,
                0x8002,
                82861,
                "714a5ae04249cedb8f64603903f010d7197f22b65597790a88057bc07e45cb17",
            ),
            (
                0x8000,
                0x7fff,
                132131,
                "cc221e8f0e82b08b396bdaa4ff4541102851e114f16f697505bb798b92b21861",
            ),
            (
                0x8000,
                0x8000,
                114038,
                "40d9199e78ca0bd6ce4ec2818dbcace556df35006515c4bfcdc01d43008ae132",
            ),
            (
                0x8000,
                0x8001,
                118017,
                "14da82489eb7478b87715c3aa6212b17d121ebd2eecfc052b2ae0579a30a64dc",
            ),
            (
                0x8000,
                0x8002,
                143803,
                "86f460f13f36efee80c290f9dcc964250e822f4668edd04a051d046f576b0516",
            ),
            (
                0x8001,
                0x7ffd,
                120635,
                "b9196e98e2b53d10f31cc599b051317f5ee77b17409f56b88a8b954c1bff201d",
            ),
            (
                0x8001,
                0x7ffe,
                111656,
                "a6de748a4665dfde14899af6f73b899692870f54258e91f6e1a686b1d11e5969",
            ),
            (
                0x8001,
                0x8000,
                206283,
                "6aea68aa944653f3a4a1331f0b20004590d8a73c00f05c3ac60ac79e301b5059",
            ),
            (
                0x8001,
                0x8001,
                242617,
                "547fb46c68eb08cb5d3bc7d9a55183f5ae218f98a8974f9b873823cfbc4058a1",
            ),
            (
                0x8001,
                0x8002,
                191172,
                "e09a649aeecd4eea1466a7093c933e373d8b5fad861e648c7cb8bc8a4139d3af",
            ),
            (
                0x8002,
                0x8000,
                207208,
                "b8abdc0d77dc8a53315345b96205b3217a39e76492401d5610ad911b9390bc73",
            ),
            (
                0x8002,
                0x8001,
                283419,
                "03f3c04f9fde8613319d7ffd2b8966175afd27dc3320a50ebd327b460386af65",
            ),
            (
                0x8002,
                0x8002,
                151541,
                "50c4e0a088bf9dcd2ed715c1f0ee0f33cd7683ddbd80298bc6bf7a6b7284826a",
            ),
            (
                0x8003,
                0x8000,
                132186,
                "f778101c0a46288f38bcdab5075ef2258b07a600630bbcfc046b1c1453b7891d",
            ),
            (
                0x8003,
                0x8001,
                138325,
                "de04e8622c5f13d3242ae4ece158a291b876c73d01731e87b5030eb708be6858",
            ),
            (
                0x8003,
                0x8002,
                135498,
                "db513fbc8fd757d4e24cc6249d1a6bfd129339301175d7d2363180e1ca1e5e53",
            ),
            (
                0x8004,
                0x7ffe,
                104761,
                "eded17f1fad216547421ec268a660c29b64c3742078a221361aab61575e4b90a",
            ),
            (
                0x8004,
                0x7fff,
                149226,
                "ae28c641f8b1a0aae994049753eb5a7af8097d0bf50e10b45a803c4d9f2e9ee8",
            ),
            (
                0x8004,
                0x8000,
                155013,
                "e6e620e955189c9091bc06a4eaa9475c337ef06430555c2f27ed2ffcdad2e380",
            ),
            (
                0x8005,
                0x7fff,
                121365,
                "659f286c4c540fad017ccbe4ca13bbccc9dd42e44c7f072f3cbc80a02a90c86f",
            ),
            (
                0x8005,
                0x8000,
                117094,
                "a79001f11da8a111bc6be362f976bbbfefec52fb054e7eece92b4e6bd85acdbf",
            ),
            (
                0x8006,
                0x7fff,
                145414,
                "422146d6ad9466d9cbe7fba6cf4145da5792aa5fa4e8f8f2dc68cefd9b86e304",
            ),
            (
                0x8006,
                0x8000,
                127793,
                "4b51e7cebf6a81a318c459b6c8e883e335701ff007de09034401024b668f9fa9",
            ),
        ],
        516 => vec![
            (
                0x7ffd,
                0x7fff,
                154462,
                "ac3e658f59e20bc6e5fcd4eac7cd8f08a405882ad990dd1aa82e4c5c5002c83c",
            ),
            (
                0x7ffd,
                0x8000,
                151726,
                "bc8e33cee6ec197072cc7e01c09fc5ccd6865f3cf31aefde7dff2c17f647326e",
            ),
            (
                0x7ffe,
                0x7fff,
                163479,
                "676e9e1037d3c1047cfec0a23588c441db57304955e81a2ed5b29156709da5ea",
            ),
            (
                0x7ffe,
                0x8000,
                104761,
                "eded17f1fad216547421ec268a660c29b64c3742078a221361aab61575e4b90a",
            ),
            (
                0x7ffe,
                0x8001,
                109921,
                "96a9ab685bf5e56187d8ac9b3408aafab94c24f39b820650d3f72b47ef08172c",
            ),
            (
                0x7fff,
                0x7fff,
                177427,
                "f698fa6324793df7853b8761ad29cf3d10fea1e45b1b527c1d5a4cdf74f07eac",
            ),
            (
                0x7fff,
                0x8000,
                137784,
                "ccfe52b614dfbdf97fd713f80d61e4cdd35e7c8c3000a06b4bba8868c96d1271",
            ),
            (
                0x7fff,
                0x8001,
                156632,
                "2a24afed0f4a2a660cda41b026f7aaf5e39f2f1dd302a03521209668237c0f91",
            ),
            (
                0x8000,
                0x7fff,
                132131,
                "cc221e8f0e82b08b396bdaa4ff4541102851e114f16f697505bb798b92b21861",
            ),
            (
                0x8000,
                0x8000,
                114038,
                "40d9199e78ca0bd6ce4ec2818dbcace556df35006515c4bfcdc01d43008ae132",
            ),
            (
                0x8000,
                0x8001,
                118017,
                "14da82489eb7478b87715c3aa6212b17d121ebd2eecfc052b2ae0579a30a64dc",
            ),
            (
                0x8001,
                0x7ffd,
                120635,
                "b9196e98e2b53d10f31cc599b051317f5ee77b17409f56b88a8b954c1bff201d",
            ),
            (
                0x8001,
                0x7ffe,
                111656,
                "a6de748a4665dfde14899af6f73b899692870f54258e91f6e1a686b1d11e5969",
            ),
            (
                0x8004,
                0x7ffe,
                104761,
                "eded17f1fad216547421ec268a660c29b64c3742078a221361aab61575e4b90a",
            ),
            (
                0x8004,
                0x7fff,
                149226,
                "ae28c641f8b1a0aae994049753eb5a7af8097d0bf50e10b45a803c4d9f2e9ee8",
            ),
            (
                0x8004,
                0x8000,
                155013,
                "e6e620e955189c9091bc06a4eaa9475c337ef06430555c2f27ed2ffcdad2e380",
            ),
            (
                0x8005,
                0x7fff,
                121365,
                "659f286c4c540fad017ccbe4ca13bbccc9dd42e44c7f072f3cbc80a02a90c86f",
            ),
            (
                0x8005,
                0x8000,
                117094,
                "a79001f11da8a111bc6be362f976bbbfefec52fb054e7eece92b4e6bd85acdbf",
            ),
            (
                0x8006,
                0x7fff,
                145416,
                "e7eb9f50ed0c0de9ddb39e3ee4e944dba99aad029ebc865ed9d1fc6afd4645b3",
            ),
            (
                0x8006,
                0x8000,
                127793,
                "4b51e7cebf6a81a318c459b6c8e883e335701ff007de09034401024b668f9fa9",
            ),
        ],
        508 => vec![
            (
                0x8000,
                0x8005,
                111267,
                "474ec483774f404752c3afb6fd1b4559b98d76ec5f78995eabc15d0f79732e54",
            ),
            (
                0x8000,
                0x8006,
                111114,
                "579b5ff6f39886c3ea5d800230437bf38caca436ecf869632271f88baa3f3d63",
            ),
            (
                0x8001,
                0x8005,
                108417,
                "566c5f536609a72b1372afb55bb4376e5ee1727c402600630469350d2860e18f",
            ),
            (
                0x8001,
                0x8006,
                100980,
                "7585a9b7324c2de2e24d21c6cb28fbd43ff1cd57708ff488d7741d0025764cdf",
            ),
        ],
        531 => vec![
            (
                0x7fff,
                0x7fff,
                142680,
                "2013d3f27e43eaea54bff1ad902e6cdbca8d845231dbab65525e6cf5bcb4e595",
            ),
            (
                0x7fff,
                0x8000,
                95593,
                "4e06868c74ac3f6fc394eeafc77067200e32ea3454c38a5b812d07b16431a881",
            ),
            (
                0x7fff,
                0x8001,
                55290,
                "a958c84e1693e685ebe4e01bd58d066c95a80f05093b20fbe5a568bdea4cb574",
            ),
            (
                0x8000,
                0x7fff,
                170478,
                "f51f3bf8cbb8db4ec526e15c3b63017876a67443d3d98f82b1c73b7b62f3a6bf",
            ),
            (
                0x8000,
                0x8000,
                192425,
                "b0420f5bb599e616a057f70de92b1ee5e1304890cad3c23c5501368a13f127d9",
            ),
            (
                0x8000,
                0x8001,
                135829,
                "6cecfcbd0eee4f52a2687273db7767d415f7a848b9a8cce8695d4c419c4cb09f",
            ),
            (
                0x8001,
                0x7fff,
                60861,
                "81a29f8f26188f743f3b28b49741828c0a364d22b606022d88be3acac9c444a3",
            ),
            (
                0x8001,
                0x8000,
                88398,
                "4cf0935a85bfb4439539e90fd9dc4ff87db9c464585b18cf936fbfe698c087b7",
            ),
            (
                0x8001,
                0x8001,
                99059,
                "8756d944ba21fc4be92b4886baf4f427359897a8a30a6bf75a63a247c6c4e98f",
            ),
            (
                0x8001,
                0x8002,
                49182,
                "e0d2fb2e1c4d3a8eacbea8138be84769c6ac9ac829b1e2aec4a857aa8119cc05",
            ),
        ],
        534 => vec![
            (
                0x7fff,
                0x8000,
                95700,
                "d8895da3a91286bb7fdbc31dcbcbe67eb0bb19609aefa63306f6c0c12a09d1ff",
            ),
            (
                0x7fff,
                0x8001,
                150330,
                "7c734bc35d5a48ef26451bf606a25bb839ebaee9a56af690d5929c8d051cddb4",
            ),
            (
                0x8000,
                0x8000,
                122479,
                "b7fed988b2110b0c7e10d0aaf119663a9a64d9e5f58605cd478e8aeec7896daf",
            ),
            (
                0x8000,
                0x8001,
                123355,
                "c804a186fc495d357cb4d27491e389bfb6b464d50e450788511ea512b74ee0aa",
            ),
            (
                0x8001,
                0x8000,
                140899,
                "ba04fde96af7342dd99029d6eac918c6e83d353651e1646aa054b3a26caf2231",
            ),
            (
                0x8001,
                0x8001,
                114188,
                "8c718188acae0248c4fdde32b06b8a2efd0f2ba6be366df73e6ce4d55dbf0172",
            ),
            (
                0x8002,
                0x8000,
                94725,
                "cc1c4f9c2e56cf6206bc80ad1fb7222bc359679f696d6053fffce3f08bb54326",
            ),
            (
                0x8002,
                0x8001,
                91672,
                "790c1d3f700fa7d5e33dc168d6c7505ac0d25b883b94d9eb7e6db8b2fbd4ad2b",
            ),
        ],
        551 => vec![
            (
                0x7ffe,
                0x8000,
                64405,
                "cbe68f15ddb96126a85af8b9f12cd7f9f4930f055359802ff33694a2f690ac46",
            ),
            (
                0x7ffe,
                0x8001,
                72640,
                "7a087e3409abcf19158fa1c520d4b4c146f5ebee9e9571e5b4a67bc8da9ed0ff",
            ),
            (
                0x7fff,
                0x8000,
                73346,
                "ebd29d761c6b59cfa486ca105c7abf5c3d7a5cd667f1ed598e77fa9f1e66767b",
            ),
            (
                0x7fff,
                0x8001,
                61210,
                "ec18d868ccf1ea6f5ce06b664fc787640112c28fde2c8e932fb6229906d4ff15",
            ),
        ],
        552 => vec![
            (
                0x7fff,
                0x8000,
                61833,
                "700be1ee3a388163bba1a6c29fa91f965f78d005bae2ce0797ee37a7576d25a6",
            ),
            (
                0x7fff,
                0x8001,
                61520,
                "9a169f909ee97218b7cdf345e1682cf6303edf0ee682f0a9746d0d78e75c3cb3",
            ),
        ],
        _ => panic!("not an explicit shared deathmatch candidate"),
    };
    rows.into_iter()
        .map(|(sector_x, sector_y, byte_length, sha)| SectorBinding {
            key: SharedMapSectorKey {
                map_id: id,
                sector_x,
                sector_y,
            },
            byte_length,
            sha256: digest(sha),
        })
        .collect()
}

#[test]
#[ignore = "explicit shared 551/552 S32 and Tile archives; no GUI/native-map-equivalence claim"]
fn actual_shared_551_552_geometry_directions_islands_and_tile_assets_match_java() {
    qualify_shared_maps(&[551, 552]);
}

#[test]
#[ignore = "explicit whole shared 508/531/534 S32 and Tile archives; no GUI/100-live-client claim"]
fn actual_shared_508_531_534_geometry_directions_islands_and_tile_assets_match_java() {
    qualify_shared_maps(&[508, 531, 534]);
}

#[test]
#[ignore = "whole shared516 with 20 sectors and original Tile budgets; no GUI claim"]
fn actual_shared_516_large_map_terrain_budget() {
    let root = PathBuf::from(
        std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root required"),
    );
    assert!(root.is_absolute());
    let cancelled = AtomicBool::new(false);
    let selected = bindings(516);
    assert_eq!(selected.len(), 20);
    let map = Arc::new(WorldMap::load_shared(&root, &selected, &cancelled).unwrap());
    assert_eq!(
        map.canonical_digest(),
        digest("a226e91341ea946a4ba0fec2e34b49094a1d713317d4743f41e2f19ad3111a1c")
    );
    let catalog = SharedActionCatalog::load(&root, &cancelled).unwrap();
    let assets = WorldSceneAssets::load_terrain_with_catalog(
        &catalog,
        map,
        SceneLimits::default(),
        &cancelled,
    )
    .unwrap();
    println!(
        "LARGE_SHARED_ASSETS map=516 tiles={} encoded={} texture={}",
        assets.tile_set_count(),
        assets.encoded_bytes(),
        assets.texture_bytes()
    );
    assert!(assets.tile_set_count() > 0);
    assert!(assets.encoded_bytes() <= 32 * 1024 * 1024);
    assert!(assets.texture_bytes() <= 128 * 1024 * 1024);
    assert_eq!(assets.tile_set_count(), 360);
    assert_eq!(assets.encoded_bytes(), 20_665_371);
    assert_eq!(assets.texture_bytes(), 41_780_030); // Also independently counted from original RGBA below.
}

#[test]
#[ignore = "inspection only: measured terrain costs are NOT production asset admission"]
fn inspect_shared_516_large_map_terrain_costs() {
    let root = PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit root"));
    let cancel = AtomicBool::new(false);
    let map = Arc::new(WorldMap::load_shared(&root, &bindings(516), &cancel).unwrap());
    let catalog = SharedActionCatalog::load(&root, &cancel).unwrap();
    let limits = SceneLimits {
        max_encoded_bytes: 128 * 1024 * 1024,
        max_texture_bytes: 256 * 1024 * 1024,
        ..SceneLimits::default()
    };
    let assets =
        WorldSceneAssets::load_terrain_with_catalog(&catalog, map, limits, &cancel).unwrap();
    println!(
        "INSPECTION_ONLY large516 tiles={} encoded={} texture={}",
        assets.tile_set_count(),
        assets.encoded_bytes(),
        assets.texture_bytes()
    );
}

#[test]
#[ignore = "all 360 shared516 Tile sets: sparse reads equal original full-entry pixels/offsets/roof RGB555"]
fn actual_shared_516_sparse_tiles_equal_full_original_decodes() {
    use super::{
        resources::{SharedArchive, SharedArchiveKind},
        scene::static_draw_list,
        tile::{decode_classic_til_selected, decode_roof_til_selected, TileLimits},
    };
    let root = PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit root"));
    let cancel = AtomicBool::new(false);
    let map = WorldMap::load_shared(&root, &bindings(516), &cancel).unwrap();
    let mut refs = std::collections::BTreeMap::<u16, BTreeSet<usize>>::new();
    for binding in map.bindings() {
        for draw in static_draw_list(map.sector(binding.key).unwrap()).unwrap() {
            refs.entry(draw.archive_entry_id)
                .or_default()
                .insert(draw.sub_index as usize);
        }
    }
    assert_eq!(refs.len(), 360);
    let mut archive = SharedArchive::open(&root, SharedArchiveKind::Tile).unwrap();
    let mut full_bytes = 0;
    let mut sparse_bytes = 0;
    for (id, selected) in refs {
        let name = format!("{id}.til");
        let original = archive.read_entry(&name).unwrap();
        full_bytes += original.len();
        let sparse = super::tile_reader::load(
            &mut archive,
            &name,
            &selected,
            32 * 1024 * 1024 - sparse_bytes,
            &cancel,
        )
        .unwrap();
        sparse_bytes += sparse.read_bytes;
        let expected =
            decode_classic_til_selected(&original, TileLimits::default(), &selected).unwrap();
        let actual =
            decode_classic_til_selected(&sparse.data, TileLimits::default(), &selected).unwrap();
        assert!(
            expected == actual,
            "Tile/{name} selected pixel/offset mismatch"
        );
        let expected =
            decode_roof_til_selected(&original, TileLimits::default(), &selected).unwrap();
        let actual =
            decode_roof_til_selected(&sparse.data, TileLimits::default(), &selected).unwrap();
        assert!(expected == actual, "Tile/{name} raw roof mismatch");
    }
    assert_eq!((full_bytes, sparse_bytes), (38_278_397, 20_665_371));
}

#[test]
#[ignore = "inspection only: full 504/505/506 under unchanged production budgets; no gameplay admission"]
fn inspect_shared_504_505_506_whole_terrain_costs() {
    let root = PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit root"));
    let cancel = AtomicBool::new(false);
    let catalog = SharedActionCatalog::load(&root, &cancel).unwrap();
    for id in [504, 505, 506] {
        let selected = bindings(id);
        let map = Arc::new(WorldMap::load_shared(&root, &selected, &cancel).unwrap());
        println!(
            "INSPECTION_ONLY map={id} sectors={} canonical={}",
            selected.len(),
            map.canonical_digest()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        );
        match WorldSceneAssets::load_terrain_with_catalog(
            &catalog,
            map,
            SceneLimits::default(),
            &cancel,
        ) {
            Ok(assets) => println!(
                "INSPECTION_ONLY map={id} tiles={} encoded={} texture={}",
                assets.tile_set_count(),
                assets.encoded_bytes(),
                assets.texture_bytes()
            ),
            Err(error) => println!("INSPECTION_ONLY map={id} rejected={error:#}"),
        }
    }
}

#[test]
#[ignore = "inspection only: counts exact RGBA reuse, not renderer or budget admission"]
fn inspect_shared_large_tile_pixel_reuse() {
    use super::{
        resources::{SharedArchive, SharedArchiveKind},
        scene::static_draw_list,
        tile::{decode_classic_til_selected, TileLimits},
    };
    let root = PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit root"));
    let cancel = AtomicBool::new(false);
    for id in [504, 505, 506, 516] {
        let map = WorldMap::load_shared(&root, &bindings(id), &cancel).unwrap();
        let mut refs = std::collections::BTreeMap::<u16, BTreeSet<usize>>::new();
        for b in map.bindings() {
            for d in static_draw_list(map.sector(b.key).unwrap()).unwrap() {
                refs.entry(d.archive_entry_id)
                    .or_default()
                    .insert(d.sub_index as usize);
            }
        }
        let mut archive = SharedArchive::open(&root, SharedArchiveKind::Tile).unwrap();
        let mut hashes = BTreeSet::<[u8; 32]>::new();
        let mut count = 0usize;
        let mut encoded = 0usize;
        for (&set, selected) in &refs {
            let bytes = archive.read_entry(&format!("{set}.til")).unwrap();
            encoded += bytes.len();
            let decoded =
                decode_classic_til_selected(&bytes, TileLimits::default(), selected).unwrap();
            count += decoded.len();
            for tile in decoded.values() {
                hashes.insert(Sha256::digest(&tile.rgba).into());
            }
        }
        println!("INSPECTION_ONLY map={id} sets={} references={count} fullEncoded={encoded} flatTexture={} uniqueRgba={} hypotheticalSharedTexture={}",refs.len(),count*2376,hashes.len(),hashes.len()*2376);
    }
}

#[test]
#[ignore = "whole shared 504, independent Java geometry and original production asset limits; no GUI"]
fn actual_shared_504_whole_geometry_and_terrain_assets() {
    qualify_shared_maps(&[504]);
}

#[test]
#[ignore = "whole shared 505, independent Java geometry and original production asset limits; no GUI"]
fn actual_shared_505_whole_geometry_and_terrain_assets() {
    qualify_shared_maps(&[505]);
}

#[test]
#[ignore = "whole shared 506, independent Java geometry and original production asset limits; no GUI"]
fn actual_shared_506_whole_geometry_and_terrain_assets() {
    qualify_shared_maps(&[506]);
}

#[test]
#[ignore = "all referenced tiles on four complete large maps, original full-entry decode, independent byte accounting"]
fn actual_shared_large_compact_cache_equals_original_pixels_and_budget() {
    use super::{
        resources::{SharedArchive, SharedArchiveKind},
        scene::static_draw_list,
        tile::{decode_classic_til_selected, decode_roof_til_selected, TileLimits},
    };
    fn expected_bytes(rgba: &[u8]) -> usize {
        assert_eq!(rgba.len(), 2304);
        let encoded: usize = rgba
            .chunks_exact(96)
            .map(|row| {
                let occupied: Vec<_> = row
                    .chunks_exact(4)
                    .enumerate()
                    .filter_map(|(x, p)| (p != [0, 0, 0, 0]).then_some(x))
                    .collect();
                match (occupied.first(), occupied.last()) {
                    (Some(first), Some(last)) => 3 + (last - first + 1) * 4,
                    _ => 0,
                }
            })
            .sum();
        encoded.min(2304) + 72
    }
    let root = PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit root"));
    let cancel = AtomicBool::new(false);
    let catalog = SharedActionCatalog::load(&root, &cancel).unwrap();
    for id in [504, 505, 506, 516] {
        let map = Arc::new(WorldMap::load_shared(&root, &bindings(id), &cancel).unwrap());
        let mut refs = std::collections::BTreeMap::<u16, BTreeSet<usize>>::new();
        for b in map.bindings() {
            for d in static_draw_list(map.sector(b.key).unwrap()).unwrap() {
                refs.entry(d.archive_entry_id)
                    .or_default()
                    .insert(d.sub_index as usize);
            }
        }
        let assets = WorldSceneAssets::load_terrain_with_catalog(
            &catalog,
            map,
            SceneLimits::default(),
            &cancel,
        )
        .unwrap();
        let mut archive = SharedArchive::open(&root, SharedArchiveKind::Tile).unwrap();
        let mut expected = 0usize;
        let mut count = 0usize;
        let mut roofs = 0usize;
        for (&set, selected) in &refs {
            let bytes = archive.read_entry(&format!("{set}.til")).unwrap();
            let original =
                decode_classic_til_selected(&bytes, TileLimits::default(), selected).unwrap();
            let raw = decode_roof_til_selected(&bytes, TileLimits::default(), selected).unwrap();
            for (&sub, tile) in &original {
                let (cached, roof) = assets.decoded_tile_for_test(set, sub);
                assert!(
                    cached == *tile,
                    "map={id} Tile/{set}:{sub} normal pixels/metadata changed"
                );
                expected += expected_bytes(&tile.rgba);
                count += 1;
                if let Some(roof) = roof {
                    assert!(
                        roof == raw[&sub].rgba,
                        "map={id} Tile/{set}:{sub} raw roof changed"
                    );
                    expected += expected_bytes(&roof);
                    roofs += 1;
                }
            }
        }
        assert_eq!(assets.tile_set_count(), refs.len());
        assert_eq!(assets.texture_bytes(), expected);
        assert!(assets.encoded_bytes() <= 32 * 1024 * 1024);
        assert!(expected <= 128 * 1024 * 1024);
        println!("EXACT_COMPACT map={id} tilesets={} tiles={count} rawRoofs={roofs} encoded={} retained={expected}",refs.len(),assets.encoded_bytes());
    }
}

fn qualify_shared_maps(ids: &[u32]) {
    let root = PathBuf::from(
        std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root required"),
    );
    assert!(root.is_absolute());
    let cancelled = AtomicBool::new(false);
    let catalog = SharedActionCatalog::load(&root, &cancelled).unwrap();
    for &id in ids {
        let bindings = bindings(id);
        let map = Arc::new(WorldMap::load_shared(&root, &bindings, &cancelled).unwrap());
        let (canonical, directions, count, sizes, anchor, capacity) = match id {
            504 => (
                "ae157073502ac84560f17cee7d60546e86151a9189f139dacfc52dae9ab4cb73",
                "6d45817493a489c8f157ba0e75fc520396657847c0ceccf4a61221c52970ae80",
                29456,
                vec![
                    9596, 6971, 4052, 2486, 2240, 1012, 874, 687, 687, 414, 162, 131, 112, 12, 4,
                    3, 3, 2, 2, 2, 2, 2,
                ],
                WorldCell { x: 32933, y: 32862 },
                100,
            ),
            505 => (
                "44867dd7fe08a479ae5fc0de7a951a157b6336d5493e40aa7d423beeedc4ad0f",
                "41de4510d162f60ca1eecc52b0d5e2b9c144f1b77b4849180f8e248a29c21e64",
                30455,
                vec![
                    9596, 7173, 6971, 2486, 2240, 1012, 687, 162, 112, 4, 3, 3, 2, 2, 2,
                ],
                WorldCell { x: 32933, y: 32862 },
                100,
            ),
            506 => (
                "8b072b684306f964c5f8692c5c2f8ec4879271a5fdd51e048cb4b144ac8de49a",
                "813f5fcb62823a86b12a5ce0107e2bae5ee94a579f50ade9e40a316b115e5121",
                27000,
                vec![
                    9596, 6971, 2486, 2221, 1794, 1048, 1012, 874, 687, 162, 112, 12, 9, 4, 3, 3,
                    2, 2, 2,
                ],
                WorldCell { x: 32933, y: 32862 },
                100,
            ),
            551 => (
                "4f15ca8c023ab4c4d2d33a305c3caf1180f916ea9a8c03a2fe74d6791b220fa5",
                "c48f16a15b2dd1f5839392d0bdfeec69d278614d145a66ea5f9c8956de5bc63d",
                1057,
                vec![183, 179, 175, 175, 175, 170],
                WorldCell { x: 32729, y: 32790 },
                50,
            ),
            552 => (
                "70f75808b445d432efdbb0494c2c2d87355ddd788c0d4a75f1c45b113934f0c4",
                "e91bc4e503588f601ed4e9c6f9287ecb263caa0fa3b60416a2a95fb27de86122",
                341,
                vec![171, 170],
                WorldCell { x: 32736, y: 32858 },
                50,
            ),
            508 => (
                "d2970a8ea9c057fc0be096874866b230f80dfbd984d2abf265aa358eaf3ef392",
                "785c9b8f503d5194626947f1f7bc6223dc5d72fe4a342a4ab685a25296239856",
                8097,
                vec![8097],
                WorldCell { x: 32832, y: 33151 },
                100,
            ),
            531 => (
                "f4f2b42676cb98cd9b1c95df19c03f7f5c3c69bc854e644d0f6f318bbdb728bf",
                "1d67671c0671c6bac78dfcf44b00a389c65fa19160d861f5cd4ad744ab82807e",
                7001,
                vec![6999, 2],
                WorldCell { x: 32799, y: 32797 },
                100,
            ),
            534 => (
                "314fd5c5b78d996e8bf2b6c3135bb63a29cb78d80c04fc96f77fd2a5bfbad2f4",
                "8320a953597b90f5aa72bcf10e2bdbf6bc57ac5bcef9b89f83e4bead5f6d7daf",
                6364,
                vec![6364],
                WorldCell { x: 32817, y: 32832 },
                100,
            ),
            _ => panic!("unselected shared map"),
        };
        assert_eq!(map.canonical_digest(), digest(canonical));
        let mut masks = Vec::new();
        let mut standing = BTreeSet::new();
        for b in &bindings {
            let (ox, oy) = b.key.world_origin();
            for y in oy..oy + 64 {
                for x in ox..ox + 64 {
                    let p = WorldCell { x, y };
                    masks.push(map.direction_mask(p));
                    if map.standing(p) {
                        standing.insert(p);
                    }
                }
            }
        }
        assert_eq!(standing.len(), count);
        assert_eq!(<[u8; 32]>::from(Sha256::digest(&masks)), digest(directions));
        let mut components = Vec::new();
        let mut arena = None;
        while let Some(start) = standing.first().copied() {
            let mut seen = BTreeSet::from([start]);
            let mut queue = VecDeque::from([start]);
            standing.remove(&start);
            while let Some(p) = queue.pop_front() {
                for (dx, dy) in DIRECTIONS {
                    let q = WorldCell {
                        x: p.x + dx,
                        y: p.y + dy,
                    };
                    if map.can_step(p, q) && map.can_step(q, p) && seen.insert(q) {
                        standing.remove(&q);
                        queue.push_back(q);
                    }
                }
                assert!(seen.len() <= bindings.len() * 4096);
            }
            components.push(seen.len());
            if seen.contains(&anchor) {
                arena = Some(seen);
            }
        }
        components.sort_unstable_by(|a, b| b.cmp(a));
        assert_eq!(components, sizes);
        let arena = arena.unwrap();
        assert_eq!(arena.len(), sizes[0]);
        map.validate_spawns(
            &arena.iter().take(capacity).copied().collect::<Vec<_>>(),
            &cancelled,
        )
        .unwrap();
        // Occupied-roster egress is independently tested on the authoritative Java side.
        let assets = WorldSceneAssets::load_terrain_with_catalog(
            &catalog,
            map,
            SceneLimits::default(),
            &cancelled,
        )
        .unwrap();
        assert!(assets.tile_set_count() > 0);
        assert!(assets.encoded_bytes() > 0);
        assert!(assets.texture_bytes() > 0);
        println!("SHARED_DM_ASSETS map={id} tiles={} encoded={} texture={} connected={} native_equivalence=false",
            assets.tile_set_count(),assets.encoded_bytes(),assets.texture_bytes(),arena.len());
    }
}
