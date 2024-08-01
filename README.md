# AESteve: An AES Encryption Library in Rust

AESteve is a Rust library for AES encryption and decryption. It provides a straightforward API for securely encrypting and decrypting messages using the AES algorithm. The library supports 128-bit keys and leverages parallel processing for efficient block handling.

## Features

- AES encryption and decryption with 128-bit keys
- Efficient block processing with parallelization using Rayon
- Error handling for common issues (e.g., invalid key length, invalid Base64 sequences)

## Installation

To use AESteve in your Rust project, add it as a dependency in your `Cargo.toml` file:

```toml
[dependencies]
aesteve = { path = "/path/to/your/aesteve" }
```

## Usage

Here's a simple example demonstrating how to use AESteve to encrypt and decrypt a message:

```rust
use aesteve::{AESteve, AESError};

fn main() -> Result<(), AESError> {
    // Create a new AES instance with a 128-bit key
    let key = b"thisisaverysecurekey!";
    let aes = AESteve::new(key)?;

    // Encrypt a message
    let message = "This is a test!";
    let encrypted_message = aes.encrypt(message.to_string())?;
    println!("Encrypted message: {}", encrypted_message);

    // Decrypt the message
    let decrypted_message = aes.decrypt(encrypted_message)?;
    println!("Decrypted message: {}", decrypted_message);

    Ok(())
}
```

## API

### AESteve

#### `pub fn new(key: &[u8]) -> Result<Self, AESError>`

Creates a new AES instance with the given 128-bit key. Returns an error if the key length is not 16 bytes.

#### `pub fn encrypt(&self, message: String) -> Result<String, AESError>`

Encrypts the given message. Returns the encrypted message in Base64 format.

#### `pub fn decrypt(&self, encrypted_message: String) -> Result<String, AESError>`

Decrypts the given encrypted message (in Base64 format). Returns the decrypted message.

## Error Handling

AESteve defines an `AESError` enum to handle various errors that might occur during encryption and decryption:

- `InvalidBase64`: Indicates an invalid Base64 sequence.
- `InvalidUTF8`: Indicates an invalid UTF-8 sequence.
- `InvalidKeyLength`: Indicates an invalid key length.

## Testing

To run the tests for AESteve, use the following command:

```bash
cargo test
```

## Contributing

Contributions are welcome! If you have any suggestions or improvements, please open an issue or submit a pull request.

## License

This project is licensed under the MIT License.

## Project Structure

```
aesteve/
├── Cargo.toml
└── src
    └── lib.rs
```
