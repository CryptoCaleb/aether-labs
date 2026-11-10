# Numerical Methods for a Real-Time Vortex/Aether Simulator

This brief was first written for a browser/three.js stack. Aether Labs was later rebuilt as native Rust (egui + rayon), so the TypeScript and WebGL specifics below are reworded as Rust and CPU or wgpu equivalents. The numerical content is unchanged.

Legend:
- [n] = reference number in the list at the end
- *(derived)* = derived by the research agent; pin it with a unit test

## 0. Build order (recommended)

| Priority | Method | Why | Rough real-time size |
|---|---|---|---|
| 1 | Vortex filaments: regularised Biot–Savart, RK4, resampling, reconnection | Closest to "matter = vortex structure"; exact tests exist | Direct O(N²): ~2k nodes multi-threaded (f64, rayon) |
| 2 | 2D grid fluid (Stam/Harris) with vorticity confinement | Cheap visual "medium" | 512² on GPU (wgpu compute) |
| 3 | Bjerknes forces: analytic plus linear-acoustics FDTD | Linear fields suffice; exact formula to test against | 2D 512², or 3D 128³ |
| 4 | 2D Gross–Pitaevskii (split-step FFT) | Quantised vortices, Feynman rule | 256² CPU (rustfft), 512²+ GPU |
| 5 | D2Q9 lattice Boltzmann | Compressible medium with real sound: Bjerknes plus acoustic gravity | 512²–1024² GPU |
| 6 | Vortons, Barnes–Hut / FMM | Scale-up | 10⁴–10⁵ elements |

## 1. Vortex filament method

### 1.1 Biot–Savart on polygonal filaments

Exact straight-segment formula, with r₁ = x − A, r₂ = x − B, r₀ = B − A:

    u = (Γ/4π) · (r₁ × r₂)/|r₁ × r₂|² · r₀·(r₁/|r₁| − r₂/|r₂|)

Rosenhead–Moore (RM) regularisation: add δ²|r₀|² to the denominator and replace |r| with √(|r|² + δ²). This is exactly what `segment_velocity` in `aether-core` implements. Under the RM kernel a ring moves at

    U_RM = (Γ/4πR)[ln(8R/δ) − 1]   [4]

### 1.2 Ring self-induced velocity: the main validation

U = (Γ/4πR)[ln(8R/a) − C]

| Core | C | Equivalent RM δ |
|---|---|---|
| Solid (Kelvin) | 1/4 | δ = a·e^{−3/4} ≈ 0.472a |
| Hollow | 1/2 | δ = a·e^{−1/2} |
| GPE quantised (a = ξ) | 0.615 | δ = ξ·e^{−0.385} |

General Saffman correction *(derived)*: C = 1/2 − (4π²/Γ²)∫₀ᵃ r v_θ² dr.

**Aether Labs result:** the discretised ring (N = 400–600, δ = 0.05–0.1) matches U_RM to better than 1%.

### 1.3 Local induction approximation (LIA) and the local/non-local split

LIA:

    dγ/dt = β κ b̂,   β = (Γ/4π) ln(c R_c / a)

Discrete curvature binormal:

    (κb)_i = 2 e_{i−1} × e_i / (|e_{i−1}||e_i| + e_{i−1}·e_i)

Baggaley–Barenghi desingularised Biot–Savart [7, 8]:

    ds_i/dt = (Γ/4π) ln(√(ℓ_i ℓ_{i+1})/a) s′ × s″ + Σ (non-adjacent segments)

LIA costs O(N). It reproduces Kelvin waves and helices, but separate rings don't interact. Use it as a fast preview mode.

### 1.4 Time integration and resampling

Time step:
- RK4.
- Kelvin-wave stiffness scales as ℓ⁻². Keep the maximum displacement per step below 0.2 ℓ_min.

Resampling:
- Keep segment lengths in [δ/2, δ] (or around a target spacing).
- Split long segments with a Catmull–Rom midpoint: 0.5625(p₁+p₂) − 0.0625(p₀+p₃).
- Merge segments that are too short.

### 1.5 Reconnection heuristics [7]

- **Type I:** reconnect any two non-adjacent nodes closer than Δ = δ/2 by swapping their successors.
- **Type II:** the same, but only when total length decreases. This is the most robust choice and the easiest to test.
- **Guard:** skip near-parallel tangents (ŝ_i′·ŝ_j′ > 0.965) and nearest neighbours.
- **Clean-up:** delete tiny loops and log the energy and impulse they carried.

Type II pseudocode:

    for each candidate pair (i, j) from a hash grid with cell size Δ:
        skip if adjacent, or tangents nearly parallel
        before = |i → next(i)| + |j → next(j)|
        after  = |i → next(j)| + |j → next(i)|
        if after < before: swap next(i), next(j)

### 1.6 Acceleration (Baggaley–Barenghi tree [8])

- Build an octree of summed segment vectors and vorticity centres.
- Opening criterion: w/(d − ζ) < θ_max, with θ_max = 0.4 (≤ 0.25% velocity error at 0.7).
- Cost: O(N log N).
- Closed loops have zero monopole moment, so add dipole terms if accuracy suffers.

## 2. Vortex particle method (vortons) [9]

Regularised Biot–Savart:

    u(x) = Σ K_σ(x − x_p) × α_p,   K_σ(r) = −q(|r|/σ) r/(4π|r|³)

Kernels:
- Gaussian: q = erf(ρ/√2) − √(2/π) ρ e^{−ρ²/2}
- Winckelmans–Leonard: q = ρ³(ρ² + 5/2)/(ρ² + 1)^{5/2}

Stretching:
- Classic form: dα/dt = (α·∇)u.
- Transposed form: uses (∇u)ᵀα, which conserves total vorticity better.

Viscosity by particle strength exchange (PSE) *(derived)*:

    dα_p/dt = (2ν/σ²) Σ_q (V_p α_q − V_q α_p) η_σ(x_p − x_q)

Divergence control: Pedrizzetti relaxation.

Prefer vortons for turbulent breakup. Prefer filaments for clean topology.

## 3. Grid fluid (Stam/Harris) [10, 11]

Each step:

    advect (semi-Lagrangian) → forces → project (solve ∇²p = ∇·u, then u −= ∇p)

Pressure solve:
- Jacobi update: x_new = (x_L + x_R + x_B + x_T + α b)/β, with α = −Δx², β = 4.
- Use 20–80 Jacobi iterations, or multigrid, which reaches the same residual in 5–10% of the sweeps.

Vorticity confinement:

    f = ε h (N × ω),   N = ∇|ω| / |∇|ω||

A MAC (staggered) grid gives exact discrete div/grad.

## 4. Lattice Boltzmann (D2Q9 / D3Q19) [13]

Collision–streaming update:

    f_i(x + e_i, t + 1) = f_i + (f_i^eq − f_i)/τ
    f_i^eq = w_i ρ [1 + 3e·u + 4.5(e·u)² − 1.5u²]

- D2Q9 weights: 4/9, 1/9, 1/36.
- Sound speed c_s² = 1/3, viscosity ν = c_s²(τ − ½).
- It is compressible, so sound propagates automatically. Keep Ma ≲ 0.1–0.2.

Practical notes:
- Use sponge layers at the boundaries.
- Model a pulsating body as a mass-source term.
- Measure forces by momentum exchange, averaged over whole periods.

## 5. Gross–Pitaevskii [16, 17]

    i∂ψ/∂t = [−½∇² + V + g|ψ|² − ΩL_z]ψ,   ξ = 1/√(2gn)

Grid spacing: dx ≲ ξ/2.

Split-step algorithm:
1. Half step of the potential and nonlinearity.
2. Kinetic step in k-space.
3. Half step of the potential and nonlinearity.

The rotation term L_z is handled by ADI: T_x = ½p_x² + Ωy p_x, applied with an FFT along x at fixed y; T_y = ½p_y² − Ωx p_y, applied with an FFT along y. **This is exactly what `aether-core::gpe` does.**

For the ground state, use imaginary time with renormalisation each step. Seed with phase noise to break symmetry.

Validation:
- n_v = Ω/π (Feynman). **Aether Labs result: passes within 15%.**
- Ring speed κ/(4πR)[ln(8R/ξ) − 0.615].

## 6. Bjerknes forces

Analytic force (incompressible):

    F₂ = −(ρ/4π) ⟨V̇₁V̇₂⟩ r̂/d²  =  −ρω² v₁ v₂ cos Δφ / (8π d²)

Note the 8π: the factor ½ comes from time-averaging.

Retarded (compressible) form *(derived)*:

    F_r = −(ρω² v₁ v₂ / (8πd²)) [cos(Δφ + kd) + kd sin(Δφ + kd)]

The sign reverses near kd ~ π/2.

In 2D (cylinders) the law is 1/d: F = −(ρ/2π)⟨Ȧ₁Ȧ₂⟩ r̂/d. Use this when validating a 2D lattice Boltzmann run.

Simplest accurate solver: linear acoustics FDTD (Yee leapfrog).
- Source term q_src inside each body.
- Absorbing (PML) boundaries.
- Average over whole periods.
- Subtract each body's self-field.

## 7. Architecture (as implemented in Rust)

    crates/aether-core   pure physics (f64, rayon); no GUI dependencies; every model ships `checks()`
    crates/aether-lab    eframe/egui app: experiments, 3D/2D views, plots, scorecard, library

Design rules:
- Deterministic seeded RNG.
- Validation tests run in `cargo test`.
- GPU (wgpu compute) backends must be checked against the CPU f64 results ("oracle").

Diagnostics:
- impulse P = ½Γ ∮ x × dl (the best health indicator)
- regularised energy (O(N²))
- Gauss linking number and writhe
- total length
- reconnection count
- GPE norm and vortex winding

## 8. Performance estimates (O(N²) Biot–Savart)

- A regularised segment interaction costs about 40 flops.
- One modern CPU core manages about 200–400 M interactions/s in Rust f64. With 8–16 threads: N ≈ 2–4k nodes at 60 fps (RK4).
- A GPU manages N ≈ 10–30k.
- A tree code at θ = 0.4–0.7 manages 20–50k nodes interactively.

## References
1. Weißmann & Pinkall, "Filament-based smoke…", ACM TOG 29(4) 2010: https://history.siggraph.org/?p=105936
2. Weißmann PhD thesis: https://diglib.eg.org/handle/10.2312/8252
3. TU Berlin F12: https://www3.math.tu-berlin.de/geometrie/f12/
4. van Garrel, core and curvature corrections: https://ar5iv.arxiv.org/html/1204.2699
5. Helical vortex (e^{−3/4}): https://arxiv.org/pdf/1508.00272
6. Zuccher & Caliari 2021: https://zucchers.github.io/downloads/ZC_JPA2021.pdf
7. Baggaley, reconnection models: https://ar5iv.arxiv.org/html/1109.4409
8. Baggaley & Barenghi, tree method: https://ar5iv.arxiv.org/html/1108.1119
9. Alvarez & Ning, VPM: https://arxiv.org/abs/2206.03658
10. Harris, GPU Gems ch. 38: https://developer.nvidia.com/gpugems/GPUGems/gpugems_ch38.html
11. Dobryakov WebGL fluid: https://github.com/PavelDoGreat/WebGL-Fluid-Simulation
13. LBM: https://en.wikipedia.org/wiki/Lattice_Boltzmann_methods
16. Mingarelli & Barnett, infinite vortex lattices: https://ar5iv.labs.arxiv.org/html/1602.05624
17. Rotating BEC codes and ADI: https://arxiv.org/pdf/1906.06327 ; https://arxiv.org/pdf/cond-mat/0609678
19. wgsl-fft: https://docs.rs/wgsl-fft
20. Crum's formula: https://ar5iv.labs.arxiv.org/html/1510.06866
21. Ida, Bjerknes sign reversal: https://ar5iv.labs.arxiv.org/html/physics/0109005
