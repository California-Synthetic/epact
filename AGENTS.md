# Epact repository protocol

Epact owns the portable language contract, canonical program image, compiler, verifier, replay
records, and command-line reference tooling used by Concord. It must remain independently usable
and must not depend on the private Concord product, its storage, providers, credentials, or
California Synthetic operations.

Epact programs describe scientific obligations, authority, effects, resources, evidence, review,
and termination. Models may propose programs and facts; durable runtimes decide what is accepted.
Canonical records must remain provider-neutral, content-addressed, deterministic, replayable, and
explicit about authority.

Proto Tools is an optional capability backend, not Epact's semantic foundation. Integrations must
sit behind an Epact/Concord capability adapter, pin a reviewed upstream revision, preserve upstream
and underlying tool licenses, and translate native inputs and outputs at the adapter boundary.
Epact must compile and verify without Proto Tools installed.

Before editing, read `git status` and preserve existing work. Structural invalidity is a compiler
error; missing activation authority is a structured finding. Normalization may remove irrelevant
ordering and duplicate set members but must not resolve conflicting declarations. Add positive,
negative, mutation, and determinism tests for semantic changes. Run
`cargo fmt --all -- --check && cargo test --workspace` before handing off a Rust change.

Use neutral branch names such as `architecture/<purpose>`, `feature/<purpose>`, or `fix/<purpose>`.
Keep commits single-purpose, stage exact paths, and never commit generated output or credentials.
