# meshcore_companion_rs

A Rust library for communicating with MeshCore companion devices over serial (bluetooth coming eventually)

## Current Features
Curious about the current state of feature-completeness?  Check out the [Project Board](https://github.com/users/PeterGrace/projects/1/views/1)

- Serial communication with mesh radio devices
- Contact management and synchronization
- Send and receive direct messages and channel messages
- Device query and status monitoring
- Async/await support with Tokio

## Usage

See examples folder for concise examples of major functionality.  For example, `cargo run --example send_channel_message`.

## Building

```bash
cargo build --release
```

## Running the test application

```bash
cargo run --bin companion_test
```

## License

See LICENSE file for details.
