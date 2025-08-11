# PowerShell 版本预质量检查脚本
$ErrorActionPreference = 'Stop'

Write-Host "[INFO] Running pre-quality checks..."
Write-Host "================================="

# 检查 Rust 工具链
Write-Host "[INFO] Checking Rust toolchain..."
rustc --version
cargo --version

# 格式检查
Write-Host "[INFO] Checking code format..."
try {
    cargo fmt --check
} catch {
    Write-Host "[ERROR] Code format issues found. Please run 'cargo fmt' to fix."
    exit 1
}

# Clippy 检查
Write-Host "[INFO] Running clippy checks..."
try {
    cargo clippy --all-targets --all-features -- -D warnings
} catch {
    Write-Host "[ERROR] Clippy warnings found. Please fix them first."
    exit 1
}

# 构建检查
Write-Host "[DEBUG] Building project..."
try {
    cargo check --all-targets --all-features
} catch {
    Write-Host "[ERROR] Build failed. Please fix compilation errors."
    exit 1
}

# 测试检查
Write-Host "[TEST] Running tests..."
try {
    cargo test --all-features
} catch {
    Write-Host "[ERROR] Tests failed. Please fix the failing tests."
    exit 1
}

# 安全审计
Write-Host "[DEBUG] Running security audit..."
if (Get-Command cargo-audit -ErrorAction SilentlyContinue) {
    try {
        cargo audit
    } catch {
        Write-Host "[WARNING] Security vulnerabilities found. Please review and address them."
        exit 1
    }
} else {
    Write-Host "[WARNING] cargo-audit not installed. Install with: cargo install cargo-audit"
}

# 文档检查
Write-Host "[DEBUG] Checking documentation build..."
try {
    cargo doc --no-deps --all-features
} catch {
    Write-Host "[ERROR] Documentation build failed."
    exit 1
}

Write-Host "[OK] All pre-quality checks passed!"
Write-Host "================================="