use bdk_wallet::{
    bitcoin::{
        bip32::Xpriv,
        key::rand::{self, RngCore},
        Network,
    },
    template::{Bip86, DescriptorTemplate},
    KeychainKind,
};

const NETWORK: Network = Network::Signet;

fn main() {
    // Create your initial seed value
    let mut seed: [u8; 32] = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut seed);

    // Create a new master key from a seed value
    let xprv: Xpriv = Xpriv::new_master(NETWORK, &seed).unwrap();

    // Create both receive descriptor (external) and  change descriptor (internal) from the master key
    let (descriptor, keymap, _) = Bip86(xprv, KeychainKind::External).build(NETWORK).unwrap();
    let (change_descriptor, change_keymap, _) =
        Bip86(xprv, KeychainKind::Internal).build(NETWORK).unwrap();

    println!("\n!!!! PLEASE CREATE A BACKUP OF YOUR DESCRIPTORS BELOW !!!!\n");
    println!(
        "EXTERNAL DESCRIPTOR: {:?}\n",
        descriptor.to_string_with_secret(&keymap)
    );
    println!(
        "INTERNAL DESCRIPTOR: {:?}",
        change_descriptor.to_string_with_secret(&change_keymap)
    );
    println!("\n!!!! PLEASE CREATE A BACKUP OF YOUR DESCRIPTORS ABOVE !!!!\n");
}
