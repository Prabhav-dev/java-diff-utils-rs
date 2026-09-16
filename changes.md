# Changes

## 0.1.0-alpha.2

### Changed

- Use `HistogramDiff` as the default algorithm in `DiffUtils`.
- Normalize adjacent histogram insert/delete records that represent one replacement into a single `Change` record.
- Keep low-entropy repeated-value regions on the histogram path instead of forcing them through the generic fallback.
- Update the crate and lockfile version to `0.1.0-alpha.2`.

### Tests

- Added regression coverage for the 100,000-element repeated-value case.
- Verified the affected `algorithm`, `diff_utils_test`, and `patch` targets with 35 passing tests and 1 ignored test.

### Follow-up

- The repeated-value benchmark still needs a fresh release-mode timing comparison against JGit before claiming full performance parity.
