# Epact language charter v0.1

Type: principle
Status: accepted
Updated: 2026-09-03

Status: long-horizon semantic charter, named 2026-08-16

The language is named **Epact**. Surface syntax remains intentionally unset. New canonical programs
use `epact.program/0.1-alpha`; historical `concord.campaign` programs retain their original identity and
semantics rather than being rewritten under the new name.

## Name and language lineage

Epact names a real semantic lineage rather than an acronym fitted after the fact:

```text
ACT     Authority-Constrained Transition calculus
PACT    Prospective ACT
Epact   Evidential PACT
```

ACT is the small operational calculus of actors, authority, obligations, constraints, effects, and
eligible transitions. PACT adds frozen prospective commitments, temporal validity, ceilings, and
lawful amendment. Epact adds the epistemic world of observations, claims, evidence, contradiction,
review, interpretation, correction, and retraction.

The ordinary word *epact* is a period added to reconcile two calendars. Its lineage through Greek
*epagein*—to bring in, intercalate, or advance—is apt for a language that reconciles operational and
epistemic histories through explicit prospective additions rather than retroactive rewriting. The
etymology is resonance, not semantics; the formal expansion above defines the language family.

Concord is the first kernel and product built around Epact. Epact is not a Concord file format that
other systems may only transport: the long-horizon goal is an open, federated language that other
kernels and institutions can independently interpret and verify.

## The question this charter answers

The language is not being designed merely to make today's Concord application programmable. Its
design horizon is an environment in which machine intelligence is abundant, persistent, distributed,
and capable of acting through software, institutions, laboratories, and physical instruments.

The important question is therefore not:

> What workflow syntax should Concord use?

It is:

> What invariant coordination layer will remain necessary when the models, agents, tools,
> organizations, and regulatory environment have all changed underneath it?

Concord's answer is a language for verifiable delegation of consequential inquiry. Science is its
first and deepest domain because science makes uncertainty, provenance, replication, physical
intervention, evidence, and correction impossible to ignore.

## Ten-year environment

The expected environment contains:

- agents that generate hypotheses, design experiments, analyze observations, and propose follow-up
  work in continuous loops;
- heterogeneous agent systems that communicate without sharing implementations, internal memory, or
  model vendors;
- large populations of specialist agents operating concurrently and across long periods;
- automated laboratories, remote compute, shared instruments, and other systems with irreversible
  effects;
- campaigns spanning investigators, universities, companies, contract research organizations,
  national laboratories, infrastructure providers, and reviewers;
- data governed by different residency, confidentiality, consent, and licensing requirements;
- models and vendors whose useful lifetimes are shorter than those of the research programs they
  participate in; and
- regulatory, publication, and institutional demands for evidence that survives the original runtime.

Connectivity and intelligence will not be the scarce resources in this environment. The persistent
bottleneck will be legitimacy and coordination:

- Who acted for whom?
- Under which frozen instructions and authority?
- What could an actor inspect, change, spend, or disclose?
- Which effects actually occurred, and which remain ambiguous?
- Which observations are authentic and conformant?
- Was an adaptation prospective or outcome-driven?
- What does the resulting evidence support, contradict, or leave unresolved?
- Can another institution inspect the answer without trusting the original deployment?

Concord should mature into an institutional runtime for answering those questions, not remain an AI
application with a proprietary workflow DSL.

## Foundational semantic decision

A conventional program commonly denotes a computation from input to output:

```text
input -> computation -> output
```

A Concord program denotes constraints over an evolving, attributable history:

```text
(declared intent, observed history)
    -> (current obligations, permitted transitions, defensible claims)
```

More formally, a program defines a set of admissible histories. The kernel observes an append-only
history of proposals, authorizations, actions, receipts, observations, evaluations, decisions, and
amendments. At every point it determines:

- which transitions are eligible;
- which transitions are forbidden;
- which obligations remain open;
- which effects require additional authority;
- which in-flight actions are ambiguous;
- whether the history still conforms to the frozen program; and
- which claims, if any, are presently defensible from the recorded evidence.

Execution is an attempt to discharge the program's obligations while preserving its invariants. A
program is therefore not primarily a recipe for producing an output. It is a constitutional contract
for consequential work.

## The language's subject is delegation

Intelligent agents should not require every tactic to be specified in advance. Unbounded objective
pursuit, however, is not an acceptable authority model.

The language must express constrained autonomy of the following form:

```text
Pursue this objective
within this scientific and institutional scope,
using capabilities satisfying these qualifications,
without changing these frozen commitments,
within these effect, data, time, and budget ceilings,
while producing these evidence classes,
and escalate when these declared ambiguities arise.
```

An agent may choose tactics inside that space. It cannot redefine the space. Agent autonomy is a
typed degree of freedom rather than blanket permission.

## The two worlds

The language must maintain a hard distinction between an operational world and an epistemic world.

### Operational world

- actors and institutions;
- capabilities and qualifications;
- proposed actions and authorized effects;
- jobs, placements, instruments, and providers;
- approvals and delegations;
- budgets, reservations, and settlements;
- artifacts, receipts, failures, and time.

### Epistemic world

- questions and hypotheses;
- claims and their declared scope;
- observations and measurements;
- evidence and counterevidence;
- evaluations and interpretations;
- uncertainty, inconclusiveness, and contradiction;
- review, conclusion, correction, and retraction.

The language relates these worlds without collapsing them:

- a completed job is not itself scientific evidence;
- an artifact is a durable object, not automatically an observation;
- an observation is an attributable report, not automatically ground truth;
- evidence is a qualified relationship between observations and a scoped claim;
- support for a scoped claim does not establish a universal proposition;
- a model response is neither authorization nor evidence that an external effect occurred; and
- procedural conformance does not prove that nature agrees with a hypothesis.

Concord may establish identity, authority, conformance, lineage, and evidential relationships. It
must never manufacture scientific truth by typechecking a program.

## Frozen center and adaptive perimeter

A static workflow is too rigid for inquiry. A free-roaming agent is too unconstrained. A program must
explicitly divide constitutional commitments from adaptive obligations.

### Constitutional commitments

Depending on the domain, the frozen center can include:

- question and claim boundary;
- population, cohort, subject, and experimental-unit identity;
- intervention and comparator;
- primary and secondary endpoints;
- denominator and missingness treatment;
- evaluator, calibration, and acceptance criteria;
- exclusion and replacement policy;
- budget, authority, effect, and data-governance ceilings;
- stop conditions and conditional-stage predicates;
- terminal receipt requirements; and
- publication and interpretation boundaries.

### Adaptive obligations

The discretionary perimeter may allow an agent or human to:

- find a qualified placement satisfying declared constraints;
- choose among capabilities whose equivalence has been established prospectively;
- schedule eligible work within concurrency and budget ceilings;
- perform scientifically inert mechanical recovery;
- gather evidence satisfying an open evidence obligation;
- explore hypotheses in a declared exploratory branch;
- propose a new plan or amendment when an obligation cannot be fulfilled; and
- request additional authority without assuming it has been granted.

The program must make this boundary inspectable. A value must not be treated as adaptive merely
because it was omitted from the source or delegated to an agent.

## Core semantic vocabulary

The stable core should be small enough to survive changing scientific domains and implementation
technologies. Its irreducible concepts are expected to include:

- **Entity:** something with stable identity across time and systems.
- **Actor:** a human, agent, organization, or machine attributable for a transition.
- **Claim:** a scoped proposition that may be related to evidence.
- **Observation:** an immutable, attributable report of something observed or measured.
- **Evidence:** a qualified relationship between observations and a claim.
- **Capability:** a versioned and qualified means of producing an effect, artifact, or observation.
- **Effect:** a possible change to software, data, money, institutions, people, or the physical world.
- **Authority:** a scoped permission to propose, approve, delegate, or execute a transition.
- **Obligation:** a condition that must be discharged, explicitly waived, or remain visibly open.
- **Constraint:** an invariant every admissible history must satisfy.
- **Receipt:** a verifiable operational record about an attempted or completed transition.
- **Decision:** an attributable judgment over cited evidence and authority.
- **Amendment:** a versioned prospective change that preserves the history governed by prior rules.
- **Time:** both event order and the validity interval of identities, rules, authority, and claims.

Operations such as `derive`, `evaluate`, `execute`, `review`, `branch`, `gate`, and `publish` remain
useful standard forms. They should be constructed from the core vocabulary rather than mistaken for
the permanent semantic atoms.

## Lifecycle and lawful change

Programs themselves have a governed lifecycle:

```text
draft -> validated -> frozen -> authorized -> active
      -> terminal -> interpreted -> published
```

- **Draft:** semantic changes are permitted and visibly versioned.
- **Validated:** the program is well-formed, but has not acquired execution authority.
- **Frozen:** its prospective commitments have a stable identity.
- **Authorized:** named actors have granted bounded authority to activate it.
- **Active:** observations and effects accrue under the frozen version.
- **Terminal:** execution obligations have reached declared terminal states.
- **Interpreted:** decisions and claims are bound to exact evidence and program versions.
- **Published:** an immutable research object is released with its claim and provenance bindings.

After activation, a semantic change creates an amendment or successor version. It cannot rewrite the
rules governing prior observations. An amendment declares its justification, authority, effective
point, affected obligations, and relation to evidence already collected. Correction and retraction
append new attributable history; they do not delete the old record.

This lifecycle is jurisprudential by design: the language specifies both the rules and the lawful
procedure through which rules may change.

## Epistemic states cannot collapse

The language must preserve distinctions that ordinary programming systems often reduce to booleans or
generic failure:

- not yet observed;
- missing;
- execution failed;
- effect ambiguous;
- observation invalid;
- protocol nonconformant;
- evidence inconclusive;
- evidence supports;
- evidence contradicts;
- claim outside declared scope;
- conditional stage not activated;
- decision intentionally withheld.

These states have different downstream meanings. A failed measurement is not a missing measurement.
Absence of evidence is not evidence of absence. A skipped conditional stage is not a negative result.
A nonconformant campaign cannot be repaired by interpreting its outputs more confidently.

## Types, effects, and authority

The canonical representation should be informed by several type-system ideas without requiring users
to write academic notation:

- refinement types for units, populations, protocols, evaluators, and scientific constraints;
- capability-based authority for access and execution rights;
- algebraic effects for external actions and their handlers;
- linear or affine resources for one-use approvals, reservations, and consumable budgets;
- temporal rules for ordering, expiry, activation, and prospective amendment; and
- content-addressed identities for programs, inputs, outputs, and provenance.

A result should carry more meaning than `Table` or `Float`. An effect should carry more meaning than
`ToolCall`. The representation must retain domain type, units, subject or population, protocol,
provenance completeness, effect class, maximum exposure, replay semantics, and required authority
where relevant.

The authoritative core should not be Turing-complete. It should use bounded iteration over identified
sets, versioned predicates, explicit state transitions, and external capabilities for general
computation. The compiler and kernel should be able to answer before activation:

- What can this program spend or change?
- What data can leave each jurisdiction?
- Which actors can authorize or execute each effect?
- Which observations and evidence are required?
- Which choices remain open to an agent?
- Which obligations can remain unresolved?
- What are all declared terminal states?
- Which amendments would alter the scientific claim boundary?

Open-ended reasoning belongs in the harness. Authoritative legality belongs in the finite,
inspectable program.

## Federation and institutional verification

The ten-year language cannot assume that every transition occurs inside one Concord process or
database. A defining campaign may live at one institution while execution, measurement, analysis,
review, and publication occur at others.

The representation must therefore evolve toward:

- globally stable object and actor identities;
- canonical, deterministic serialization;
- content-addressed program and evidence bundles;
- signed delegations, approvals, attestations, and receipts;
- explicit institutional trust roots;
- portable domain schemas and semantic-version compatibility;
- selective disclosure and redaction proofs for restricted evidence;
- cross-system provenance and lineage; and
- verification that does not require access to the original runtime.

This does not prescribe a blockchain or any particular public ledger. It prescribes verifiability
beyond one database administrator's assertion.

The durability test is:

> Could an independent institution in 2036 determine what was proposed, authorized, executed,
> observed, and claimed if the original model providers and Concord deployment no longer existed?

## Proof-carrying work

A program cannot prove that a scientific conclusion is true. It can require work to carry
machine-verifiable support for procedural and evidential assertions such as:

- this actor possessed authority for this exact action;
- this artifact derives from these exact inputs and capability versions;
- this evaluator and denominator were frozen before these observations;
- this receipt accounts for every declared experimental unit;
- this execution satisfied its placement, data, budget, and effect constraints;
- this amendment applies only to future transitions;
- this evidence relationship uses the declared evaluator and claim scope; and
- this published object is the version that was reviewed and approved.

This is proof-carrying work: not proof of nature, but durable proof of what humans and machines did,
under which rules, and why a conclusion is entitled to the scope it claims.

## Humans, agents, and authoring

The canonical language need not be the interface most people edit directly. Mature programs will
often be elaborated collaboratively:

1. A human states an objective, constraints, and scientific intent.
2. An agent proposes a prospective program and marks unresolved obligations.
3. The compiler exposes dangerous effects, underspecified authority, ambiguity, and impossible
   promises.
4. Concord projects the same program as a scientific protocol, authority map, budget exposure,
   evidence contract, and execution graph.
5. Humans and authorized agents revise the draft.
6. Named actors freeze and authorize the constitutional center.
7. Agents elaborate only the declared discretionary perimeter.
8. The kernel validates each transition against the same canonical program.
9. Reviewers inspect the resulting history through appropriate projections.

The permanent asset is therefore a stable semantic representation with multiple authoring and review
surfaces:

- natural-language authoring;
- structured forms;
- graphical campaign design;
- domain-specific notation;
- agent-generated plans;
- direct canonical IR; and
- import/export mappings for external workflow, provenance, and research-object standards.

No surface syntax should become the sole representation of meaning.

## Layered language architecture

### Layer 1: small core calculus

The stable concepts and transition rules described in this charter. This layer should evolve very
slowly and remain independent of scientific domain, agent framework, and execution provider.

### Layer 2: canonical intermediate representation

A deterministic, versioned, content-addressed, diffable, signable representation validated by the
kernel. Historical versions must remain interpretable under their original semantics.

### Layer 3: domain vocabularies

Biology, chemistry, medicine, materials science, software engineering, and other communities define
objects, units, observation types, evidence relationships, and qualification policies. Concord owns
the meta-ontology that lets these vocabularies participate safely; it should not pretend to own a
universal ontology for every domain.

### Layer 4: surface languages and projections

Text, visual editors, notebooks, forms, natural language, and domain-specific views used to author or
inspect the canonical program.

### Layer 5: compilers and execution adapters

Adapters lower eligible obligations to Python, workflow engines, schedulers, agent protocols, cloud
APIs, robotic protocols, instruments, and future systems. Placement is a recorded compilation
decision, not part of scientific meaning.

## Interoperability posture

Concord should interoperate with external standards without delegating its semantics to them:

- tool and agent protocols can transport capability discovery, tasks, and messages;
- provenance standards can provide mappings for entities, activities, and actors;
- research-object formats can package programs, artifacts, metadata, and publications;
- workflow languages can serve as execution targets or imported operational fragments; and
- domain ontologies can supply identifiers, units, and scientific vocabularies.

These systems do not presently replace Concord's prospective authority, amendment, obligation,
evidence, and claim-conformance semantics. Interoperability should be expressed through explicit,
versioned mappings rather than by weakening the core to the least common denominator.

## Permanent decisions

The following commitments are intended to survive changes in syntax and implementation:

1. A program constrains admissible histories rather than prescribing one execution trace.
2. Proposal, authorization, execution, observation, evidence, decision, and claim are distinct.
3. Frozen commitments cannot be rewritten retroactively.
4. Authority, jurisdiction, data access, and effects are explicit and scoped.
5. Missing, failed, ambiguous, invalid, unsupported, contradicted, and out-of-scope remain distinct.
6. Agent discretion is declared and bounded.
7. Identity and provenance survive model vendors, deployments, and institutions.
8. The semantic core is small; domains extend it without redefining it.
9. Historical programs remain interpretable under their original language version.
10. The language can establish procedural and evidential conformance, never scientific truth by fiat.

## Deliberately deferred decisions

The following choices should be made only after the semantic core and real campaigns expose their
requirements:

- file extension and package coordinates;
- keywords and textual grammar;
- whether most users primarily see text, forms, graphs, or natural language;
- exact surface operations and syntactic sugar;
- preferred workflow engines and execution targets;
- preferred agent planner or model family; and
- dominant interoperability transports.

Deferring these choices is not indecision. It prevents current tooling and branding from becoming an
accidental permanent ontology.

## Design and acceptance tests

Every future language proposal should be challenged with these questions:

1. Can the same program survive replacement of every model and execution provider?
2. Can an agent exercise meaningful initiative without gaining undeclared authority?
3. Can the kernel enumerate maximum effects, spend, disclosures, and required approvals before
   activation?
4. Can a campaign change prospectively without altering the meaning of prior observations?
5. Can two institutions exchange and independently verify a program, delegation, and receipt?
6. Can the representation distinguish operational success from evidential support?
7. Can it retain failures and missing observations without contaminating denominators?
8. Can domain vocabularies evolve without changing the stable core?
9. Can a human understand why a transition is eligible, forbidden, or unresolved?
10. Can the complete historical meaning be recovered after the original implementation disappears?

A syntax or feature that cannot answer these questions should not enter the permanent core.

## Current execution projection

The first implementation asset is a versioned intermediate representation, not a permanent surface
syntax. Its alpha machine identity is `epact.program/0.1-alpha`; the file extension remains open. Readers must
continue to interpret pre-name `concord.campaign` programs under their recorded language version.
Its initial nodes are:

- `derive`: a deterministic or model-assisted transformation;
- `evaluate`: production of evidence under a versioned evaluator;
- `decide`: a policy or human decision citing evidence;
- `execute`: an effectful capability invocation;
- `review`: independent verification of a claim, calculation, citation, or artifact;
- `branch`: an immutable alternative or counterfactual;
- `gate`: an evidence predicate controlling eligibility; and
- `publish`: immutable artifact assembly with claim-to-evidence bindings.

Every node carries stable identity, input references, capability requirements, declared effects,
limits, output schema, retry policy, placement constraints, and terminal receipt requirements.

Placement is compilation, not meaning. One semantic node may lower to a local process, SSH worker,
HPC scheduler, hosted API, or instrument-adjacent adapter only when that placement satisfies its data
residency, resource, qualification, effect, approval, and budget requirements. The compiler rejects an
assignment rather than weakening the program to make it runnable.

The language requests capabilities such as `scientific_synthesis`, `tool_calling`, or
`independent_review`; it does not request a model vendor. Routing is a separate attributable decision.
Models may propose program fragments and effects, but Concord validates them and remains the runtime.

The first credible proof compiles one small campaign unchanged through a deterministic dry run, local
inference, a customer-key remote endpoint, SSH or scheduler compute, independent review, and recovery
after desktop disconnection. Semantic node identities must survive every placement while artifacts,
costs, approvals, and ancestry remain explicit.

## Near-term implication

The next language work should define, in order:

1. a precise admissible-history model;
2. the minimal ontology and identity rules;
3. program lifecycle and prospective amendment semantics;
4. authority, effect, obligation, and delegation rules;
5. epistemic states and claim-to-evidence relationships;
6. the canonical, versioned IR; and only then
7. authoring syntax and execution lowering.

The working north star is:

> An open, federated protocol for verifiable delegation of consequential inquiry between humans,
> agents, institutions, and machines.

Concord is the first kernel and product built around that protocol. It should not be the only system
capable of interpreting it.

## External reference points

These are interoperability and environmental reference points, not semantic authorities for the
language:

- W3C PROV-O: <https://www.w3.org/TR/prov-o/>
- Research Object Crate: <https://www.researchobject.org/ro-crate/specification.html>
- Model Context Protocol architecture:
  <https://modelcontextprotocol.io/specification/2025-06-18/architecture>
- Agent2Agent Protocol: <https://a2a-protocol.org/v1.0.0/>
- Google AI co-scientist:
  <https://research.google/blog/accelerating-scientific-breakthroughs-with-an-ai-co-scientist/>
- Robin, a multi-agent system for automating scientific discovery:
  <https://www.nature.com/articles/s41586-026-10652-y>
- AutoLabs, autonomous chemical experimentation:
  <https://www.nature.com/articles/s41598-026-45593-z>
