# Solana UI - Blockchain Network Monitor

A modern desktop GUI application for monitoring the Solana blockchain network, built with Rust and egui. This tool provides real-time insights into validators, gossip nodes, vote activity, and leader schedules.

## Features

### 🔍 **Validators Tab**
- View all active validators on the network
- Sort by multiple columns (identity, vote account, commission, stake, etc.)
- Filter validators by identity and vote account addresses
- Real-time data including last vote, root slot, and skip rates
- Activated stake amounts displayed in SOL

### 🌐 **Gossip Nodes Tab**  
- Monitor gossip network nodes and their endpoints
- View TPU, RPC, and QUIC protocol addresses
- Filter nodes by identity pubkey
- Network version and feature set information

### 🗳️ **Find Voters Tab**
- Search for validators that voted in a specific slot
- Filter results by vote account address
- View detailed voting information and vote account signatures

### 📅 **Leader Schedule Tab**
- Fetch leader schedule for any validator identity
- Specify epoch or use current epoch
- View assigned leader slots for validators

### 📋 **Logs Tab**
- Real-time RPC request/response logging
- Color-coded log entries (requests, responses, errors)
- View detailed API calls and their responses
- Automatic log rotation (keeps last 1000 entries)

## Installation

### Prerequisites
- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- Git

### Build from Source
```bash
git clone <repository-url>
cd find_voters_in_slot
cargo build --release
```

### Run
```bash
cargo run --release
```

## Usage

### Network Selection
- Use the cluster dropdown in the top-right to switch between **Testnet** and **Mainnet**
- All API calls will automatically use the selected network endpoint

### Data Refresh
- Click the **Refresh** button to update validator and gossip node data
- Status messages show current loading state
- Data is cached locally for better performance

### Search and Filtering
- Each tab includes search functionality for filtering results
- Use the **Clear** buttons to reset search filters
- Search is case-insensitive and supports partial matching

### Multi-Column Sorting
- Click column headers to sort data
- Hold **Shift** while clicking to add secondary sort columns
- Up to 3 columns can be sorted simultaneously
- Sort direction toggles between ascending/descending

### Configuration Persistence
- Search filters, selected cluster, and window settings are automatically saved
- Configuration stored in `~/.config/solana-ui/config.json` (Linux/macOS) or equivalent Windows location

## Architecture

The application is built with a modular architecture:

```
src/
├── main.rs           # Application entry point
├── ui.rs             # Main UI orchestration and state management
├── solana.rs         # Solana RPC client and data fetching
├── utils.rs          # Utility functions and status management  
├── config/           # Configuration persistence
│   └── mod.rs
└── tabs/             # Individual tab implementations
    ├── mod.rs
    ├── validators.rs
    ├── gossip_nodes.rs
    ├── find_voters.rs
    ├── leader_schedule.rs
    └── logs.rs
```

### Key Components

- **SolanaClient**: Handles all RPC communication with Solana nodes
- **ConfigManager**: Persists user preferences between sessions
- **StatusManager**: Manages loading states and timeout handling
- **LogStore**: Thread-safe logging system for RPC activity

## RPC Endpoints

The application connects to official Solana RPC endpoints:

- **Testnet**: `https://api.testnet.solana.com`
- **Mainnet**: `https://api.mainnet-beta.solana.com`

## Dependencies

- **eframe/egui**: Cross-platform GUI framework
- **solana-client**: Official Solana RPC client
- **tokio**: Async runtime for non-blocking API calls
- **serde**: Configuration serialization
- **chrono**: Timestamp handling for logs

## Development

### Code Style
- Follows standard Rust conventions
- Uses `cargo clippy` for linting
- Comprehensive error handling with `anyhow`
- Async/await patterns for non-blocking UI

### Testing
```bash
cargo test
```

### Linting
```bash
cargo clippy
```

## Performance

- Efficient async data fetching prevents UI blocking
- Smart caching reduces redundant API calls
- Configurable timeouts prevent hanging requests
- Memory-efficient log rotation

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Troubleshooting

### Connection Issues
- Verify internet connectivity
- Check if Solana RPC endpoints are accessible
- Try switching between Testnet and Mainnet

### Performance Issues  
- Close other resource-intensive applications
- Reduce refresh frequency
- Clear logs tab if memory usage is high

### Build Issues
- Ensure Rust toolchain is up to date: `rustup update`
- Clear build cache: `cargo clean && cargo build`

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with the [egui](https://github.com/emilk/egui) immediate mode GUI framework
- Uses [Solana Labs](https://github.com/solana-labs/solana) official RPC client
- Inspired by the need for better Solana network monitoring tools