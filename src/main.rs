use bdk_wallet::{
    bitcoin::{
        bip32::Xpriv,
        key::rand::{self, RngCore},
        Network,
    },
    template::{Bip86, DescriptorTemplate},
    AddressInfo, KeychainKind, Wallet,
};

const NETWORK: Network = Network::Signet;

const EXTERNAL_DESCRIPTOR: &str = "";
const INTERNAL_DESCRIPTOR: &str = "";

fn main() {
    let (external_descriptor, internal_descriptor) =
        if EXTERNAL_DESCRIPTOR.is_empty() || INTERNAL_DESCRIPTOR.is_empty() {
            create_descriptors()
        } else {
            println!("\n!!!! PLEASE CREATE A BACKUP OF YOUR DESCRIPTORS BELOW !!!!\n");
            println!("EXTERNAL DESCRIPTOR: {:?}\n", EXTERNAL_DESCRIPTOR);
            println!("INTERNAL DESCRIPTOR: {:?}", INTERNAL_DESCRIPTOR);
            println!("\n!!!! PLEASE CREATE A BACKUP OF YOUR DESCRIPTORS ABOVE !!!!\n");

            (
                EXTERNAL_DESCRIPTOR.to_string(),
                INTERNAL_DESCRIPTOR.to_string(),
            )
        };

    // Create BDK Wallet from both (receive) external descriptor and (change) internal descriptor
    let mut wallet: Wallet = Wallet::create(external_descriptor, internal_descriptor)
        .network(NETWORK)
        .create_wallet_no_persist()
        .unwrap();

    // Reveal an address from (receive) external keychain
    let receiving_address: AddressInfo = wallet.reveal_next_address(KeychainKind::External);

    // Reveal an address from (change) internal keychain
    let change_address: AddressInfo = wallet.reveal_next_address(KeychainKind::Internal);

    println!(
        "REVEALED RECEIVE (EXTERNAL) ADDRESS {} @ INDEX {}",
        receiving_address.address, receiving_address.index
    );

    println!(
        "REVEALED CHANGE (INTERNAL) ADDRESS {} @ INDEX {}",
        change_address.address, change_address.index
    );
}

fn create_descriptors() -> (String, String) {
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

    (
        descriptor.to_string_with_secret(&keymap),
        change_descriptor.to_string_with_secret(&change_keymap),
    )
}
