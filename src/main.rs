use std::time::UNIX_EPOCH;

use bdk_esplora::{
    esplora_client::{self},
    EsploraExt,
};
use bdk_wallet::{
    bitcoin::{
        bip32::Xpriv,
        key::rand::{self, RngCore},
        Network,
    },
    chain::Persisted,
    rusqlite::Connection,
    template::{Bip86, DescriptorTemplate},
    AddressInfo, KeychainKind, Wallet,
};

const NETWORK: Network = Network::Signet;

const EXTERNAL_DESCRIPTOR: &str = "tr(tprv8ZgxMBicQKsPf6KHfH1XnfAiNnVMszztDXmCwjXJeWMno3o7HLbP4TdFiduhZ5QxY6nxjZ4XHxr3tr1oxo3K917N5ETB3qvuJc6pW3P367p/86'/1'/0'/0/*)#tmhwtshy";
const INTERNAL_DESCRIPTOR: &str = "tr(tprv8ZgxMBicQKsPf6KHfH1XnfAiNnVMszztDXmCwjXJeWMno3o7HLbP4TdFiduhZ5QxY6nxjZ4XHxr3tr1oxo3K917N5ETB3qvuJc6pW3P367p/86'/1'/0'/1/*)#60j0k98u";

const ESPLORA_URL: &str = "http://mutinynet.com/api";
const STOP_GAP: usize = 5;
const PARALLEL_REQUESTS: usize = 5;

const DB_PATH: &str = "database.db3";

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

    // Start a new database connection with the given SQLITE DB file
    let mut db = Connection::open(DB_PATH).unwrap();

    // Create BDK Wallet from both (receive) external descriptor and (change) internal descriptor
    let mut wallet: Persisted<Wallet> = Wallet::create(external_descriptor, internal_descriptor)
        .network(NETWORK)
        .create_wallet(&mut db)
        .unwrap();

    // Reveal an address from (receive) external keychain
    let receiving_address: AddressInfo = wallet.next_unused_address(KeychainKind::External);
    wallet.persist(&mut db).unwrap();

    // Reveal an address from (change) internal keychain
    let change_address: AddressInfo = wallet.next_unused_address(KeychainKind::Internal);
    wallet.persist(&mut db).unwrap();

    println!(
        "REVEALED RECEIVE (EXTERNAL) ADDRESS {} @ INDEX {}",
        receiving_address.address, receiving_address.index
    );

    println!(
        "REVEALED CHANGE (INTERNAL) ADDRESS {} @ INDEX {}",
        change_address.address, change_address.index
    );

    println!(
        "WALLET BALANCE (BEFORE FULL SCAN): {}",
        wallet.balance().total().to_btc()
    );

    perform_full_scan(&mut wallet, &mut db);

    println!(
        "WALLET BALANCE (AFTER FULL SCAN): {}",
        wallet.balance().total().to_btc()
    );

    println!(
        "WALLET BALANCE (BEFORE PARTIAL SYNC): {}",
        wallet.balance().total().to_btc()
    );

    perform_sync(&mut wallet, &mut db);

    println!(
        "WALLET BALANCE (AFTER PARTIAL SYNC): {}",
        wallet.balance().total().to_btc()
    );

    // Closing the database connection
    let _ = db.close();
}

fn perform_sync(wallet: &mut Persisted<Wallet>, db: &mut Connection) {
    let blocking_client = esplora_client::Builder::new(ESPLORA_URL).build_blocking();

    let request = wallet.start_sync_with_revealed_spks();
    let mut update = blocking_client
        .sync(request, PARALLEL_REQUESTS)
        .expect("Failed to perform full scan");

    let now = UNIX_EPOCH
        .elapsed()
        .expect("Failed to get current time")
        .as_secs();

    let _changeset = update.graph_update.update_last_seen_unconfirmed(now);
    wallet.apply_update(update).expect("Failed to apply update");
    wallet.persist(db).unwrap();
}

fn perform_full_scan(wallet: &mut Persisted<Wallet>, db: &mut Connection) {
    let blocking_client = esplora_client::Builder::new(ESPLORA_URL).build_blocking();

    let request = wallet.start_full_scan();
    let mut update = blocking_client
        .full_scan(request, STOP_GAP, PARALLEL_REQUESTS)
        .expect("Failed to perform full scan");

    let now = UNIX_EPOCH
        .elapsed()
        .expect("Failed to get current time")
        .as_secs();

    let _changeset = update.graph_update.update_last_seen_unconfirmed(now);
    wallet.apply_update(update).expect("Failed to apply update");
    wallet.persist(db).unwrap();
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
