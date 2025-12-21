# M-Patek Automation Script

.PHONY: all build run-node verify bench audit

all: build

# 1. 编译 (Release Mode for Speed)
build:
	@echo "🔧 Building HTP binaries (Optimized)..."
	@cargo build --release

# 2. 运行服务端 (后台运行)
run-node:
	@echo "🚀 Starting Prover Node..."
	@RUST_LOG=info ./target/release/htp-node --dim 4 --seed "M-Patek-Secret"

# 3. 运行客户端进行验证
verify:
	@echo "🔍 Verifying User 'Alice_001'..."
	@./target/release/htp-cli --server 127.0.0.1:4433 verify Alice_001

# 4. 运行基准测试
bench:
	@echo "📊 Running Micro-benchmarks..."
	@cargo bench

# 5. 生成文档 (Internal Use)
doc:
	@cargo doc --no-deps --open
