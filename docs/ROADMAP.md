# Aether Labs Roadmap

The app tests one hypothesis: reality is a single continuous medium, and matter and forces are vortex and flow structures within it. Each milestone adds a model with quantitative checks. Passes and failures are both reported.

## Done (v0.1)
- [x] Rust workspace: `aether-core` (physics, tests) + `aether-lab` (egui desktop app)
- [x] Vortex filament engine:
  - Rosenhead–Moore Biot–Savart, RK4, adaptive resampling, rayon parallel
  - Diagnostics: impulse, energy, linking number, writhe
  - Experiments: Kelvin ring, leapfrogging, head-on collision, Kelvin waves, Hopf link, trefoil knot, passive "aether tracers"
- [x] 2D point vortices ("2D Coulomb + Magnus"), with presets and click-to-add
- [x] Bjerknes pulsating spheres: force law, phase dependence, dynamics, sign inversion
- [x] Acoustic black hole: draining bathtub flow, ray tracing, horizon/ergosurface, κ and Hawking T
- [x] Emergent relativity:
  - sine-Gordon kink contraction, E = 8γ
  - breather time dilation
  - kink–antikink collision
- [x] Quantum aether: 2D rotating Gross–Pitaevskii, vortex lattice, Feynman rule, imprinting
- [x] Hypothesis scorecard (all suites, threaded) + 12-point requirements table
- [x] In-app research library

## Next
1. **Vortex reconnection** (Baggaley Type II) for filaments. Trefoil untying, as in Kleckner & Irvine.
2. **Compressible medium (D2Q9 lattice Boltzmann).**
   - Bjerknes with retardation: sign flip at kd ~ π/2.
   - Sound radiation from accelerating vortices.
3. **Two-species medium.** A universality-of-c test: do two excitation types share one limiting speed?
4. **Kelvin-wave dispersion measurement.** Fit ω(k) against (Γk²/4π)ln(1/ka).
5. **3D GPE vortex ring.** Speed constant 0.615; reconnection scaling δ ∝ t^{1/2}.
6. **Faddeev–Skyrme Hopfion relaxation.** Topological solitons with a fixed size, unlike fluid knots.
7. **Export.** CSV/JSON of diagnostics; PNG snapshots.
8. **GPU (wgpu) backend.** Filament N-body and lattice Boltzmann, checked against the CPU oracle.
9. **Packaging.** Windows installer, app icon, CI release builds.
