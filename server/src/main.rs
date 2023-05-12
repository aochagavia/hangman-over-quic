mod game_state;
mod word_picker;

use std::net::Ipv4Addr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use quinn::{Connecting, Connection, Endpoint};
use rustls::{Certificate, PrivateKey};

use game_state::GameState;
use word_picker::WordPicker;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_server().await
}

async fn run_server() -> anyhow::Result<()> {
    let endpoint = setup_endpoint();

    println!("Listening on port {}", endpoint.local_addr()?.port());

    // Global server state
    let mut word_picker = WordPicker::new();
    let active_players = Arc::new(AtomicU32::new(0));

    // Accept connections from new players
    while let Some(conn) = endpoint.accept().await {
        let active_players = active_players.clone();
        let word = word_picker.pick_random_word();

        // A connection's logic is run in its own task
        tokio::spawn(async move {
            active_players.fetch_add(1, Ordering::Relaxed);

            handle_connection(conn, word, active_players.clone())
                .await
                .ok(); // We don't care about connection errors

            active_players.fetch_sub(1, Ordering::Relaxed);
            println!("Connection closed");
        });
    }

    Ok(())
}

async fn handle_connection(
    conn: Connecting,
    word: Vec<u8>,
    active_players: Arc<AtomicU32>,
) -> anyhow::Result<()> {
    let connection = conn.await?;
    println!("connection established");

    let mut state = GameState::new(word);

    // Send the word length
    let mut word_length_stream = connection.open_uni().await?;
    word_length_stream.write_all(&[state.word_len()]).await?;
    word_length_stream.finish().await?;

    // In a separate task, regularly send the active players to the client
    tokio::spawn(send_active_players(connection.clone(), active_players));

    while state.is_running() {
        // Each stream initiated by the client represents a new guess
        let (mut send, mut recv) = connection.accept_bi().await?;

        // A guess consists of a single byte
        let mut read_buf = [0; 1];
        recv.read_exact(&mut read_buf).await?;

        let indexes = state.handle_guess(read_buf[0]);

        // The guess result is a list of bytes, preceded by the length of the list
        let mut write_buf = Vec::new();
        write_buf.push(indexes.len() as u8);
        write_buf.extend(indexes);
        send.write_all(&write_buf).await?;

        // We won't need the stream any longer
        send.finish().await?;
    }

    // The game is no longer running, and the connection will be closed on drop
    Ok(())
}

async fn send_active_players(
    connection: Connection,
    active_players: Arc<AtomicU32>,
) -> anyhow::Result<()> {
    // We use a unidirectional stream
    let mut stream = connection.open_uni().await?;

    loop {
        // Send an update once per second
        tokio::time::sleep(Duration::from_secs(1)).await;
        let bytes = active_players.load(Ordering::Relaxed).to_le_bytes();
        stream.write_all(&bytes).await?;
    }
}

fn setup_endpoint() -> Endpoint {
    let (cert, key) = gen_cert();
    let mut server_crypto = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)
        .expect("failed to build rustls config");

    // The name of our custom protocol, needed because QUIC automatically negotiates the protocol to
    // be used when a client attempts to connect
    server_crypto.alpn_protocols = vec![b"hangman-over-quic".to_vec()];

    let mut server_config = quinn::ServerConfig::with_crypto(Arc::new(server_crypto));

    // Forbid the client from creating unidirectional streams (only the server is allowed to)
    Arc::get_mut(&mut server_config.transport)
        .unwrap()
        .max_concurrent_uni_streams(0u8.into());

    Endpoint::server(server_config, (Ipv4Addr::new(127, 0, 0, 1), 8080).into())
        .expect("failed to create server endpoint")
}

// We use a self-signed certificate generated for the occasion
fn gen_cert() -> (Certificate, PrivateKey) {
    let cert = rcgen::generate_simple_self_signed(vec!["dummy-cert-alt-name".into()]).unwrap();
    let key = cert.serialize_private_key_der();
    let cert = cert.serialize_der().unwrap();
    let key = PrivateKey(key);
    let cert = Certificate(cert);
    (cert, key)
}
