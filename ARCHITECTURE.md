# System Architecture

HTP is designed as a layered library, separating pure mathematics, topological structures, and network protocols.

## 📂 Directory Structure

```text
/
├── src/
│   ├── core/                  # Layer 1: Mathematical Primitives
│   │   ├── algebra/           # Class Group arithmetic (NuCOMP, NUDUPL)
│   │   ├── prime/             # Hash-to-Prime & Miller-Rabin Sieve
│   │   └── affine.rs          # The Affine Tuple (P, Q) & Composition Law
│   │
│   ├── topology/              # Layer 2: Hyper-Tensor Structure
│   │   ├── tensor.rs          # Sparse Tensor storage & Coordinate mapping
│   │   ├── folding.rs         # The Dimensional Folding Algorithm (Recursive)
│   │   └── segment_tree.rs    # Parallel Segment Tree for 1D aggregation
│   │
│   ├── protocol/              # Layer 3: Interaction
│   │   ├── challenge.rs       # Fiat-Shamir Challenge Generation
│   │   ├── prover.rs          # Proof generation & Path extraction
│   │   └── verifier.rs        # Lightweight verification logic
│   │
│   └── ffi/                   # C-bindings for Python/Go integration
│
├── benchmarks/                # Criterion.rs benchmark suites
└── tests/                     # Integration tests & Test Vectors
```

## 🧩 Key Components

### 1. The Algebra Engine (`src/core`)
* **Responsibility:** Implements the group operations in $Cl(\Delta)$.
* **Key Trait:** `GroupElement` which supports `compose`, `inverse`, and `pow`.
* **Optimization:** Uses **NuCOMP** algorithm to perform composition and reduction simultaneously, keeping intermediate coefficients small.

### 2. The Topology Manager (`src/topology`)
* **Responsibility:** Maps user IDs to coordinates $\vec{v} = (x, y, z, w)$.
* **Storage:** Uses a **Sparse Merkle-like structure**. It does not allocate memory for empty tensor cells (defaulting to Identity Affine Tuple).
* **Folding:** Implements the $\Phi$ operator described in `THEORY.md`. It recursively reduces a $d$-dimensional tensor to $d-1$ via Segment Trees.

### 3. The Verifier (`src/protocol`)
* **Design Goal:** Stateless and lightweight.
* **Input:** `GlobalRoot`, `Proof`, `TargetID`.
* **Process:**
    1.  Reconstructs the `AffinePath` from the proof.
    2.  Computes the result of the affine transformation.
    3.  Checks consistency against orthogonal anchors provided in the proof.

## 🔄 Data Flow: Proof Generation

1.  **Request:** User asks proof for ID `12345`.
2.  **Mapping:** `topology` converts `12345` -> `[12, 45, 0, 0]`.
3.  **Challenge:** Hash determines `Challenge Axis = Y`.
4.  **Extraction:**
    * Lock the Tensor state (Reader lock).
    * Extract Segment Tree path for column `[12, *, 0, 0]`.
    * Extract Roots for rows intersecting at `y=45`.
5.  **Serialization:** Package into `Proof` struct.
