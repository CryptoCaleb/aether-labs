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

## Done (v0.2): scale hierarchy ("vortex of vortices")
- [x] Hierarchical bundles: rings made of N filaments, made of M filaments… (rotation-minimising frames, twist)
- [x] N → ∞ limit converges to Kelvin's hollow ring (0.7% at N=64)
- [x] Far field blind to microstructure except through impulse (10⁻⁵ when impulse is matched)
- [x] Point-vortex clusters orbit like single vortices (quantum → classical, 2D)
- [x] Coarse-grained superfluid lattice velocity = rigid rotation Ωr (Feynman)
- [x] Research brief: docs/RESEARCH-hierarchy.md (coarse-graining, emergent-gravity programmes, no-go theorems)

## Done (v0.3): gravity tests, pass 2
- [x] E4 Hamiltonian consistency: thin ring 0.1%; bundles pass at fixed volume and fail at fixed radius
- [x] E9 acoustic Sagnac universality (ray level)
- [x] E11 equivalence principle: **fails**, η ≈ 0.095 vs a noise floor of 0.004

## Done (v0.4): gravity tests, pass 3
- [x] Numerical curvature engine (Christoffel/Ricci), validated: Schwarzschild Ricci-flat to 1.4×10⁻¹⁰
- [x] E10 fluid-made metrics violate Einstein vacuum equations (|R_μν| ≈ 0.8 of scale)
- [x] E12 Bogoliubov spectrum measured to 10⁻⁵: a single scalar branch, no spin-2 mode, Lorentz violation at kξ ~ 1

## Next: the unification programme (see docs/RESEARCH-hierarchy.md §6)
Passing side (emergence):
- E7 ħ → 0: GPE vortex patch → classical Kirchhoff ellipse
- E9b wave-level test: sound scattering off a vortex lattice vs a smooth flow (kℓ sweep)
- E3 fit of the (1/N)ln(d/a₀) residual

Falsification side (gravity):
- E12b Tkachenko modes of a vortex lattice (ω ∝ k² expected): still no spin-2?
- E11b universal attraction: force law between two vortex bodies vs 1/r²

## Earlier backlog
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
