# HTTP API and experiment contracts

This service runs locally by default: `http://127.0.0.1:3000`. Set `BIND_ADDR` to an explicit socket address to change it. The server has **no authentication**, rate limiting, or TLS; do not expose it on the public Internet without an appropriate reverse proxy and access controls.

The contract is described in [openapi.yaml](../openapi.yaml). All POST endpoints require `Content-Type: application/json`. Missing fields use documented defaults for stereo, deployment and road-sign requests. Malformed JSON receives an Axum rejection; domain validation errors return HTTP 400 with `{"error":"..."}`. Request bodies are limited to 64 KiB.

## Service endpoints

| Method | Path | Purpose |
|---|---|---|
| GET | `/health` | Runtime/version status |
| GET | `/api/v1/experiments` | Experiment names |
| POST | `/api/v1/experiments/stereo` | Rectified stereo candidate consistency |
| POST | `/api/v1/experiments/multi-agent` | Independent vs jointly feasible trajectories |
| POST | `/api/v1/experiments/deployment` | Producer/consumer version changes with in-flight work and deadlines |
| POST | `/api/v1/experiments/road-sign` | Recognize-and-respond before decision line |
| POST | `/api/v1/experiments/road-sign/sweep` | Compare sign designs over a deterministic grid |

## Stereo

```bash
curl -s -X POST localhost:3000/api/v1/experiments/stereo \
  -H 'Content-Type: application/json' -d '{}'
curl -s -X POST localhost:3000/api/v1/experiments/stereo \
  -H 'Content-Type: application/json' -d @examples/stereo-invalid.json
```

Defaults: baseline 0.2 m, focal length 500 px, principal u 320 px, left pixel (350,240), right (330,240), epipolar tolerance 1 px, independently assumed depth 3 m, pixel sigma 0.3 px. A rectified camera pair is assumed, both optical axes aligned, baseline along x.

Output fields: disparity (pixels), epipolar residual (pixels), independent-world-point disagreement at a **common arbitrarily assumed depth** (meters), and optional joint reconstruction (meters and approximate first-order depth sigma). When observations violate the toy geometry, reconstruction is `null`. The independent disagreement is pedagogical rather than a statistically principled independent-depth estimator.

## Multi-agent

```bash
curl -s -X POST localhost:3000/api/v1/experiments/multi-agent \
  -H 'Content-Type: application/json' -d @examples/multi-agent.json
```

Each agent needs `start` and `goal` planar coordinates in meters, `speed_mps`, `radius_m`, and `deadline_s`. Joint scheduling has `safety_margin_m`, `delay_step_s`, and `max_delay_s`.

Each agent follows its straight-line segment at constant speed, waits at its start until its scheduled delay, then stays at its goal. The solver checks distance continuously between time events and searches a grid of start delays up to 100 steps per agent. It minimizes makespan among feasible schedules in that **restricted** search space. A failed search does *not* prove global infeasibility when alternative paths or intermediate waits exist.

`independent_collision` describes both agents starting immediately. `joint_schedule_found` means a valid delay-grid solution was found within both deadlines and physical separation bounds.


## Versioned deployment

```bash
# Both stable configurations support matching schemas, but an in-flight
# request sent with version 1 arrives after the consumer switches to 2.
curl -s -X POST localhost:3000/api/v1/experiments/deployment \\
  -H 'Content-Type: application/json' -d '{}'

# During the transition, the consumer accepts both schema versions.
curl -s -X POST localhost:3000/api/v1/experiments/deployment \\
  -H 'Content-Type: application/json' -d @examples/deployment-overlap.json

# Fully compatible versions still cannot guarantee CPU deadlines.
curl -s -X POST localhost:3000/api/v1/experiments/deployment \\
  -H 'Content-Type: application/json' -d @examples/deployment-deadline.json
```

Input fields:

| Field | Meaning |
|---|---|
| `initial_producer_version` | Schema emitted before any producer change (default 1) |
| `initial_consumer_accepts` | Versions initially accepted (default [1]) |
| `producer_changes` | Ordered `{at_ms, version}` changes |
| `consumer_changes` | Ordered `{at_ms, accepted_versions}` changes |
| `requests` | Unique `{id, sent_at_ms, deadline_at_ms}` operations |
| `network_latency_ms` | Fixed transport delay (default 4 ms) |
| `processing_time_ms` | Fixed processing time on one FIFO worker (default 2 ms) |

Defaults change producer and consumer at 5 ms from version 1 to version 2, with three operations sent at 0, 3, and 6 ms. The middle request is version-compatible when sent, but becomes incompatible before service starts. The result reports compatible initial and final configurations, **one schema rejection**, and an event trace with send, receive, start and finish times. The overlap example accepts both 1 and 2 during the transition, avoiding the rejection. The slower-worker example shows missed deadlines despite valid schemas.

Changes take effect inclusively at their `at_ms`. A request uses the producer version at send time and the consumer's accepted-version set at *service start*. Accepted requests remain accepted until completion. Incompatible requests are rejected without worker time. At most 10,000 requests and 1,000 changes of either type are allowed; change timestamps within each list must be strictly increasing. The model has one deterministic FIFO worker, not a complete distributed runtime.

Do not confuse a source repository version with the actual process state. This experiment models **active versions**, not Git metadata. For its physical interpretation and limits, read [Evolving Constraints](EVOLVING_CONSTRAINTS.md).

## Road sign

```bash
curl -s -X POST localhost:3000/api/v1/experiments/road-sign \
  -H 'Content-Type: application/json' -d '{}'
curl -s -X POST localhost:3000/api/v1/experiments/road-sign \
  -H 'Content-Type: application/json' -d @examples/road-sign-late.json
```

Input parameters:

| Parameter | Unit | Interpretation |
|---|---|---|
| `approach_distance_m` | m | Initial distance before decision line |
| `speed_mps` | m/s | Constant approach speed before response |
| `sign_width_m` | m | Physical width used for angular legibility |
| `sign_height_m` | m | Recorded geometry parameter (not used in the 1D legibility test) |
| `visibility_m` | m | Environmental visibility bound |
| `visual_quality` | 0–1 | Simplified multiplier of effective width |
| `minimum_angular_width_deg` | degrees | Threshold for discriminability in this proxy |
| `recognition_latency_s` | s | Time to decode the signal after it becomes legible |
| `reaction_latency_s` | s | Time before response actuation |
| `uncertainty_margin_s` | s | Extra delay margin |
| `deceleration_mps2` | m/s² | Constant positive braking deceleration |
| `maneuver_margin_m` | m | Additional required spatial margin |

The output includes first legible distance, necessary stopping/maneuver distance, slack, and boolean response feasibility. The model assumes a straight approach, a stationary sign, one recognition event at first visibility, and idealized braking. The result is **not** a measured human or machine perception probability.

## Sign sweep

```bash
curl -s -X POST localhost:3000/api/v1/experiments/road-sign/sweep \
  -H 'Content-Type: application/json' -d '{}'
```

The optional `baseline` and `improved` objects use the road-sign parameters, and three arrays define a Cartesian product:

- `speeds_mps` (default: 10, 15, 20, 25, 30)
- `visibility_factors` (default: 0.5, 0.75, 1)
- `quality_factors` (default: 0.6, 0.8, 1)

For each triple both designs get the *same* speed and multiplicative visibility/quality conditions. Up to 10,000 combinations are supported. The reported fractions are **fractions of this grid**, not population-level or weather-event probabilities. To make probabilistic claims, use a defensible distribution and repeated noisy trials.

## Interpretive cautions

The API uses `f64` numerical computations, not formal proofs or hardware validated controllers. Defaults are chosen to make the contrasting cases visible. In particular, joint scheduling is not decentralized negotiation; stereo does not perform feature detection; the sign experiment does not use an actual vision model. These limitations are part of the experiment specification.
