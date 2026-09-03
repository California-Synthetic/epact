# Epact

Epact is the model-independent language and executable contract for governed scientific work. It
turns a portable program of objects, capabilities, obligations, gates, evidence rules, authority,
effects, resources, amendments, and terminal conditions into a deterministic, content-addressed
program image that a runtime can enforce and replay.

This repository is the canonical home of:

- `epact-protocol`: portable source, image, event, replay, effect, and eligibility records;
- `epact-compiler`: deterministic normalization, validation, compilation, and amendment
  verification;
- `epact-runtime`: provider-neutral event authority, replay, eligibility, and terminal semantics;
- `epact-cli`: the `epact` reference compiler and independent verifier;
- `fixtures/alpha`: canonical source, image, history, projection, and eligibility vectors that every
  consuming runtime must reproduce exactly.

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

Compile, verify, replay, or evaluate canonical JSON records with:

```bash
cargo run -p epact-cli -- compile program.json
cargo run -p epact-cli -- verify-image image.json
cargo run -p epact-cli -- replay image.json events.json
cargo run -p epact-cli -- evaluate image.json events.json request.json
```

The committed alpha vectors are regenerated deliberately with
`cargo run -p epact-runtime --example generate_alpha_fixtures`; the conformance test then recompiles,
replays, and reevaluates them from disk.
