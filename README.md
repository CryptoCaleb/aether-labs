# Aether Labs

A desktop laboratory for testing one hypothesis: **reality is a single continuous medium (an aether), and matter and forces are vortex and flow structures within it.**

Each experiment is a real numerical model with **quantitative checks** against independent theory or experiment. The app reports PASS and FAIL honestly. Where the aether picture works, you'll see it work. Where it historically failed, you'll see that too.

## Experiments
| Area | Experiment | What it tests |
|---|---|---|
| Vortex matter (3D) | Kelvin ring, leapfrogging, collision, Kelvin waves, Hopf link, trefoil knot | Kelvin's vortex atom: ring speed vs Kelvin's formula, conservation laws, frozen-in topology |
| Vortex matter (2D) | Point vortices | 2D Coulomb energy plus Magnus motion (particle–vortex duality) |
| Forces | Bjerknes pulsating spheres | 1/d² force through the medium, and its inverted sign |
| Gravity as flow | Acoustic black hole | Unruh–Visser acoustic metric, horizon, ergosurface, Hawking temperature |
| Relativity | Sine-Gordon medium | Emergent Lorentz contraction, E = γmc², time dilation (Michelson–Morley) |
| Quantum aether | Rotating Gross–Pitaevskii superfluid | Quantised vortices, Abrikosov lattice, Feynman's rule |

The **Hypothesis scorecard** runs every validation suite and sets the results against a 12-point checklist of what any single-medium theory must reproduce. The **Research library** bundles the research briefs in [`docs/`](docs/).

## Build & run
Requires Rust (stable).

```bash
cargo run --release -p aether-lab
```

Run the physics validation suite:

```bash
cargo test --release -p aether-core
cargo test --release -p aether-core -- --ignored   # slow superfluid lattice test
```

On Windows with the GNU toolchain, MinGW (`dlltool`) must be on PATH.

## Layout
- `crates/aether-core`: pure physics, f64 + rayon, every model with `checks()`
- `crates/aether-lab`: egui/eframe desktop app
- `docs/`: research briefs and roadmap
