# Epact

Epact is the model-independent language and executable contract for governed scientific work. It
turns a portable program of objects, capabilities, obligations, gates, evidence rules, authority,
effects, resources, placement constraints, amendments, and terminal conditions into a
deterministic, content-addressed program image that a runtime can enforce and replay.

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

## Alpha boundary

The alpha is a complete language seam, not a complete scientific operating system. It provides a
strict compiler, provider-neutral runtime semantics, a reference CLI, and portable conformance
fixtures. A consumer remains responsible for durable storage, qualified tool adapters, credentials,
cost accounting, and effect execution; it must bind those effects back to the compiled image rather
than allowing a model or adapter to reinterpret policy.

## Proto Tools boundary

[Proto Tools](https://github.com/evo-design/proto-tools) is the first planned large capability
backend. The current adapter baseline was reviewed at upstream commit
`d70c99a797c2c1c27632ed30429d669aec5e2839`. Its isolated tool environments, persistent workers,
device management, parallel pools, typed I/O, caching, and biological tool catalog are valuable
infrastructure, but they are not part of Epact's language semantics. Concord will consume them
through a separately qualified adapter pinned to an exact upstream revision. This pin records the
candidate boundary; it does not claim that the adapter is qualified or executable. Epact compilation
and verification remain fully independent of Proto and biology-specific types.

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
