mod game_state;

use std::io::Write;
use std::net::Ipv4Addr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::SystemTime;

use anyhow::Context;
use quinn::{Connection, Endpoint};
use rustls::client::ServerCertVerified;
use rustls::{Certificate, Error, ServerName};

use game_state::GameState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let endpoint = setup_endpoint();

    let remote_addr = std::env::args()
        .nth(1)
        .unwrap_or("127.0.0.1:8080".to_string());
    let remote_addr = remote_addr
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid server address"))?;

    // The server is using a self-signed certificate under this name
    let server_cert_name = "localhost";

    // Connect
    let connection = endpoint
        .connect(remote_addr, server_cert_name)?
        .await
        .context("Error connecting to host")?;

    // Play!
    play(connection).await?;

    Ok(())
}

async fn play(connection: Connection) -> anyhow::Result<()> {
    // Global state
    let active_players = Arc::new(AtomicU32::new(1));

    // Receive the word length, sent in the first unidirectional stream
    let mut word_length_stream = connection.accept_uni().await?;
    let mut word_len_buf = [0; 1];
    word_length_stream.read_exact(&mut word_len_buf).await?;
    let word_len = word_len_buf[0];

    // Receive active players in a background task, sent in the second unidirectional stream
    tokio::spawn(receive_active_players(
        connection.clone(),
        active_players.clone(),
    ));

    // The game loop, driven by the terminal
    println!("Welcome to Hangman over QUIC!");

    let mut game_state = GameState::with_word_len(word_len);
    let mut stdout = std::io::stdout();
    let stdin = std::io::stdin();
    while game_state.is_running() {
        println!("Active players: {}", active_players.load(Ordering::Relaxed));
        println!("Word: {}", game_state.display_word());
        println!("Lives left: {}", game_state.lives_left);
        println!("Previous guesses: {}", game_state.all_guesses);
        print!("Next guess: ");
        stdout.flush()?;

        let mut line = String::new();
        stdin.read_line(&mut line)?;
        let line = line.trim();
        if line.bytes().len() == 1 && line.as_bytes()[0].is_ascii() {
            println!("\n...");

            // We send the guess and retrieve the result in a dedicated bidirectional stream, opened
            // for the occasion
            let (mut send, mut recv) = connection.open_bi().await?;

            // Send the guess
            let guessed_char = line.as_bytes()[0];
            send.write_all(&[guessed_char]).await?;

            // Retrieve the indexes corresponding to the guess
            let mut response_len_buf = [0; 1];
            recv.read_exact(&mut response_len_buf).await?;
            let response_len = response_len_buf[0];

            let mut response_indexes = vec![0; response_len as usize];
            if response_len > 0 {
                recv.read_exact(&mut response_indexes).await?;
            }

            // Handle the result
            let correct_guess = game_state.handle_guess_result(guessed_char, &response_indexes);
            if correct_guess {
                println!("You guessed correctly!");
            } else {
                println!("You guessed incorrectly!");
            }
        } else {
            println!("Invalid character (you must enter a single ASCII character)");
        }

        println!();
    }

    if game_state.player_won() {
        println!(
            "Congratulations, you won! The word was {}",
            game_state.display_word()
        );
    } else {
        println!("Alas, you lost! Better luck next time...")
    }

    Ok(())
}

async fn receive_active_players(
    connection: Connection,
    active_players: Arc<AtomicU32>,
) -> anyhow::Result<()> {
    let mut stream = connection.accept_uni().await?;

    let mut buf = [0; 4];
    loop {
        match stream.read_exact(&mut buf).await {
            Ok(()) => {
                active_players.store(u32::from_le_bytes(buf), Ordering::Relaxed);
            }
            Err(e) => {
                println!("Oops, something went wrong! {e:?}");
                return Ok(());
            }
        }
    }
}

fn setup_endpoint() -> Endpoint {
    let mut client_crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();

    client_crypto.alpn_protocols = vec![b"hangman-over-quic".to_vec()];
    let mut endpoint =
        Endpoint::client((Ipv4Addr::new(0, 0, 0, 0), 0).into()).expect("failed to create client");
    endpoint.set_default_client_config(quinn::ClientConfig::new(Arc::new(client_crypto)));
    endpoint
}

// Note: the struct and impl below disable verification for the server's certificate, which you
// should never do on production, because it opens the door to man-in-the-middle attacks. Why do we
// use it here then? Because this is just example code, and it makes it easier for people to run it
// in their own machines.
struct SkipServerVerification;

impl SkipServerVerification {
    fn new() -> Arc<SkipServerVerification> {
        Arc::new(Self)
    }
}

impl rustls::client::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: SystemTime,
    ) -> Result<ServerCertVerified, Error> {
        Ok(ServerCertVerified::assertion())
    }
}
