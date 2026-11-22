# Evolving constraints: history, versions, and continuity

## 1. Why a static intersection is not enough

A fixed constraint-satisfaction question asks whether a shared set of admissible configurations or transitions is nonempty. But operational systems also change the rules they implement: releases replace protocols, access-control policies are revised, schedulers are reconfigured, calibration parameters change, and environmental conditions vary.

A snapshot may be satisfiable both **before** and **after** a revision without admitting a safe, timely transition between those snapshots. In-flight work, retained state, buffered messages, nonreversible effects, and downstream dependencies cross the boundary.

The basic distinction is:

```text
version A individually works     version B individually works
              \                   /
               deployment/migration
                       |
            in-flight work and state
                       |
             no guarantee of success
```

We study **viability continuity**: whether required operations remain feasible through the change, given the actual physical process and the constructed constraints imposed upon it.

## 2. Physical restrictions and constructed restrictions

For explanatory purposes, let the physical transition relation at time t be

```text
P_t = { (x, a, x') | physical evolution from x to x' under a is possible }
```

and let A_(j,t) describe a *constructed* admissibility condition (an API schema, task priority rule, mutex, authorization policy, traffic signal, or sign convention). When these predicates are implemented as actual restrictions on transitions, the admissible relation is

```text
T_t = P_t ∩ A_(1,t) ∩ ... ∩ A_(n,t)
```

This is an intersection **after all predicates are lifted into a common space of state, action, and time**. A policy written in a document is not automatically enforced. A rule has to be implemented by physical processes; otherwise it describes a requirement or expectation, not necessarily a restriction on what physically happens.

The distinction is analytical, **not ontological**. Constructed rules run on matter and energy. A mutex needs memory, processors and scheduling. A release policy needs a deployment system. An institution depends on actual communication and action. Removing the underlying physical support removes active execution of these mechanisms.

In some models P_t varies because environmental conditions, parameters, or available resources vary. This does **not** mean fundamental physics is arbitrarily overwritten: the variation can arise from changing physical state or from revisions to our models.

## 3. Treat the active constraint configuration as state

Let x_t include agents, environment, physical resources, in-flight work, and other causally relevant variables; c_t denote the **actual active** constructed configuration; and r_t denote records or measurements of that configuration.

A useful augmented state is

```text
z_t = (x_t, c_t, r_t)
z_(t+1) ∈ F(z_t, u_t, w_t)
```

Version changes then become ordinary events within the same joint transition relation:

```text
(x_t, c_t, r_t)  --deployment, communication, execution-->
(x_(t+1), c_(t+1), r_(t+1))
```

The constraint configuration is not a privileged external authority exempt from physical limits; changing it consumes time, compute, access, memory, network capacity, and can affect ongoing work.

Sometimes a model writes `K_t` for the active admissible-state set and introduces `K_(t+1) = Γ(K_t, ...)`. This is convenient shorthand; the actual mechanism implementing Γ is part of the joint state and dynamics. It does not need a magical rule writer outside the system.

Histories matter when the current variables are not a sufficient description of the future. Rather than postulating an unbounded full-history variable, enlarge the state with whatever retained memory, pending operations, and causal context the experiment requires. If necessary, use explicit histories to state non-Markovian requirements.

## 4. Constraint relations are temporal properties

Several distinctions are needed:

| Question | Meaning |
|---|---|
| Individual satisfiability | Does each predicate have some allowed state/transition? |
| Joint satisfiability | Does their common admissible set contain anything? |
| Redundancy | Does removing one predicate leave the joint set unchanged? |
| Overconstraint | Is a required trajectory eliminated or its deadline made impossible? |
| Underconstraint | Do the remaining possibilities fail to establish the required guarantee? |
| Fragility | How easily do bounded variations eliminate the viable trajectories? |
| Transition compatibility | Can pending operations migrate across a change while meeting requirements? |
| Recoverability | After a disturbance or failed change, which required capabilities remain reachable? |

A large set is not inherently bad, and a small set is not inherently bad. The test is the required property, available observation/control information, time horizon, and disturbance class. A syntactically valid representation need not be operationally viable.

A candidate trajectory of augmented states, `τ = z_0, z_1, ..., z_H`, must satisfy the **active** conditions at the appropriate times:

```text
∀ t < H:
  (z_t, u_t, z_(t+1)) ∈ T(z_t)
  required events finish by their deadlines
  relevant safety conditions hold across the whole interval
```

The stronger robust question fixes the order of policy and disturbance quantifiers and checks feasibility for all declared disturbances. A nonempty snapshot intersection is insufficient for any of these temporal claims.

## 5. What Git provides, and what it does not

Git provides identifiable content snapshots, parent relationships, branch histories, merges, and reversions of *represented artifacts*. A pull request can propose a change, while CI can check some of its consequences. Both branches may work independently, and even an automatic merge can succeed syntactically, without their combined execution remaining viable.

A commit hash identifies committed content under Git's object rules. It does **not** identify the complete running configuration: live binaries, compiler/toolchain, memory, deployment state, queued requests, network conditions, permissions, and external effects may differ.

A revert makes another recorded transition; it does not automatically reverse already delivered messages, physical motions, financial transactions, or cached state. For those, an explicit compensating action may be needed, and compensation can itself fail or be incomplete.

Git's own development is version-controlled. That does not make it self-validating: its implementation and storage continue to depend on physical hardware, operating systems, energy, and other executable constraints.

## 6. No software–human ontological exception

Software, institutions, biological agents, and physical robots are all evolving physical processes. Different models are useful because their observed variables and mechanisms differ, not because one category uniquely experiences history, irreversibility, or partial observability.

We can represent continuously evolving processes using discrete recorded events. Discrete observation does **not** assert that the underlying process stops between events. Likewise, event-sourced software does not record every processor instruction; a scientific log does not record every physical state; and a historical archive does not observe every interaction.

The general pattern is

```text
evolving process → observations / event records → persistent representation
persistent representation + available resources → attempted reconstruction / execution
```

Both arrows are fallible, physically implemented mappings.

An event can be modeled as `e_k = (timestamp, source, observation, action, context, provenance)`. What must be recorded is determined by the distinctions needed for the particular question. For a scheduler bug, task releases, mutex ownership, interrupts, and deadlines may matter. For a deployment, active schemas and in-flight message timestamps may matter. There is no requirement for a complete description of everything.

The important distinction is **process versus representation**, not “human versus code.”

## 7. Experiment: a rolling deployment that breaks in-flight work

The Rust `deployment` experiment implements a minimal producer–consumer system.

Physical/execution assumptions:

- Messages are sent at recorded times, transported after a fixed latency, then processed by one FIFO worker.
- Each accepted message consumes a fixed amount of worker time.
- Each request has an absolute completion deadline.
- All changes have explicit effective times; an event at time t is active at t.

Constructed constraints:

- The producer emits a versioned message according to its **active version at send time**.
- The consumer accepts an explicitly configured set of versions **when service starts**.
- Accepted work continues with its start-time contract even if the active acceptance set changes before it finishes.
- Incompatible messages are rejected immediately at service start and consume no processing time in this small model.

Default event history:

| Time (ms) | Event | Active effect |
|---:|---|---|
| 0 | Request `before` is sent | Producer version 1; consumer accepts 1 |
| 3 | Request `in_flight` is sent | Producer version 1 |
| 4 | `before` arrives and starts | Version 1 accepted; completes at 6 |
| 5 | Producer and consumer change | Producer emits 2; consumer now accepts only 2 |
| 6 | Request `after` is sent | Producer version 2 |
| 7 | `in_flight` arrives | Version 1 rejected; valid when sent, invalid when received |
| 10 | `after` arrives | Version 2 accepted; completes at 12 |

The old and new **steady-state configurations** each agree on schema versions, but their transition is incompatible with the in-flight request.

This experiment is not a model of Git internals. It models running configurations and messages; a source-code version label would be only one observation of that process.

### Counterfactual 1: retain an overlap window

If the consumer accepts both versions 1 and 2 after the change, the in-flight request succeeds. This is a form of backward-compatibility window.

### Counterfactual 2: preserve schemas but increase processing time

Even if every version is accepted, a slow single worker can miss deadlines. Correct version history does not imply viable CPU execution.

### Counterfactual 3: change the representation only

Updating a repository or metadata label without changing the actual running configuration changes the *record*, not the active producer or consumer. A more detailed future simulator can represent deployment drift between these explicitly.

Run through:

```bash
curl -s -X POST http://127.0.0.1:3000/api/v1/experiments/deployment \
  -H 'Content-Type: application/json' -d '{}'
```

See [API.md](API.md) for a parameterized overlap-window example.

## 8. A proposed criterion for change viability

A proposed configuration transition should be evaluated with at least:

1. **Snapshot compatibility:** Are the old and new states internally admissible?
2. **In-flight compatibility:** Can ongoing work complete through the change?
3. **Temporal feasibility:** Do operations meet deadlines under declared resource assumptions?
4. **Observation fidelity:** Do records correctly identify the active configurations and relevant causal history?
5. **Recovery options:** After partial failure, can useful states and capabilities still be reached?

This should not be mistaken for an assertion that every change must preserve every old behavior. A system may deliberately withdraw support for an old interface or reject operations. The relevant design requirement is to declare *which guarantees are intended*, which operations may be lost, and which consequences or compensations are acceptable.

## 9. Testable hypotheses and limitations

A useful hypothesis is conditional: under specified workloads and resource bounds, explicitly managing compatibility windows and deployment timing reduces failure of in-flight operations compared with independently valid but uncoordinated version changes.

Counterexamples matter. Extended compatibility can increase memory, attack surface, complexity, or processing time. A compatibility window may be impractical or harmful. A deployment could remain safe by draining traffic instead of accepting both versions.

The toy model does not simulate packet loss, queues outside the single worker, replication, partial deployments, changing clocks, message ordering over independent networks, real authentication, or transactional rollback. It can demonstrate *why* snapshot consistency is insufficient; it cannot establish production reliability.

**Core conclusion:** Continuity of execution or version history does not imply continuity of joint viability. Changes to constraints are themselves constrained, time-bearing physical events.
