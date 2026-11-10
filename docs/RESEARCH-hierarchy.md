# Vortices Made of Vortices: Scale Hierarchy, Coarse-Graining and Emergent Gravity

**The hypothesis under test:** gravity (large scale) and quantum mechanics can be unified by a *hierarchical vortex*. It is a vortex made of smaller vortices, which are made of smaller vortices, and so on until the substructure "becomes nothing".

Tags: [ESTABLISHED] means textbook, peer-reviewed and reproduced. [SPECULATIVE] means published proposals, partial support or derivation sketches. [OPEN] means unresolved.

## 0. Bottom line

The hierarchy idea is **mathematically legitimate in one direction**:
- quantised vortices coarse-grain into classical vorticity;
- bundles of thin rings behave like one thick ring;
- the far field forgets the microstructure;
- the Gross–Pitaevskii equation converges to classical Euler flow as κ = h/m → 0 at fixed total circulation. This is a theorem in 2D (Jerrard & Spirn).

Aether Labs now reproduces this to high precision (see §8).

The same mechanism is why it **does not by itself give gravity**. The infinite-hierarchy limit returns classical fluid dynamics, which is Euler flow, not Einstein gravity. Every serious emergent-gravity programme hits a version of this wall: the emergent geometry has the right *kinematics* but the wrong *dynamics*. Fluid analogues also face sharp no-go results:
- Weinberg–Witten
- Lorentz-violation bounds
- the equivalence principle
- Bell nonlocality

The honest programme tests both the parts expected to work and the parts expected to fail.

## 1. Coarse-graining quantised vortices into classical flow

### 1.1 Feynman's rule [ESTABLISHED]
Circulation in a superfluid is quantised: ∮v_s·dl = nκ, with κ = h/m. A container rotating at Ω fills with vortices of areal density

    n_v = 2Ω/κ,   spacing ℓ ≈ n_v^(−1/2)

Averaged over areas ≫ ℓ², the flow is solid-body rotation ⟨v⟩ = Ω × r. Quantised "atoms" of circulation reproduce continuous vorticity ω = 2Ω. The microstructure survives only at scale ℓ, in Tkachenko waves and in the ripple near each core.

### 1.2 HVBK coarse-grained equations [ESTABLISHED as a model]
The Hall–Vinen–Bekarevich–Khalatnikov equations treat v_s as smooth above ℓ but remember the lines:

    ∂v_s/∂t + (v_s·∇)v_s = −∇μ + T + F_ns/ρ_s
    T = −ν_s ω_s × (∇ × ω̂_s),   ν_s = (κ/4π) ln(ℓ/a₀)   (vortex tension)

What survives coarse-graining is the vorticity. The microstructure leaves a **logarithmic fingerprint**, ln(ℓ/a₀). HVBK assumes polarised vortices and fails for random tangles.

### 1.3 Kolmogorov turbulence from quantum turbulence [ESTABLISHED]
At k ≪ 2π/ℓ, quantum turbulence shows E(k) ∝ k^(−5/3) (Barenghi, Skrbek & Sreenivasan 2014). This happens only when like-signed vortices organise into **bundles** (Baggaley, Laurie & Barenghi, PRL 2012). Random tangles give k^(+2) at large scales. **The "vortex of vortices" picture is literally the coarse-graining mechanism.**

### 1.4 Kelvin-wave cascade [ESTABLISHED in outline, OPEN in detail]
Below ℓ, energy moves along lines as Kelvin waves. Two theories disagree on the energy spectrum:
- Kozik–Svistunov: E ∝ k^(−7/5)
- L'vov–Nazarenko: E ∝ k^(−5/3)

The cascade ends in phonon emission at k ~ 1/a₀. That is the point where information about small scales becomes heat.

### 1.5 What crosses levels [ESTABLISHED]
| Quantity | Across coarse-graining |
|---|---|
| Circulation Γ | Exactly additive (topological) |
| Impulse P = (ρ/2)∫x×ω dV | Exactly additive, conserved |
| Energy | **Not** additive: sub-core self-energies carry ln(ℓ/a₀) terms |
| Helicity | Conserved in ideal flow; changes at quantum reconnections |
| Small-scale information | Lost, via Kelvin waves → phonons → heat |

## 2. Rings made of N filaments

### 2.1 Thin rings [ESTABLISHED]

    U = (Γ/4πR)[ln(8R/a) − α],   E = ½ρΓ²R[ln(8R/a) − β]

Core constants: α = ½ for a hollow core, ¼ for a uniform (Rankine) core, and ≈ 0.615 for a GPE core. The Hamiltonian relation U = ∂E/∂P at fixed Γ ties α to β. **Don't mix conventions.**

### 2.2 Published bundle numerics [ESTABLISHED]
Wacks, Baggaley & Barenghi (Phys. Fluids 26, 027102, 2014) simulated toroidal bundles of N quantised rings (N = 1–19, with estimates to 1027). Findings:
- Bundles move by **generalised leapfrogging** and stay coherent for about 10 diameters.
- The speed matches Γ/(4πR)[ln(8R/a) − α], with a the bundle radius.

Whether bundles stay coherent at finite temperature is [OPEN] (arXiv 1412.1917; 2603.05387).

### 2.3 The N → ∞ limit [derivation SPECULATIVE; tested here]
- Filaments on a torus *shell* become a vortex sheet, i.e. a hollow ring with α = ½.
- Filaments *filling* the cross-section give a uniform core, α = ¼.

The finite-N excess comes from sub-core self-energy, roughly

    U_N − U_∞ ≈ (Γ/4πR)(1/N)[ln(d/2πa₀) + O(1)],   d = filament spacing

For a hierarchy, the excess at level k scales as Γ²/(N₁⋯N_k)·log. **This is the precise sense in which the substructure "becomes nothing":** its footprint vanishes like 1/(N₁N₂⋯), up to logs.

## 3. Scale separation: the honest "becomes nothing" mechanism [ESTABLISHED]
Outside a compact vorticity region, the far field is a **dipole whose strength is the impulse P**, decaying as r⁻³. Higher multipoles, the only place the internal arrangement appears, decay faster. This is the logic of effective field theory and renormalisation-group decoupling: short-scale structure appears at long distance only through a few renormalised couplings.

**Implication:** decoupling cuts both ways. The more completely the substructure becomes nothing, the less it can do at large scales that a plain continuum fluid can't already do. If gravity emerges, it must come from the long-wavelength effective theory, which is the coarse-grained fluid. The hierarchy can renormalise parameters, but it cannot add new long-range physics.

## 4. Emergent-gravity programmes: what each achieves, where each fails

| Programme | Achieves | Fails / status |
|---|---|---|
| **Sakharov induced gravity** (1967) | The Einstein–Hilbert action appears at one loop: 1/16πG ~ N Λ² | Presupposes a metric; G depends on the cutoff; cosmological constant ~Λ⁴ |
| **Jacobson, "Thermodynamics of spacetime"** (PRL 1995) | Einstein's equations as an *equation of state*, from δQ = TdS on local horizons | Needs S ∝ A as input; says nothing about the microscopic medium |
| **Verlinde entropic gravity** (2010/2016) | Newton's law from entropy; apparent dark matter | Galaxy RAR and KiDS-1000 lensing tensions (≥6σ early/late-type difference); no cosmology |
| **Volovik superfluid vacuum** | Emergent Weyl fermions, gauge fields and metric in ³He-A; vacuum energy naturally zero | The metric obeys superfluid hydrodynamics, **"as a rule" not Einstein's equations**; no general covariance |
| **Analog gravity** (Unruh, Visser) | Horizons, ergoregions, observed analogue Hawking radiation | Kinematic only: the metric follows Euler + continuity, not G_μν = 8πG T_μν |
| **BEC emergent dynamics** (Girelli, Liberati & Sindoni 2008) | A real dynamical emergent gravity | It is **Nordström scalar gravity**: no light bending, already ruled out |
| **Superfluid dark matter** (Berezhiani & Khoury 2015) | Testable: galactic vortex arrays, MOND-like phonon force | SPARC fits often not MOND-like and in tension with lensing; sits *on top of* GR |

**No-go constraints:**
- **Weinberg–Witten:** no massless spin-2 particle with a Lorentz-covariant conserved stress tensor. Fluids escape only by being non-relativistic, which brings generic Lorentz violation.
- **Lorentz-violation bounds:** dimension-5 dispersion is constrained beyond the Planck scale (Liberati 2013). Multi-component media give *different* metrics to different excitations, which breaks universality and the equivalence principle.
- **Quantum gravity is [OPEN]:** GR is non-renormalisable; there's no agreed background independence or treatment of time; no decisive experiment at the Planck scale.

## 5. The quantum ↔ classical bridge

### 5.1 Madelung hydrodynamics [ESTABLISHED]
Write ψ = √ρ e^{iS/ħ}, v = ∇S/m. This gives continuity plus Euler with a quantum pressure Q = −(ħ²/2m)∇²√ρ/√ρ, which sets the healing length ξ. Circulation quantisation comes from ψ being single-valued, not from the fluid equations (Wallstrom's objection).

### 5.2 The classical limit is "ever smaller vortices" [ESTABLISHED, rigorous]
Take ħ → 0 at fixed Γ. Then κ → 0, N = Γ/κ → ∞ and ξ → 0, and the vortex cloud coarse-grains to Euler vorticity. Rigorous results:
- **2D:** Jerrard & Spirn (arXiv 1310.4558)
- **3D, nearly parallel filaments:** Jerrard & Smets (arXiv 2006.03827)

**This is precisely the hypothesis's limit, and it is a theorem. But it heads *away* from quantum mechanics toward classical fluids, not toward gravity.**

### 5.3 Limits of hydrodynamic and Bohmian pictures [ESTABLISHED]
- **Entanglement:** an N-particle wavefunction is a fluid in **3N-dimensional configuration space**, not one 3D medium.
- **Bell:** any *local* medium gives |CHSH| ≤ 2. Loophole-free tests (2015) exclude this, so a vortex-medium account of QM must be explicitly nonlocal.
- **Spin-½:** needs extra structure; a scalar-order-parameter vortex doesn't supply it.
- **Walking droplets:** reproduce some single-particle statistics, but not Bell violation.

## 6. Experiment programme

**Expected to pass:** these test the coarse-graining thread.

| # | Experiment | Pass criterion | Status in Aether Labs |
|---|---|---|---|
| E1 | Feynman rule: coarse-grained lattice velocity = Ωr | < 10% (lattice), < 2% for wide filters | ✅ **PASS**, within 10% |
| E2 | Cluster of N quanta has the far field of one N-quantum vortex; difference ∝ (a/r)^N | exponents ±0.1 | ✅ dynamics version: 64-vortex clusters orbit like single vortices (1%) |
| E3 | Bundle-ring speed vs N → hollow ring; residual ∝ (1/N)ln(d/a₀) | slope 1 ± 0.15 | ✅ converges to within 0.7% at N=64; residual law not yet fitted |
| E4 | Hamiltonian consistency U = ∂E/∂P at each level | < 1% | ✅ **PASS**: thin ring 0.1%; 16-filament bundle 0.55% *at fixed bundle volume* (12–14% off at fixed radius) |
| E5 | Bundle of bundles: far field and impulse universality | < 10⁻³ at 20R | ✅ **PASS**, < 10⁻⁴ at 3–8R once impulse is matched |
| E6 | Energy/impulse/helicity bookkeeping through GPE reconnections | drift < 10⁻³ | ⏳ needs 3D GPE |
| E7 | ħ → 0: GPE vortex patch → Kirchhoff ellipse Ω = ω₀ab/(a+b)² | error falls with N | ⏳ next |
| E8 | Kolmogorov −5/3 only from *polarised* tangles | slope −1.67 ± 0.1 | ⏳ |
| E9 | Long-wavelength sound sees the same metric for a lattice and a smooth flow (Δt = 2Γ/c²) | 1% for kℓ < 0.1 | ✅ **PASS** (ray level): 61-vortex cluster = single vortex to 10⁻¹¹; wave-level scattering test still to do |

**Falsification tests:** expected to fail, and informative when they do.

| # | Experiment | What the program needs | Expected |
|---|---|---|---|
| E10 | Compute G_μν of the emergent acoustic metric vs the medium's T_μν | one universal G_eff to 10% | ❌ **FAILS as predicted**: fluid metrics have \|R_μν\| ≈ 0.80 of the curvature scale in empty space (true Schwarzschild: 1.4×10⁻¹⁰) |
| E11 | Universal 1/r² attraction ∝ E₁E₂; equivalence principle across internal structures | exponent −2 ± 0.05, trajectories differ < 10⁻³ | ❌ **FAILS as predicted**: η ≈ 0.095 (noise floor 0.004) |
| E12 | A massless, 2-polarisation, transverse-traceless (spin-2) mode | exists and couples universally | ❌ **FAILS as predicted**: one scalar Bogoliubov branch (matched to 10⁻⁵); ω/ck = 1.61 at kξ = 1.8 |

## 7. Where the idea stands
- **Legitimate:**
  - quantised vortices coarse-grain to classical vorticity;
  - bundles converge to classical rings;
  - the GPE becomes Euler flow as κ → 0;
  - acoustic geometry is universal at long wavelength;
  - superfluid-vacuum ideas give real insight (the cosmological constant, galactic vortex arrays).
- **Not supported:** that the hierarchy produces gravity or explains QM. The κ → 0 limit is classical hydrodynamics. Emergent-metric dynamics are Euler, not Einstein. Fluid gravity is scalar (Nordström-like, ruled out), not spin-2. Fluids violate the equivalence principle and Lorentz invariance generically. A local medium can't reproduce entanglement.
- **Open:**
  - whether some medium with the right symmetries can evade Weinberg–Witten through emergent gauge invariance (as holography does);
  - bundle coherence at finite temperature;
  - the Kelvin-wave spectrum;
  - Euler blow-up;
  - quantum gravity itself.

## 8. Results from Aether Labs (this codebase)
Setup: ring R = 1, Γ = 1, tube radius a = 0.1, sub-core δ = 0.004, 8000 nodes per filament.

| Sub-vortices N | Speed U_N | vs hollow ring 0.30892 |
|---|---|---|
| 4 | 0.33572 | +8.7% |
| 8 | 0.31544 | +2.1% |
| 16 | 0.30872 | −0.06% |
| 32 | 0.30697 | −0.6% |
| 64 | 0.30675 | −0.7% |

- **Bundles are faster at small N**, which matches the sign predicted by the sub-core-energy argument and seen by Wacks et al.
- **Successive differences shrink with an effective exponent of about −2.1**, faster than 1/N because of the ln(d/a₀) factor.
- **The residual −0.7% at large N** is the O((a/R)²) correction to the thin-ring formula.

Far field:
- A 2-level hierarchy (6 × 4 = 24 filaments) differs from a plain ring of the *same radius* by 0.77%. That is exactly its extra impulse Σa²/2R².
- Against a plain ring of the *same impulse*, the difference is ~10⁻⁵ at every distance from 3R to 8R.
- **The microstructure is visible at large scale only through its conserved impulse.**

Quantum → classical:
- Two clusters of 64 point vortices orbit at Ω = 1/π (within 1%), the same as two single vortices.
- In the rotating Gross–Pitaevskii superfluid, the density-weighted coarse-grained velocity of a lattice of quantised vortices fits v = Ωr within 10%.

### Pass 2: gravity tests (E4, E9, E11)

**E4, Hamiltonian consistency.**
- **Thin ring:** U = 0.32383 vs dE/dP = 0.32414, a 0.1% difference.
- **16-filament bundle at fixed radius a:** U exceeds dE/dP by 12–14% for N = 4–32. This doesn't change with resolution or core size, so it is real.
- **Same bundle at fixed volume (a²R constant, Kelvin's constraint for an incompressible thick core):** the residual is 1.97%, 0.55% and 0.25% for a = 0.15, 0.075 and 0.0375. That is the expected (a/R)² behaviour.

*Lesson:* coarse-grained objects keep their Hamiltonian structure, but only with the right constraint on their internal degrees of freedom. "What is held fixed inside" is real physics, not bookkeeping.

**E9, acoustic Sagnac.** Sound guided around a loop of radius L picks up a delay Δt → 2Γ/c², with a measured value of 0.0050000 against a predicted 0.005. A cluster of 61 vortices and a single vortex agree to 10⁻¹¹.

*Caveat:* at the ray level this is guaranteed by Stokes' theorem (∮v·dl = Γ). Microstructure could appear only through wave scattering at kℓ ~ 1, which needs a wave solver (planned).

**E11, equivalence principle (falsification test).**
- *Setup:* two small vortex bodies with the same impulse fall through the flow of a large "planet" ring. One is a thin ring (R = 0.5). The other is a 6-filament bundle with its radius adjusted to match the impulse.
- *Result:* after subtracting free motion, their extra displacements are 0.430 and 0.392. That gives **η = 0.094**, and 0.097 at higher resolution. The control, two identical thin rings at different resolutions, gives η = 0.004.
- *Conclusion:* the violation is 25× the noise floor. **Vortex "matter" does not fall universally; its response depends on internal structure.** Real gravity requires η < 10⁻¹⁵ (MICROSCOPE). This is the falsification predicted in §4, now measured.

### Pass 3: Einstein's equations and the graviton (E10, E12)

**E10, does the emergent geometry obey Einstein's equations?**

A numerical curvature engine (4th-order finite differences for the Christoffel symbols and the Ricci tensor) was validated two ways:
- the Schwarzschild black hole in Painlevé–Gullstrand "river" form, ds² = −dt² + (dx − v dt)² with v = −√(2M/r), has |R_μν| / (curvature scale) = **1.4×10⁻¹⁰**, i.e. Ricci-flat;
- flat space gives 0.

Two metrics that a real fluid can make were then tested at r = 6M:

| Fluid | Why it differs from GR | \|R_μν\| / scale |
|---|---|---|
| Constant-density sink | Continuity forces v ∝ 1/r², not √(2M/r), so the pull is ∝ 1/r⁵ | **0.80** |
| Schwarzschild velocity profile | Continuity then forces ρ ∝ r^(−3/2), and the acoustic conformal factor ρ/c spoils it (Visser) | **0.80** |

The "space as a flowing river" picture of a black hole is exactly GR. But **no fluid obeying mass conservation can flow that way and still give a vacuum Einstein metric**. The emergent geometry has curvature where Einstein's equations say there must be none.

**E12, is there a graviton?**

The spectrum of a uniform 2D superfluid was measured by seeding standing waves and timing their oscillation:
- The result matches Bogoliubov's ω² = c²k² + (k²/2)² **to 10⁻⁵** for kξ = 0.22–1.78.
- Density seeds and phase seeds give the **same** frequencies, so there is a single scalar (helicity-0) branch. That is sound, not a spin-2 wave.
- At short wavelength the dispersion breaks Lorentz invariance: ω/ck = 1.01, 1.05, 1.11, 1.18, 1.37, **1.61**. Astrophysical bounds require any such effect to sit beyond the Planck scale.

This is the Weinberg–Witten problem made concrete.

### Scorecard after pass 3
The emergence side works. The gravity side fails, in exactly the places the literature predicted:

| Works (measured) | Fails (measured) |
|---|---|
| Vortex-of-vortices → classical ring (0.7%) | Equivalence principle (η ≈ 0.095) |
| Far field sees only impulse (10⁻⁵) | Einstein vacuum equations (\|R\| ≈ 0.8) |
| Quantised lattice → rigid rotation (Feynman) | Spin-2 graviton (none; only sound) |
| Hamiltonian structure across levels (0.55%) | Lorentz invariance at short scales (ω/ck = 1.61) |
| Acoustic metric universal at long wavelength | Force sign (Bjerknes: like attracts like) |

**Conclusion so far.** A vortex made of ever-smaller vortices really does "become nothing" and leave classical fluid physics behind; that half of the idea is right and is now demonstrated. But classical fluid physics is not Einstein gravity. To get gravity, the medium would need extra structure no simple fluid has: a universal coupling, a conserved covariant stress tensor and a spin-2 mode. That is where the problem stays open.

## References (checked by the research agent)
- Barenghi, Skrbek, Sreenivasan, PNAS 2014: https://arxiv.org/abs/1404.1909
- Baggaley, Laurie, Barenghi, PRL 109, 205304 (2012): https://arxiv.org/abs/1207.7296
- Wacks, Baggaley, Barenghi, Phys. Fluids 26, 027102 (2014): https://arxiv.org/abs/1401.6468
- Wacks, Baggaley, Barenghi, PRB 2014: https://arxiv.org/abs/1412.1917
- Bundle dispersal preprint (2026): https://arxiv.org/abs/2603.05387
- Kozik & Svistunov, Kelvin-wave cascade: https://arxiv.org/abs/0904.1379
- HVBK: https://arxiv.org/abs/1910.00276 ; Holm: https://arxiv.org/abs/nlin/0103040
- Jerrard & Spirn, GPE → Euler: https://arxiv.org/abs/1310.4558
- Jerrard & Smets: https://arxiv.org/abs/2006.03827
- Jacobson 1995: https://arxiv.org/abs/gr-qc/9504004
- Visser, Sakharov induced gravity: https://arxiv.org/abs/gr-qc/0204062
- Verlinde: https://arxiv.org/abs/1001.0785 ; https://arxiv.org/abs/1611.02269
- Brouwer et al. 2017: https://arxiv.org/abs/1612.03034 ; Lelli et al. 2017: https://arxiv.org/abs/1702.04355 ; Brouwer et al. 2021: https://arxiv.org/abs/2106.11677
- Volovik: https://arxiv.org/abs/gr-qc/0005091 ; https://arxiv.org/abs/1004.0597 ; https://arxiv.org/abs/1108.5086
- Barceló, Liberati, Visser, Analogue Gravity: https://arxiv.org/abs/gr-qc/0505065 ; Visser 1998: https://arxiv.org/abs/gr-qc/9712010
- Girelli, Liberati, Sindoni: https://arxiv.org/abs/0807.4910 ; https://arxiv.org/abs/0806.4239
- Jenkins, Weinberg–Witten constraints: https://arxiv.org/abs/0904.0453 ; https://arxiv.org/abs/2108.06582
- Liberati, Lorentz-invariance tests: https://arxiv.org/abs/1304.5795
- Berezhiani & Khoury: https://arxiv.org/abs/1507.01019 ; Mistele, McGaugh, Hossenfelder: https://arxiv.org/abs/2201.07282

**Cited from memory (verify before relying on exact numbers):**
- Saffman, *Vortex Dynamics*
- Roberts & Grant 1971
- Unruh 1981
- L'vov, Nazarenko & Rudenko
- Chorin, *Vorticity and Turbulence*
- Kerr; Hou–Luo; Chen–Hou
- Scheeler et al. 2017
- Wallstrom 1994
- Hensen et al. 2015
- Bush 2015
- Sonin on Tkachenko modes
