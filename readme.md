# Hangman over QUIC

This is an implementation of the hangman game over QUIC, introduced in [this blog
post](https://ochagavia.nl/blog/hangman-over-quic/).

Feel free to contribute with alternative clients (different programming languages, UIs, etc).

## How to run

* Server: `cd server && cargo run --release`
* Client: `cd client && cargo run --release`

By default, the server will bind to `127.0.0.1:8080`, and the client will try to connect to that
address.
