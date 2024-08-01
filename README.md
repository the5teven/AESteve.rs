# AESteve: An AES Encryption Library in Rust

AESteve is a Rust library for AES encryption and decryption. It provides a straightforward API for securely encrypting and decrypting messages using the AES algorithm. The library supports 128-bit keys and leverages parallel processing for efficient block handling.

## Features

- AES encryption and decryption with 128-bit keys
- Efficient block processing with parallelization using Rayon
- Error handling for common issues (e.g., invalid key length, invalid Base64 sequences)
- 
## Installation

To use AESteve in your Rust project, add it as a dependency in your `Cargo.toml` file:

```toml
[dependencies]
aesteve = { path = "/path/to/your/aesteve" }
