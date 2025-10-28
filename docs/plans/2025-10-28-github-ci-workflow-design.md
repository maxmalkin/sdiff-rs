# GitHub CI Workflow Design

**Date:** 2025-10-28
**Repository:** maxmalkin/sdiff
**Status:** Approved

## Overview

This document describes the GitHub Actions CI workflow for the SDIFF project. The workflow provides automated testing, linting, formatting checks, and release builds on pull requests to the main branch.

## Goals

- Validate code quality on all pull requests before merging
- Test against both stable and MSRV (Minimum Supported Rust Version)
- Provide fast feedback to contributors
- Maintain high code quality standards
- Display CI status via README badge

## Architecture Decision

**Chosen Approach:** Single workflow with matrix strategy

**Rationale:**
- Simple to maintain (one file)
- Fast feedback (all checks run in parallel)
- Single status check for branch protection
- Matrix testing provides version coverage
- Easier for contributors to understand

**Alternatives Considered:**
- Separate workflows per check: More granular but added complexity
- Hybrid with dependencies: Optimizes CI minutes but slower feedback

## Workflow Structure

### File Location
```
.github/workflows/ci.yml
```

### Triggers
- **Pull Requests:** All PRs targeting `main` branch
- **Push to main:** Updates badge status after merge

### Matrix Strategy
Test against two Rust versions in parallel:
- **stable:** Latest stable Rust toolchain
- **MSRV:** 1.70.0 (Rust edition 2021 baseline)

### Jobs

#### 1. Test Job
- **Command:** `cargo test --verbose --all`
- **Purpose:** Run all unit tests, integration tests, and doc tests
- **Matrix:** Runs on both stable and MSRV
- **Expected time:** 30-60 seconds (with cache)

#### 2. Format Job
- **Command:** `cargo fmt --all --check`
- **Purpose:** Verify code follows Rust formatting standards
- **Matrix:** Stable only (rustfmt versions differ between Rust releases)
- **Expected time:** 5-10 seconds (with cache)

#### 3. Clippy Job
- **Command:** `cargo clippy --all-targets --all-features -- -D warnings`
- **Purpose:** Lint code and treat all warnings as errors
- **Matrix:** Runs on both stable and MSRV
- **Expected time:** 20-40 seconds (with cache)

#### 4. Build Job
- **Command:** `cargo build --release --verbose`
- **Purpose:** Verify optimized release build succeeds
- **Matrix:** Runs on both stable and MSRV
- **Expected time:** 60-90 seconds (with cache)

## Common Configuration

### Operating System
- **OS:** ubuntu-latest
- **Rationale:** Fastest runners, standard for Rust CI, sufficient for cross-platform Rust code

### Actions Used
- `actions/checkout@v4`: Check out repository code
- `dtolnay/rust-toolchain@{version}`: Install specific Rust toolchain
- `Swatinem/rust-cache@v2`: Cache Cargo dependencies and build artifacts

### Caching Strategy
- Automatic cache key based on Cargo.lock
- Shared cache across jobs
- Invalidates when dependencies change
- Reduces CI time from 3-5 minutes to 1-2 minutes

## Badge Integration

### Badge URLs
- **Badge Image:** `https://github.com/maxmalkin/sdiff/workflows/CI/badge.svg`
- **Badge Link:** `https://github.com/maxmalkin/sdiff/actions`

### README Placement
Add near the top of README.md, next to existing MIT License badge:
```markdown
[![CI](https://github.com/maxmalkin/sdiff/workflows/CI/badge.svg)](https://github.com/maxmalkin/sdiff/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
```

## Error Handling

### Failure Behavior
- Any job failure causes entire workflow to fail
- Matrix failures are independent (see which Rust version fails)
- Fail-fast disabled: all jobs complete to show all failures
- Verbose output for debugging

### Status Checks
- Workflow appears as single "CI" status check on PRs
- Must pass before merge (when branch protection enabled)
- Shows granular job results in GitHub Actions UI

## Branch Protection

### Recommended Settings
Once workflow is deployed:
1. Navigate to Settings → Branches → Branch protection rules
2. Add rule for `main` branch
3. Enable "Require status checks to pass before merging"
4. Select "CI" workflow as required check
5. Enable "Require branches to be up to date before merging"

## Performance Expectations

### CI Duration
- **First run (cold cache):** 3-5 minutes
- **Subsequent runs (warm cache):** 1-2 minutes
- **Matrix execution:** Parallel (total time ≈ slowest job)

### Resource Usage
- **Concurrent jobs:** 8 (4 jobs × 2 Rust versions)
- **Free tier:** GitHub provides 2,000 minutes/month for public repos
- **Expected usage:** ~2-4 minutes per PR (well within free tier)

## Implementation Notes

### MSRV Selection
- **Chosen:** 1.70.0
- **Rationale:**
  - Released April 2023
  - Stable support for Rust 2021 edition features
  - Recent enough for modern dependencies
  - Old enough for broad compatibility

### Why Not Test Beta/Nightly?
- SDIFF is a CLI tool, not a library
- Users will use stable Rust
- Testing beta/nightly adds CI cost without user benefit
- Can add later if needed

## Future Enhancements

Potential additions (not in initial scope):
- Security audit job (`cargo audit`)
- Automated releases on tags
- Multi-platform builds (Windows, macOS)
- Coverage reporting
- Documentation deployment

## Success Criteria

✅ All PRs automatically tested before merge
✅ Contributors see clear pass/fail status
✅ Badge shows current main branch status
✅ CI completes in under 2 minutes (cached)
✅ Catches formatting, lint, test, and build issues
✅ Works on both stable and MSRV

## References

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust CI Best Practices](https://github.com/actions-rs/meta)
- [Swatinem/rust-cache](https://github.com/Swatinem/rust-cache)
- [dtolnay/rust-toolchain](https://github.com/dtolnay/rust-toolchain)
