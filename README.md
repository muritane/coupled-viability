# Coupled Viability

**Jointly realizable trajectories under physical, informational, temporal, and relational constraints.**

An agent cannot make a desired trajectory executable merely by specifying it. Nor do individually feasible plans necessarily form a feasible joint plan. What matters is whether *all relevant constraints hold together*, under the system's actual embodiment, available observations, history, shared environment, deadlines, and bounded disturbances.

This repository contains an explicit research rationale and a **Rust HTTP API** implementing three deliberately small, falsifiable experiments:

1. **Stereo geometry:** two image observations require shared calibration, correspondence, and a single explainable 3D point. Independent 3D guesses do not establish a consistent reconstruction.
2. **Multi-agent trajectories:** independently deadline-feasible motions can collide. A bounded start-delay search can sometimes produce a collision-free joint schedule.
3. **Road-sign viability:** recognizing the correct sign is not sufficient when latency and braking/maneuver distance make the response too late. Better environmental support enlarges the viable initial-condition region.

The models are illustrative, *not* safety-certified perception, navigation, or automotive software.

## Why this exists

The motivating mistake is to treat independently specified observations, beliefs, objectives, plans, controls, and agents as though a coherent reality necessarily emerges from connecting their outputs. These quantities are coupled. Computational modules can be decomposed, but their underlying constraints cannot be treated as independent.

Non-uniqueness is **not** a defect: many trajectories may satisfy the same joint constraints. Heterogeneous agents can optimize locally within their differing reachable possibilities without being forced to share an objective or trajectory. However, their interactions must still respect shared constraints.

The research question is:

> When does improving shared constraint architecture and environmental support improve reliable, deadline-bounded outcomes more than improving isolated agent optimization?

Read the [full rationale and intellectual trajectory](docs/WHY.md), including the translation from commonplace human descriptions (the seven deadly sins, “works on my machine,” stereo vision, road signs), the formal distinctions, and what this approach does **not** claim.

## Run

Requires a stable Rust toolchain with Cargo (edition 2021).

```bash
cargo run
# default: http://127.0.0.1:3000
# to listen beyond localhost (only in a trusted network):
BIND_ADDR=0.0.0.0:3000 cargo run
```

Then from another terminal:

```bash
curl -s http://127.0.0.1:3000/health
curl -s http://127.0.0.1:3000/api/v1/experiments
curl -s -X POST http://127.0.0.1:3000/api/v1/experiments/stereo -H 'Content-Type: application/json' -d '{}'
curl -s -X POST http://127.0.0.1:3000/api/v1/experiments/road-sign -H 'Content-Type: application/json' -d '{}'
curl -s -X POST http://127.0.0.1:3000/api/v1/experiments/road-sign/sweep -H 'Content-Type: application/json' -d '{}'
curl -s -X POST http://127.0.0.1:3000/api/v1/experiments/multi-agent -H 'Content-Type: application/json' -d @examples/multi-agent.json
```

Endpoints require JSON request bodies and return JSON responses. `{}` uses defaults for stereo, road-sign and road-sign sweep. Multi-agent requires two agents' initial positions, goals, radii, speeds, and deadlines; see the provided file.

See [API.md](docs/API.md) for field descriptions, requests, outcomes, assumptions, and [OpenAPI 3.1](openapi.yaml).

## Verification

```bash
cargo fmt --all
cargo test --all-targets
cargo clippy --all-targets
```

GitHub Actions runs those commands on pushes and pull requests. We explicitly do not infer real-world reliability from the illustrative models or from passing unit tests.

## Scope and project structure

```text
src/
  main.rs          # configurable TCP entry point
  api.rs           # Axum HTTP routes, input errors and API tests
  stereo.rs        # rectified pinhole correspondence
  multi_agent.rs   # continuous-time collision check + delay-grid search
  road_sign.rs     # observability / reaction / stopping feasibility + sweep
docs/
  WHY.md           # comprehensive conceptual foundation and provenance
  API.md           # parameters, HTTP requests, assumptions
examples/
  multi-agent.json
  stereo-invalid.json
  road-sign-late.json
openapi.yaml
.github/workflows/rust.yml
```

## Relationship to earlier work

This is an **integration/testbed** rather than a claim to have invented constraint satisfaction, viability theory, real-time scheduling, multi-view geometry, or distributed control. It connects work already expressed in [relational_constraint_substrate](https://github.com/muritane/relational_constraint_substrate), [invariant_generating_constraint_substrate](https://github.com/muritane/invariant_generating_constraint_substrate), [geometric-viability-framework](https://github.com/muritane/geometric-viability-framework), [support-structures-and-viability-regions](https://github.com/muritane/support-structures-and-viability-regions), and [constraint_architecture](https://github.com/muritane/constraint_architecture).

The project remains deliberately open to counterexamples: showing that stronger coupling **reduces** attainable actions, increases latency, or worsens reliability under some conditions is just as relevant as showing improvement.

## License

See [LICENSE](LICENSE) (Creative Commons Attribution 4.0 International). This license covers the repository as committed; consult it before reuse, including source code.
