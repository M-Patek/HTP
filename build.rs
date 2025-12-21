// build.rs
fn main() {
    // [Optimization] Enable native CPU instructions (AVX2/AVX512)
    // allowing the Class Group arithmetic to be vectorized.
    println!("cargo:rustc-env=RUSTFLAGS=-C target-cpu=native");
    
    // 如果有 C/C++ 的底层库 (如 GMP)，在这里链接
    println!("cargo:rustc-link-lib=gmp");
}
