# Security Policy

## ⚠️ Experimental Status
**Hyper-Tensor Protocol （HTP)** implements novel cryptographic primitives including Class Groups of Imaginary Quadratic Fields and Non-commutative Affine Accumulators.

While the mathematical proofs are documented in [THEORY.md](THEORY.md), the codebase has **NOT yet undergone a formal external security audit**. 
* **Do not use PHTP for storing high-value assets in production without secondary verification mechanisms.**
* Use at your own risk.

## 📝 Reporting a Vulnerability

**Please DO NOT open public GitHub issues for security vulnerabilities.**

If you discover a security issue (e.g., side-channel leakage, adaptive root attack vector, or memory safety violation), please report it responsibly:

1.  **Email:** `security@phoenix-project.io`
2.  **PGP Key:** `0xDEADBEEF...` (Fingerprint: `...`)
3.  **Response:** We aim to acknowledge receipt within 24 hours and provide a fix timeline within 72 hours.

## 🎯 Threat Model

We assume the following adversary capabilities:

### 1. Cryptographic Assumptions
* **Hidden Order Assumption:** We assume it is computationally infeasible for an adversary to compute the order of the Class Group $Cl(\Delta)$ where $\Delta$ is generated via a high-entropy random beacon.
* **Adaptive Root Assumption:** We assume it is infeasible to find $y$ such that $y^P = x$ for a random $x$ and chosen prime $P$.

### 2. Side-Channel Attacks
* **Timing Attacks:** The core algebraic operations (`composition`, `squaring`) in `src/core/algebra` MUST be constant-time regarding private inputs (witnesses).
* **Memory Access:** We strive to minimize data-dependent memory access patterns in the NuCOMP implementation.

### 3. Trusted Setup
* **None.** PHTP relies on Class Groups, which require no trusted setup (unlike RSA Accumulators). The discriminant $\Delta$ is public and verifiable.

## 📦 Supply Chain Security
* All dependencies are pinned in `Cargo.lock`.
* We use `cargo-audit` in CI to detect vulnerabilities in upstream crates (e.g., `rug`, `gmp`).
