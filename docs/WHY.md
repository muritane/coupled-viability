# Why coupled viability is fundamental

## 0. Thesis, scope, and the question

An independently valid statement is not automatically jointly realizable. An observation is not a scene. A plan is not an action. A physically possible action is not necessarily timely. A locally admissible trajectory need not be compatible with its neighbors. A recognized signal is useless for some tasks after the latest decision time.

**Coupled viability** asks which trajectories remain simultaneously realizable when physical state, observation, representation, computation, timing, shared resources, other agents, and environment are modeled together, under bounded uncertainty.

“Fundamental” here means *necessary to make certain execution claims meaningful*, not a metaphysical axiom or a claim of mathematical novelty. A system's feasible behavior cannot be inferred from unconstrained wishes or independently modeled parts. The exact constraints, causal effects, and relevant state must still be specified empirically.

The central research question is **not** “which objective must everyone pursue?” It is: **which jointly constrained trajectories are accessible to locally situated agents, by what mechanisms, and with what timing, robustness, and recoverability?**

## 1. How we arrived here: from everyday language to executable questions

The starting point was an everyday enumeration of supposed human “sins”: pride, greed, lust, envy, gluttony, wrath, sloth, together with dishonesty, gossip, selfishness, and omission. Translation into development slang produced examples such as “works on my machine,” chasing a shiny framework, hoarding compute resources, and ignoring a failing build.

A second translation used robotics metaphors: pride became overconfident priors, wrath became excessive actuator response, and sloth became calibration or model drift.

The flaw in these mappings is that *labels do not identify mechanisms*. A high-confidence prior may be justified; allocating a resource may be necessary; stopping to act may be the only safe action. A behavior becomes explainable only relative to the actual state, model, measurements, actuation capabilities, environment, history, and shared dependencies. Without these, naming a vice or “failure mode” is not a diagnosis.

The conversation then exposed a deeper flaw: making a list of modules—perception, belief estimation, objective selection, planning, control, coordination, adaptation—suggested that each module could be specified independently and repaired by adding “ethics” or an externally preferred objective. But the modules describe *one interacting physical process*. Each constrains the others, including over time.

Instead of asking “What should an agent want?”, ask: what can it sense, what can it infer, what is physically possible from its current state, what competing demands exist, what is jointly possible with other agents, and which actions leave sufficient future options? This does **not** uniquely determine a trajectory, and it need not. Heterogeneous agents may follow different viable paths.

### 1.1 A diagnostic crosswalk, not a new morality

The original everyday vocabulary can be operationalized as **hypotheses**, not one-to-one equivalences:

| Common label | Engineering anti-pattern | Observable constraint question |
|---|---|---|
| Pride | “Works on my machine” | Do model predictions remain compatible with observations across the deployed hardware and environmental distribution? |
| Greed | Local resource monopolization | Does one participant's allocation deplete shared bandwidth, power, time, or recovery margin needed by others? |
| Lust | Reward chasing / hype-driven rewrite | Does short-horizon gain eliminate feasible longer-horizon trajectories? |
| Envy | Copying another architecture or trajectory | Is the imported plan admissible under this agent's embodiment, topology, and history? |
| Gluttony | Unbounded dependency/input growth | Does consumption exceed processing, storage, energy, or timing budgets? |
| Wrath | Excessive control action | Does the response violate actuator, safety, stability, or disturbance bounds? |
| Sloth | Ignored calibration / maintenance | Does accumulated model drift remove reachable corrective actions? |
| Dishonesty | Inconsistent reported telemetry | Does a communicated claim disagree with independently constrained observations? Intent is a separate empirical issue. |
| Gossip | Propagation of unverified reports | Does correlated misinformation contaminate shared estimates or induce invalid actions? |
| Selfishness | Independent planning with externalized costs | Are local feasible trajectories jointly collision-free and resource-feasible? |
| Omission | Dropped fault or missed deadline | Was the event observable and actionable in time, given capacity and actual obligations? |

This crosswalk **does not establish guilt, blame, or moral correctness**. It is a way of converting vague labels into questions about measurable mismatches and engineering interventions. The converse is equally important: a system can satisfy its specified constraints yet still leave open whether the specification covers all relevant consequences. Model boundaries and omissions must be documented and revised.

### 1.2 Why “works on my machine” is a compact statement of the problem

A build that succeeds on machine A under configuration E_A has shown only that at least one trajectory succeeds under those conditions. It has not established success under machine B, E_B, timing variation, workload, or hardware interfaces. The shared claim “it works” requires explicit scope.

Similarly, two robots can each plan a collision-free path *against a static environment* and still collide with one another. “Locally valid” and “jointly feasible” are different predicates.

## 2. Stereo vision: the minimal counterexample to independence

For two calibrated, rectified cameras observing the *same* feature, let B be baseline, f focal length, u_L and u_R horizontal pixel coordinates, c_x principal point, and d = u_L - u_R disparity. Under a pinhole model:

```text
Z = f * B / d
X = (u_L - c_x) * Z / f
```

A positive disparity and consistent scanline are necessary in this simplified setup; actual matching also depends on feature identity, occlusion, rectification accuracy, lens distortion, and uncertainty.

Without constraints relating the camera poses and one common world point, we have independent image positions, not an identified 3D scene. A second camera does not automatically improve reconstruction: poorly calibrated or mismatched observations may **shrink the consistent solution set to empty**. This is evidence of a modeling or measurement mismatch, not a reason to arbitrarily select a “correct” world state.

**Important:** even a jointly consistent pair does not always provide unique depth or correspondence. Gauge freedom, degenerate geometry, and uncertainty matter. Equivalence classes are the right language when states coincide only under a selected projection or transformation.

## 3. From multi-view to multi-agent

Two physical agents have distinct embodied states and different views. Their reference coordinates may coincide in a projection without them having the same complete physical configuration. For robots with nonzero bodies, shared occupied volume conflicts with collision constraints.

Represent the joint state by

```text
z_t = (x_1,t, ..., x_n,t, environment_t, resources_t, history_t, ...)
```

with joint dynamics

```text
z_(t+1) = F(z_t, a_1,t, ..., a_n,t, w_t)
```

and local observations

```text
y_i,t = h_i(z_t, sensor_i, noise_i)
```

Each local policy can use its own belief, goals, and information:

```text
a_i,t = pi_i(y_i,0:t, messages_i,0:t, memory_i,t)
```

There is no requirement that the policies, goals, trajectories, or representations match.

However, the **joint** sequence must satisfy shared geometry, resource, communication, and timing constraints. If two independently planned trajectories cross at the same time, their combination is not in the joint feasible set. Coordination can take place through sensing, signaling, reservation, yielding, environmental design, or re-planning—subject to computation and latency limits. In practice, distributed solutions approximate this coupled requirement; “simultaneous” refers to *joint satisfaction*, not mandatory instantaneous centralized computation.

## 4. Feasibility, reachability, viability, recoverability

Different mathematical questions must not be collapsed:

1. **State feasibility:** is z in a specified admissible set K?
2. **Action/transition feasibility:** is (z,a,z') an allowed transition given dynamics and constraints?
3. **Reachability:** can the process reach a target G within horizon H?
4. **Reach-avoid:** can it reach G by H without entering a forbidden set?
5. **Viability:** can a policy keep the trajectory in K over a specified horizon despite defined disturbances?
6. **Recoverability:** after deviation or degradation, which viable states or trajectories remain accessible, at what cost, with what likelihood?
7. **Deadline feasibility:** can sensing, computation, and physical response complete before the last usable time?

A useful bounded reach-avoid formulation is

```text
R_H = { z_0 | exists an admissible policy pi such that
              for all allowed disturbances w:
              z_t in K for t in [0,H],
              and z_t in G for some t <= H }
```

Under stochastic assumptions replace a worst-case condition with an explicitly stated probability threshold. Every claim must declare **which** disturbances, models, horizons, and agent information states it covers.

Physical restrictions can narrow mathematical possibilities but create **usable capabilities** through stable interfaces and predictable support. A road sign, lane markings, standardized units, or communication protocol can expand robust reachable behavior by making observations actionable. This does not imply that adding arbitrary restrictions always enlarges a set; constraints can also remove possibilities. The impact of an *architectural redesign* must be measured.

## 5. The road-sign example: why the environment is part of the controller

Consider an agent approaching a turn or exit. Recognition alone is not the right success predicate. The signal must become discriminable early enough for decoding, decision, actuation, and motion before the commitment point.

A crude longitudinal condition is

```text
available_distance >= speed * (recognition_latency
                               + reaction_latency
                               + uncertainty_margin)
                    + speed^2 / (2 * braking_deceleration)
                    + maneuver_margin
```

Available distance is limited by initial approach distance, range of visibility, and angular legibility. Real systems also depend on glare, rain, dust, occlusion, road geometry, visual language, camera optics, sensor motion, human learning, and actuation disturbances.

A sign designed to be visible only from one meter away is technically recognizable in a narrow configuration but generally fails the timing requirement. A larger, earlier, redundant, standardized sign can enlarge the set of starting conditions and agents from which an actionable signal is recoverable in time.

This is **environmental support as part of joint dynamics**. It reduces the burden of requiring all agents to invent the same interpretation or planning policy independently.

The example does not imply that every sign must be universally legible. It implies that its claimed operating domain and accessibility assumptions must be measured and tested.

## 6. Locality, history, and non-unique outcomes

An agent in Rome cannot visit a physical landmark in Paris without traversing the necessary route and consuming time and resources. It can look at a photograph, travel later, or pursue another activity. These are *different executable trajectories*. Declaring a destination does not transport the environment.

Feasible sets depend on current configuration, history, hardware/software version, topology, other agents, and what changes can be made along the way. Local circumstances constrain what is immediately actionable and what may become actionable later.

**Non-uniqueness is not an error.** Multiple viable trajectories can coexist; no external uniquely “correct” objective follows merely from feasibility. Individual agents can optimize locally. Their shared interfaces and interactions must remain jointly admissible, or failures can arise from otherwise valid local behavior.

Constraints alone do not predict which feasible action will occur; that requires a model of policies and influences, including stochasticity and learning. These should be studied empirically rather than imposed as universal rules.

## 7. A joint formulation for research

Let theta denote architecture and environmental design (sensor placement, sign size, communication paths, resource capacity, protocol timing), and let pi denote agent policies.

We can investigate:

```text
V_H(theta, pi) =
  { z_0 | admissible coupled trajectory exists
          and required events complete before deadlines
          under declared model/disturbance assumptions }
```

For a family of agents, one question is how modifying theta changes the measure or distribution-weighted mass of V_H. But **a larger geometric set** and **higher probability under a given population** are different metrics, and they may trade off against cost, throughput, privacy, or other measurable consequences.

Policies may be fixed, jointly synthesized, or local adaptive policies. The quantifier order matters:

```text
exists one shared robust policy for all disturbances?
for each disturbance, does some policy exist?
can each agent choose a policy from local observations only?
```

These are different mathematical claims. The toy experiments do **not** solve the general decentralized partially observed control problem.

Important failure cases to test include:
- independently valid solutions whose joint intersection is empty;
- a nonempty joint feasible set with no executable decentralized policy under given information;
- recognition that succeeds but misses its response deadline;
- environmental improvements that enlarge viable regions only for some agent populations;
- stronger coupling that increases overhead and reduces feasibility;
- solutions feasible under zero disturbance but infeasible under uncertainty or latency.

## 8. The initial experiments and what they prove

### E1 — Rectified stereo

A pair of pixel coordinates is used to compare independent same-depth estimates with a jointly constrained pinhole reconstruction. Epipolar mismatch and nonpositive disparity invalidate the candidate.

**It does not prove** correct feature identity, uniqueness under arbitrary camera geometry, or real-world calibration validity.

### E2 — Two mobile agents

Two discs move along straight-line paths with constant speed. Each can reach its goal before its deadline. The independent simultaneous plan intersects; a finite-grid search over initial waiting times may find a joint collision-free, on-time schedule. Minimum separation is checked *continuously* over all piecewise-linear segments (not by fixed-timestep sampling).

**It does not solve** arbitrary collision-free multi-agent pathfinding, unknown dynamics, locomotion, heterogeneous sensors, or distributed protocols.

### E3 — Road-sign viability

Angular legibility, effective visibility, observation latency, reaction latency, and braking/maneuver distance determine whether a sign is actionable before a decision line. A deterministic sweep counts how often a modified design works across a specified finite grid of speed, visibility, and visual-quality factors.

**It does not establish** recognition probability or automotive compliance. Sweep fractions are not statistical probabilities without a justified operating-condition distribution.

### E4 — Versioned deployment across an event boundary

Two producer/consumer versions each agree on a schema in their own stable configuration, but a request sent under the old configuration can arrive after the consumer stops accepting that schema. Fixed transport latency, processing time, and deadlines jointly determine the outcome. Keeping the old schema accepted during the overlap restores the example's in-flight compatibility.

**It does not prove** that source versions identify running configurations or that recorded commits alone guarantee compatible physical execution. See [Evolving Constraints](EVOLVING_CONSTRAINTS.md).

## 9. Why this is distinct from existing repositories

This project does not rename the already existing theories. The relationships are:

- [Relational Constraint Substrate](https://github.com/muritane/relational_constraint_substrate): state distinctions, joint interpretation, constraint composition and coupling. Its formal companion already defines intersection of admissible transitions and nonempty joint action sets.
- [Invariant-Generating Constraint Substrate](https://github.com/muritane/invariant_generating_constraint_substrate): how constrained degrees of freedom produce recoverable invariants.
- [Geometric Viability Framework](https://github.com/muritane/geometric-viability-framework): admissible regions and viable reachable futures.
- [Support Structures and Viability Regions](https://github.com/muritane/support-structures-and-viability-regions): support structures and environmental sustaining conditions.
- [Constraint Architecture](https://github.com/muritane/constraint_architecture): physical execution, information limits, bounded resources, topology, cognitive/control layers.

Here the missing connection is exercised in code: **one set of physical facts, multiple views and agents, intersecting admissibility conditions, finite observation and action times, and measurable differences between isolated and coupled designs.**

Related established fields include multi-view geometry, factor graphs, viability theory, reachability and safety control, real-time scheduling, distributed constraint optimization, and multi-agent pathfinding. Novelty must be established through formal results or comparison with baselines, not by changing terminology.

## 10. Research discipline: how to falsify the proposal

The claim worth testing is conditional:

> For specified agent populations and operating distributions, improving the shared constraint/environment architecture can increase the probability or measure of deadline-bounded joint success without requiring identical local objectives.

Strong tests need:
- controlled baselines (isolated policies versus coupling/environment modifications);
- documented identical budgets and disturbance distributions;
- state and action traces, not just scalar rewards;
- collision margins, missed deadlines, recognition/response success, failure reasons;
- sensitivity across uncertainty and latency;
- negative results, including cases where coupling adds overhead and makes outcomes worse;
- progression from toy models to calibrated simulations and then hardware where appropriate.

No amount of internally consistent prose establishes these outcomes by itself. This repository starts with executable counterexamples and exposes assumptions for direct inspection.

## 11. Evolving constraints are part of the evolving physical state

The earlier formulation should not be mistaken for a once-and-for-all intersection of fixed constraint sets. Constructed constraints change, and **their changes are physical, time-bearing transitions**. The agents, environment, active configuration, queued operations, resource state, and relevant recorded history can be modeled as an augmented joint state. Source-control revisions identify representations, not all live instances or consequences of running code.

An individually valid old configuration and individually valid new configuration need not admit a viable migration. Requests crossing a change can become incompatible or miss deadlines. Conversely, adding a compatibility window can save in-flight operations while increasing processing or maintenance cost. The issue is joint trajectories under active constraints, not whether continuous processes happen to have discrete event records.

This applies to software, institutions, robotic controllers, and other embodied agents without introducing an ontological exception for humans. What changes across domains is the physical dynamics and what can be observed or recorded. Partial observability, irreversibility, and causal dependence occur in running software as well.

The companion [Evolving Constraints: History, Versions, and Continuity](EVOLVING_CONSTRAINTS.md) contains the formal distinction between physical and constructed restrictions, versioned state and record, Git-like histories, event boundaries, and the executable deployment experiment.

**Continuity of activity is not continuity of viability.**
