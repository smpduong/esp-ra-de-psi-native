# Status — checkpoint 2026-10-23

This is an incomplete research scaffold. It contains a Bevy 0.19.1 diagnostic host with Metal selection on macOS, keyboard/gamepad input adapters, synthetic square movement, and a host-selected 60 Hz fixed step. None of those host behaviors are recovered ESP Ra.De. Psi gameplay. Separately, `RecoveredCore::tick` returns `NotRecovered` on every call; the diagnostic host does not call it.

The public core includes four dependency-free input-contract tests; all four passed with offline `cargo test --locked -p recovery-core --all-targets` on 2026-10-23. The public host includes one bounded-frame-counter unit test, which was not freshly run. No fresh GUI or Metal run was performed. The private source workspace has reported 140 default tests and five separate private composition tests; their source, fixtures, receipts, and evidence are excluded here, so they are not publicly reproducible.

Private bounded research reports a ROM-startup composition reaching PC `0x59b6` after 161,612 selected handlers, 202,046 modeled reads, and 892,212 model cycles. Its neutral and controlled pressed input states produce the expected distinction in the fixture. The composition uses selected Rust models and a manual map; it is not an original-machine-code whole-chain comparison or a complete CPU emulator, and does not establish CPU boot, game boot, player control, or gameplay. The next bounded work is a MOVL oracle, then RAM-clear research.

These statements summarize private research without publishing its implementation, test evidence, inputs, or receipts. Recheck all claims against the private project before citing this checkpoint.
