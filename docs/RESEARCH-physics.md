# Vortex-Aether Physics and History: Research Brief

Purpose: the physics and history behind the hypothesis that matter, charge and forces are vortex structures in one continuous medium. Every claim carries one of three tags:

- [ESTABLISHED]: textbook physics or a replicated experiment.
- [HISTORICAL]: an idea once taken seriously and later abandoned, with the reasons.
- [SPECULATIVE]: a research programme without empirical support as fundamental physics.

The goal is to test the idea against real numbers, not to confirm it.

## 0. Bottom line

1. **What a continuous medium does well [ESTABLISHED].** It reproduces many structures of field theory as analogies, and some are quantitatively exact.
   - Sound in a moving fluid obeys the wave equation of a curved Lorentzian spacetime (Unruh 1981; Visser 1998).
   - Quantized vortices in superfluids carry topological charge (circulation h/m).
   - In 2D, vortices behave like charges in a dual electrodynamics, with the Magnus force playing the role of the Lorentz force.
   - Pulsating spheres in an ideal fluid attract or repel with an exact 1/r² law (Bjerknes).
   - Knotted vortices exist in real fluids (Kleckner & Irvine 2013).
2. **What no medium model has achieved [ESTABLISHED as a negative result].** None reproduces all of the following together:
   - Lorentz invariance that is universal across particle species, to 10⁻¹⁷ or better
   - the correct signs for both gravity and electrostatics
   - stable particles with fixed masses and charges
   - spin-½ and fermion statistics
   - the equivalence principle to 10⁻¹⁵
   - Standard-Model gauge structure and chirality
3. **The serious modern descendants [SPECULATIVE as fundamental physics].** These are Volovik's superfluid-vacuum programme and analog gravity. Superfluid ³He-A shows emergent chiral fermions, gauge fields and an effective metric. Even the proponents present this as emergent *low-energy* physics without Einstein dynamics.

A good simulator turns each of these successes and failures into a measured result.

## 1. Historical vortex aether

### 1.1 Descartes (1644) and Huygens [HISTORICAL]
Descartes filled space with matter swirling in vortices that carried the planets. Gravity was the inward push on coarse matter as fine matter was flung outward. Huygens (1690) built the first quantitative vortex-gravity model. Newton (*Principia*, Book II) raised three objections:

- a vortex medium produces drag and orbital deviations that are not observed
- a viscous vortex gives an orbital period ∝ R², not Kepler's R^{3/2}
- comets cross the supposed vortex flow

### 1.2 Helmholtz's vortex theorems (1858) [ESTABLISHED]
For an inviscid, barotropic fluid under conservative forces:

    ∂ω/∂t + (u·∇)ω = (ω·∇)u        (+ ν∇²ω if viscous)

Consequences:
1. Vortex lines move with the fluid ("frozen in").
2. The strength Γ = ∮u·dl of a vortex tube is the same along its length and constant in time. Kelvin's circulation theorem (1869) states the same result.
3. Vortex tubes cannot end in the fluid. They close into rings or knots, or end on boundaries.

The velocity follows from the vorticity by the Biot–Savart law:

    u(x) = (1/4π) ∫ ω(x′) × (x − x′)/|x − x′|³ d³x′

So filaments interact like current-carrying wires, but with the opposite sign: parallel vortices orbit each other instead of attracting.

### 1.3 Kelvin's vortex atom (1867) and Tait's knot tables [HISTORICAL]
Thomson ("On Vortex Atoms", Proc. R. Soc. Edinburgh 6, 1867) proposed that atoms are knotted vortex rings in a perfect-fluid aether:

- the knot type is permanent, which would explain why elements never change
- vibration modes of the knot would explain spectral lines

Tait tabulated knots up to 10 crossings, which founded knot theory. J.J. Thomson (1883) analysed linked rings.

**Why it was abandoned:**
- Kelvin admitted the model could not explain inertia or gravitation.
- Thin rings turned out to be unstable (Widnall 1974).
- After the electron was discovered (1897), J.J. Thomson moved on to other atomic models.
- Quantum mechanics explained atomic spectra directly.

Standard history: Kragh, *Physics in Perspective* 4, 265 (2002).

### 1.4 Maxwell's molecular vortices (1861–62) [HISTORICAL, with a lasting legacy]
The model, from "On Physical Lines of Force":

- Magnetic field lines are the axes of rotating cells in the medium.
- "Idle wheel" particles sit between cells, and their motion is electric current.
- The cells are elastic, so an electric force *displaces* the idle wheels. The rate of change of that displacement is the **displacement current** ∂D/∂t.
- The elastic medium carries transverse waves at v = 1/√(με). This came out close to the measured speed of light.

In 1865 Maxwell dropped the mechanism and kept the equations. The lesson is that a mechanical model can be scaffolding for correct field equations. A modern fluid reconstruction is Brady & Anderson, arXiv:1502.05926 [SPECULATIVE, unrefereed].

### 1.5 The Bjerknes pulsating spheres (1875–1906) [ESTABLISHED as fluid dynamics; HISTORICAL as a theory of forces]
A sphere whose volume is V₁(t) acts as a point source with q₁ = dV₁/dt:

- φ = −q₁/(4πr)
- u = q₁/(4πr²)
- ∇p = −ρ ∂u/∂t

The force on sphere 2 is F = −V₂∇p. Averaged over a cycle:

    ⟨F⟩ = − (ρ / 4π d²) ⟨V̇₁ V̇₂⟩ r̂₁₂ = − ρ ω² ΔV₁ ΔV₂ cos(φ₁−φ₂) / (8π d²)

- **In phase → attraction, ∝ 1/d². Antiphase → repulsion.**
- **The sign is inverted relative to Coulomb's law:** "like charges" attract.

This is the same physics as the secondary Bjerknes force between bubbles (Crum 1975). With compressibility, phase lag can flip the sign at kd ~ O(1).

**As a theory of gravity**, it fails for several reasons:
- (a) every body in the universe would have to pulsate in phase
- (b) it needs an incompressible medium, or retardation spoils the result
- (c) Maxwell's objection: it requires continual creation and destruction of aether
- (d) the force scales with ρV̇₁V̇₂, not with inertial mass, so the equivalence principle would need fine-tuning

It cannot also give electrostatics, because the sign is wrong.

### 1.6 Hicks, FitzGerald, Larmor, Lorentz [HISTORICAL]
- **Hicks** (1880s–1890s) studied vortex and pulsating-body interactions and the vortex-sponge aether. He also derived the Hicks (Squire–Long / Bragg–Hawthorne) equation.
- **FitzGerald** (1889) proposed length contraction to explain the Michelson–Morley result.
- **Larmor** (1897, 1900) derived the full Lorentz transformation from a rotationally elastic aether.
- **Lorentz and Poincaré** (1904–05) completed Lorentz Ether Theory. It is empirically equivalent to special relativity because the preferred frame cannot be detected. Einstein dropped that frame as superfluous.

### 1.7 Why the vortex aether was abandoned [ESTABLISHED]
1. **Michelson–Morley (1887):** the expected shift was about 0.4 fringe, and the observed shift was under 0.02. Modern bounds are far tighter.
2. **Special relativity:** Lorentz invariance is a symmetry of all physics, so a rest frame for the medium plays no role.
3. **Stability:** rings and knots stretch, reconnect and untie. Even in an ideal fluid nothing fixes their size, so nothing fixes their "mass".
4. **Quantization:** classical continuum mechanics could not explain discrete spectra, ħ, spin or fermions.
5. **Gravity:** the models had the wrong sign, needed fine-tuning, or predicted drag.

## 2. Modern descendants

### 2.1 Analog gravity: Unruh (1981), Visser (1998) [ESTABLISHED]
Linearised sound φ₁ in a barotropic, inviscid, irrotational flow satisfies (1/√−g) ∂_μ(√−g g^{μν} ∂_ν φ₁) = 0 with:

    ds² = (ρ/c) [ −c² dt² + (dx − v dt)·(dx − v dt) ]

Properties:
- A horizon forms wherever the normal component of the flow reaches c.
- A radial sink gives the Painlevé–Gullstrand form of the Schwarzschild metric.
- The analog Hawking temperature is T_H = ħκ/(2πk_B).
- Rays follow null geodesics. The "draining bathtub" flow has an ergosphere.

Experiments:
- Weinfurtner et al. PRL 106, 021302 (2011): water waves
- Steinhauer, Nat. Phys. 12, 959 (2016): BEC
- Muñoz de Nova et al., Nature 569, 688 (2019): thermal spectrum

**Limits:**
- The metric is kinematic. The fluid's motion sets it, not mass through Einstein's equations.
- It is valid only at long wavelengths.
- Different excitations generally see different metrics.

Review: Barceló, Liberati & Visser, Living Rev. Relativ. 8, 12 (2005).

### 2.2 Emergent Lorentz invariance [ESTABLISHED for a single species; universality is OPEN]
The phonon equation is Lorentz invariant with c = c_sound. Frank (1949) showed that a moving screw dislocation contracts by γ, and its energy goes as γ. The same holds for sine-Gordon kinks. **This is the honest mechanism by which an aether gives a null Michelson–Morley result:** rulers, clocks and signals are all excitations of one relativistic wave equation.

In a BEC, the Bogoliubov dispersion ω² = c²k² + (ħk²/2m)² breaks Lorentz invariance at kξ ~ 1. That acts as a built-in "Planck scale".

**Universality problem:** different species generically have different limiting speeds. Radiative corrections spread Lorentz violation from short distances into everyday physics (Collins et al., PRL 93, 191301, 2004).

### 2.3 Superfluid vacuum: Volovik [SPECULATIVE as fundamental physics]
Volovik (*The Universe in a Helium Droplet*, 2003; Phys. Rep. 351, 195, 2001) shows that ³He-A has:

- emergent chiral Weyl fermions
- effective gauge fields
- an effective metric
- anomalies

These are protected by momentum-space topology. In equilibrium the vacuum energy cancels exactly.

Caveats:
- No diffeomorphism invariance and no Einstein dynamics.
- The emergent speed of light differs between sectors.
- No distinctive prediction has been confirmed.

### 2.4 Quantized vortices, the Gross–Pitaevskii equation, lattices, reconnection [ESTABLISHED]
The Gross–Pitaevskii equation in a rotating frame:

    iħ ∂ψ/∂t = [ −ħ²∇²/2m + V + g|ψ|² − Ω L_z ] ψ

Key results:
- **Madelung form:** ψ = √n e^{iθ}, superfluid velocity v_s = (ħ/m)∇θ.
- **Quantized circulation:** ∮v_s·dl = n h/m. For ⁴He, κ = 9.97×10⁻⁸ m²/s.
- **Feynman's rule:** vortex density n_v = 2mΩ/h. The vortices form a triangular (Abrikosov) lattice. Abo-Shaeer et al. (Science 292, 476, 2001) saw more than 100.
- **Energy per unit length:** E/L ≈ (ρκ²/4π) ln(b/ξ). So doubly quantized vortices split into two.
- **Reconnection** has been observed (Bewley et al., PNAS 105, 13707, 2008). The separation follows δ(t) ≈ A√(κ|t−t₀|). So quantized "vortex atoms" can change their topology.

### 2.5 Knotted vortices and helicity [ESTABLISHED]
Helicity:

    H = ∫ u·ω d³x = Σ_{i≠j} Lk_ij Γ_iΓ_j + Σ_i (Wr_i + Tw_i) Γ_i²

It is conserved in ideal flow (Moffatt 1969). Experiments:

- **Kleckner & Irvine** (Nat. Phys. 9, 253, 2013): trefoil knots in water stretched, reconnected and broke into unknotted rings. No knot survived.
- **Scheeler et al.** (PNAS 111, 15350, 2014): reconnection conserves helicity by converting linking into coiling.
- **Kleckner, Kauffman & Irvine** (2016): superfluid knots untie through a cascade of reconnections.

### 2.6 Knot solitons: Skyrmions, Hopfions, Faddeev–Niemi [SPECULATIVE as particle physics]
The Faddeev–Skyrme energy:

    E = ∫ [(∂n)² + κ(n·∂ᵢn×∂ⱼn)²] d³x

Properties:
- The Hopf charge Q is conserved, and E ≥ C·Q^{3/4}.
- The quartic term gives solitons a fixed size, which Kelvin's vortex atoms lacked.
- Battye & Sutcliffe (PRL 81, 4798, 1998): Q = 7 is a trefoil.
- Skyrmions quantize as fermions (Finkelstein–Rubinstein; Witten). This is the most credible route to spin-½ from a continuum field.

These are relativistic field theories, not fluids.

### 2.7 Kelvin waves [ESTABLISHED]
For ka ≪ 1:

    ω(k) ≈ (κk²/4π)[ln(1/(ka)) + O(1)]

The waves counter-rotate and feed the Kelvin-wave cascade. The dispersion is quadratic, so it is particle-like, not light-like.

### 2.8 Magnus force and the Lorentz analogy [ESTABLISHED]
The Magnus force is f_M = ρ_s κ × (v_L − v_s) (Thouless, Ao & Niu 1996). Compare it with F = qv×B.

**2D particle–vortex duality:**
- vortices are charges
- the Magnus force plays the role of the Lorentz force
- the phonon plays the role of the photon
- vortices interact through a logarithmic (2D Coulomb) law

The duality is exact only in 2+1 dimensions.

### 2.9 Vortex-ring dynamics [ESTABLISHED]
Thin-ring speed:

    U = (Γ/4πR) [ ln(8R/a) − β ]

| Core type | β |
|---|---|
| Solid (uniform vorticity) | 1/4 |
| Hollow | 1/2 |
| GPE quantized ring | ≈ 0.615 (Roberts & Grant 1971) |

For the solid core:
- E = ½ρΓ²R[ln(8R/a) − 7/4]
- P = ρπΓR²

**Big rings are slower.** Adding energy makes a ring larger and slower, which is not how particles behave.

Coaxial rings leapfrog:
- the 2D threshold is α > 3 − 2√2 ≈ 0.172 (Love 1894)
- the motion is stable for α ≳ 0.382 (Acheson 2000)

## 3. What any aether/vortex model must reproduce

| # | Requirement | Observed bound | What the medium must show |
|---|---|---|---|
| 1 | Light-speed isotropy | Δc/c ≲ 10⁻¹⁷ (Herrmann et al. 2009) | rulers/clocks contract/dilate with signals |
| 2 | Universal limiting speed | species agree to 10⁻¹⁵–10⁻²² | all excitations share one metric |
| 3 | No light dispersion | Fermi-LAT GRB 090510: beyond E_Planck | healing length below the Planck length |
| 4 | Inverse-square laws | Coulomb exponent to 10⁻¹⁶ | F ∝ 1/r^{2+ε}, ε → 0 |
| 5 | Correct signs | like charges repel, masses attract | both signs from different mechanisms |
| 6 | Charge quantization | \|q_e+q_p\| ≲ 10⁻²¹ e | topological charge survives reconnection |
| 7 | Stable particles, fixed mass | proton > 10³⁴ yr | soliton size fixed by the dynamics |
| 8 | Spin-½, fermions | 4π neutron interferometry | configuration space with π₁ = ℤ₂ |
| 9 | Equivalence principle | η ≲ 10⁻¹⁵ (MICROSCOPE 2022) | coupling ∝ inertial mass |
| 10 | E² = p²c² + m²c⁴ | all particles | moving soliton E = γE₀ |
| 11 | No drag | orbits stable for Gyr | ideal medium (d'Alembert) |
| 12 | Einstein gravity, spin-2 waves | light bending 1.75″, GW speed = c | metric sourced like GR |

## 4. Simulation experiments, ordered easiest to hardest

**E1. Point vortices / 2D Coulomb**
- Like pair: Ω = (Γ₁+Γ₂)/(2πd²)
- Opposite pair: U = Γ/(2πd)
- H and impulses conserved

**E2. Ring speed vs Kelvin**
- Within ~2%
- Bigger rings slower

**E3. Bjerknes**
- Slope −2.00 ± 0.02
- In-phase attracts, antiphase repels
- cos Δφ dependence
- Independent of mass

**E4. Sine-Gordon emergent Lorentz contraction**
- w·γ/w₀ = 1 ± 1% up to 0.9c
- Breather time dilation

**E5. Leapfrogging**
- Periodic passes
- Impulse conserved

**E6. Kelvin waves**
- ω ∝ k² (with a log correction)
- Counter-rotation

**E7. Acoustic metric**
- r_H = A/c and r_E = √(A²+B²)/c
- Capture and frame dragging
- Show that the metric is not sourced by mass

**E8. 2D GPE**
- Integer winding
- Dipole speed ≈ ħ/(md)
- Precession in a trap

**E9. Rotating BEC lattice**
- N_v within 10% of Feynman's rule
- Triangular coordination

**E10. 3D reconnection**
- δ(t) ∝ |t − t₀|^{1/2}

**E11. Trefoil untying**
- Helicity bookkeeping
- With no reconnection, the knot type never changes

**E12. Hopfion relaxation**
- Q stays an integer
- E ∝ Q^{3/4}
- Fixed soliton size

## 5. Assessment

The historical vortex models were serious science and left real legacies: knot theory, displacement current, the Lorentz transformations and analog gravity. They were abandoned for concrete, quantitative reasons (§1.7).

A modern single-medium picture is coherent if it is built like Volovik's: a quantum medium whose low-energy excitations are relativistic, with topology (not fluid knots) protecting particles. It still has to deliver:

- a universal c
- Einstein gravity
- correct force signs
- spin-½
- the equivalence principle

None of these is solved within a fluid picture.

## References (checked by the research agent unless noted)
1. https://en.wikipedia.org/wiki/Vortex_theory_of_the_atom
2. https://en.wikipedia.org/wiki/Vortex_ring
3. https://en.wikipedia.org/wiki/On_Physical_Lines_of_Force
4. https://galileo-unbound.blog/2025/03/05/maxwellian-steampunk-and-the-origins-of-maxwells-equations/
5. https://en.wikipedia.org/wiki/Bjerknes_force
6. https://mathshistory.st-andrews.ac.uk/Biographies/Bjerknes_Carl
7. https://en.wikipedia.org/wiki/Mechanical_explanations_of_gravitation
8. https://en.wikipedia.org/wiki/Lorentz_ether_theory
9. https://arxiv.org/pdf/1409.1204
10. Unruh 1981: https://personal.lse.ac.uk/robert49/teaching/partiii/2020-2021/pdf/Unruh1981a.pdf
11. Visser 1998: https://arxiv.org/abs/gr-qc/9712010
12. Barceló, Liberati, Visser 2005: https://arxiv.org/abs/gr-qc/0505065
13. Volovik 2001: https://arxiv.org/abs/gr-qc/0005091
14. Steinhauer 2016: https://arxiv.org/pdf/1510.00621.pdf
15. https://phys.technion.ac.il/en/about/research-bits/steinhauer-nature-may2019
16. https://en.wikipedia.org/wiki/Quantum_vortex
17. Abo-Shaeer et al. 2001: https://arxiv.org/abs/cond-mat/0108195
18. Bewley et al. 2008: https://pmc.ncbi.nlm.nih.gov/articles/PMC2544517
19. Kleckner & Irvine 2013: https://news.uchicago.edu/story/vortex-loops-could-untie-knotty-physics-problems ; https://arxiv.org/pdf/1310.3321
20. Scheeler et al. 2014: https://arxiv.org/pdf/1404.6513
21. Faddeev & Niemi 1997: https://arxiv.org/abs/hep-th/9610193
22. Battye & Sutcliffe 1998: https://arxiv.org/abs/hep-th/9808129
23. Thouless, Ao, Niu 1996: https://arxiv.org/pdf/cond-mat/9608089
24. Moroz et al.: https://arxiv.org/pdf/1803.10934
25. Herrmann et al. 2009: https://arxiv.org/pdf/1002.1284
26. Brady & Anderson: https://arxiv.org/abs/1502.05926

**Cited, not fetched:**
- Moffatt 1969
- Widnall et al. 1974
- Roberts & Grant 1971
- Acheson 2000
- Frank 1949
- Weinfurtner et al. 2011
- Kleckner, Kauffman & Irvine 2016
- Collins et al. 2004
- Fermi-LAT 2009
- MICROSCOPE 2022
- Finkelstein & Rubinstein 1968
- Lamb, *Hydrodynamics*
- Kragh 2002

Check the specific numerical thresholds against the originals before hard-coding them.
