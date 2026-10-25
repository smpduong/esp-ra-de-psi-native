# ESP Ra.De. Psi native research scaffold

This repository is a source-only research checkpoint for a native Rust/Bevy host and a small set of input contracts. It is not a playable port, a game backup, or a reconstruction of ESP Ra.De. Psi.

The host uses Bevy 0.19.1, requests Metal on macOS, accepts keyboard and gamepad input, and displays a moving square at a host-selected 60 Hz fixed step. The square's motion is synthetic diagnostic behavior. Separately, the public `RecoveredCore::tick` seam always returns `NotRecovered`; the host does not call that seam, and no recovered game routine is connected to the host.

## Build and run

With Rust installed, run from the repository root:

```sh
cargo test --locked -p recovery-core
cargo run -p native-host
cargo run -p native-host -- --smoke-frames 120
```

The first command covers four public contract tests. The host crate contains one unit test for its bounded diagnostic frame counter. A successful build or bounded host run demonstrates host initialization and loop behavior only, not game boot or gameplay. Private validation records a default suite of 144 passed with 49 ignored, plus a separate eight-test composition group that passed 8/8. Those private implementations, fixtures, receipts, and evidence are excluded here and are not publicly reproducible. This repository is not a backup of that private semantic work.

## Current recovery boundary

Private bounded startup research now reaches PC `0x5a06` after composing a startup prefix, a 64 KiB RAM clear, and a zero-RAM scan. The composition records 309,074 selected handlers, 578,888 modeled reads, and 2,956,652 model cycles. The zero path completes; a nonzero control fails closed at PC `0x59f4` on an unrecovered instruction before its store. These are selected Rust models with bounded fixtures, not original-machine-code whole-chain execution or a complete CPU emulator. They do not establish CPU boot, game boot, player control, or gameplay.

See [status](docs/STATUS.md) and the [publication boundary](docs/PUBLICATION_BOUNDARY.md).
