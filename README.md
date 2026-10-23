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

The first command covers four public contract tests. The host crate contains one unit test for its bounded diagnostic frame counter. A successful build or bounded host run demonstrates host initialization and loop behavior only, not game boot or gameplay. The source workspace's 140 default tests and five separately invoked private composition tests are not part of this export and are not publicly reproducible from it. This repository is not a backup of that private semantic work.

## Current recovery boundary

Private research has a bounded ROM-startup Rust composition that reaches PC `0x59b6` after 161,612 selected handlers, 202,046 modeled reads, and 892,212 model cycles. In that fixture, neutral and controlled pressed input states differ as expected. This uses selected Rust models and a manual map; it is not an original-machine-code whole-chain comparison or a complete CPU emulator, and does not establish CPU boot, game boot, player control, or gameplay. The next bounded source task is the MOVL oracle, followed by RAM-clear work.

See [status](docs/STATUS.md) and the [publication boundary](docs/PUBLICATION_BOUNDARY.md).
