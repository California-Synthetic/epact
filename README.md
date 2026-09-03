# Epact

Epact is the model-independent language and executable contract for governed scientific work. It
turns a portable program of objects, capabilities, obligations, gates, evidence rules, authority,
effects, resources, amendments, and terminal conditions into a deterministic, content-addressed
program image that a runtime can enforce and replay.

This repository is the canonical home of:

- `epact-protocol`: portable source, image, event, replay, effect, and eligibility records;
- `epact-compiler`: deterministic normalization, validation, compilation, amendment verification,
  and the `epactc` reference CLI;
- conformance fixtures and tests that every consuming runtime must pass.

The accepted long-horizon semantics are recorded in the
[`Epact language charter`](docs/language-charter.md).

Concord supplies the scientific control plane and durable runtime. Epact does not own campaign
storage, credentials, provider routing, or user-interface state.

## Proto Tools boundary

[Proto Tools](https://github.com/evo-design/proto-tools) is the first planned large capability
backend. Its isolated tool environments, persistent workers, device management, parallel pools,
typed I/O, caching, and biological tool catalog are valuable infrastructure, but they are not part
of Epact's language semantics. Concord will consume them through a qualified adapter pinned to a
reviewed upstream commit. Epact compilation and verification remain fully independent of Proto and
biology-specific types.

## Verify

```bash
cargo fmt --all -- --check
cargo test --workspace
```

Compile or verify a canonical JSON program image with:

```bash
cargo run -p epact-compiler --bin epactc -- compile program.json
cargo run -p epact-compiler --bin epactc -- verify-image image.json
```
