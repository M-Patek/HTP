# HYPER-TENSOR PROTOCOL (HTP): Technical Specification

## 1. Mathematical Preliminaries (数学基础)

### 1.1 Class Group Parameters
* **Discriminant Generation:**
    定义 $\Delta = -M$，其中 $M$ 是满足 $Hash(Seed)$ 之后第一个符合 $M \equiv 3 \pmod 4$ 条件的素数。
    > **原理说明:** $M \equiv 3 \pmod 4 \implies \Delta \equiv 1 \pmod 4$，这对于生成奇数类数 (Odd Class Numbers) 是理想的。
* **Hash-to-Prime:**
    采用“基于 Nonce 的 Hash-and-Test 结合小素数筛法”算法。
    * **输入:** 身份标识 (Identity)
    * **算法:** $SHA256(ID \parallel k) \to \text{Candidate} \to \text{Sieve} \to \text{Miller-Rabin}$。

### 1.2 Non-Commutative Algebra (非交换代数)
* **State Evolution (状态演化):**
    $$S_{t} = S_{t-1}^{P_t} \cdot G^{H(t)}$$
* **Recursive Unrolling (递归展开推导):**
    展示 $W$ (见证人) 和 $R$ (剩余项) 如何分离：
    $$T_n = T_{k-1}^{(\prod_{j=k}^n P_j)} \cdot \dots$$
* **Verification Equation (验证方程):**
    $$W^{P_k} \cdot R \equiv T_{root} \pmod \Delta$$

---

## 2. Affine Structure & Optimization (仿射结构与优化)

### 2.1 The Affine Tuple
定义元组 $\mathcal{A} = (P, Q)$ 作用于状态 $S$ 的方式为：$S^P \cdot Q$。

### 2.2 Composition Law (合成法则)
推导 $\mathcal{A}_1$ 后接 $\mathcal{A}_2$ 的合成结果：
$$\mathcal{A}_{1 \oplus 2} = (P_1 P_2, \quad Q_1^{P_2} \cdot Q_2)$$
> **注意:** 强调其非交换性 (Non-commutativity)，即 $\mathcal{A}_1 \oplus \mathcal{A}_2 \neq \mathcal{A}_2 \oplus \mathcal{A}_1$。

### 2.3 Segment Tree Construction (线段树构建)
描述如何构建一棵树：
* **Leaf (叶子节点):** $\mathcal{A}_i = (P_i, G^{H(i)})$
* **Node (中间节点):** $\mathcal{A}_{left} \oplus \mathcal{A}_{right}$
* **Root (根节点):** 代表整个范围的聚合变换。

---

## 3. Hyper-Tensor Topology (超张量拓扑)

### 3.1 Coordinate Mapping
定义从逻辑索引 $i$ 到向量 $\vec{v}$ 的映射：
$$v_k = (i // L^{k-1}) \pmod L$$

### 3.2 Dimensional Folding (维度折叠算法)
定义张量降维函数 $\Phi$：
$$\Phi(Tensor_d) \to Tensor_{d-1}$$
通过在第一维度上应用线段树聚合来实现。

### 3.3 Orthogonal Anchoring (正交锚定)
解释点 $\vec{v}$ 的“证明”组成部分：
1.  沿 **Challenge Axis (挑战轴)** 的主路径。
2.  在 $\vec{v}$ 处相交的正交轴的根。
3.  **一致性检查:** $\text{Root}(\text{Axis}_1) == \text{Root}(\text{Axis}_2) == \text{GlobalRoot}$。

---

## 4. Protocol Flow (协议流程)

### 4.1 Fiat-Shamir Transformation
定义非交互式挑战生成：
$$Challenge\_Axis = Hash(Global\_Root \parallel User\_ID) \pmod d$$

### 4.2 Verification Algorithm
验证者客户端伪代码：
1.  **Parse Proof:** 解析证明内容。
2.  **Recompute Affine Path:** 计算聚合路径 $\to$ 得到 $(P_{agg}, Q_{agg})$。
3.  **Compute Result:** $Result = W_{local}^{P_{agg}} \cdot Q_{agg}$。
4.  **Assert:** 检查 $Result == Global\_Root$。
