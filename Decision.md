# decision.md

## `java-diff-utils-rs` — Architecture Decision Log

**Reference project:** [`java-diff-utils`](https://github.com/java-diff-utils/java-diff-utils) (Java)
**Our project:** `java-diff-utils-rs` — a Rust port
**Status:** Hackathon submission, work in progress

This document records the actual architecture decisions we made while porting `java-diff-utils` to Rust — where we kept the original's design, where we deliberately diverged, and why.

---

## AD-1: Class hierarchy → enum + composition

**Java does this with:** `AbstractDelta<T>` as a base class, subclassed by `ChangeDelta`, `DeleteDelta`, `InsertDelta`, `EqualDelta`, each overriding `applyTo` / `restore`. Type checking happens via `instanceof` / `getType()`.

**We did this instead:** A single `DeltaType` enum (`Change`, `Delete`, `Insert`, `Equal`) as a plain tag on a generic `Delta<T>` struct, plus thin wrapper types (`ChangeDelta<T>`, `DeleteDelta<T>`, etc.) that hold an inner `Delta<T>` by composition rather than inheritance.

**Why:** Rust has no class inheritance, and reaching for `Box<dyn Delta>` trait objects would have cost us dynamic dispatch and lifetime complexity for something that's really just a closed, 4-variant tag. An enum lets us pattern-match exhaustively (the compiler tells us if we miss a case) and keeps `Delta<T>` a plain, cheaply-cloneable, serializable value type. The wrapper structs exist to keep the ergonomic, per-type constructors (`ChangeDelta::new(...)`) that callers coming from the Java API would expect, while `into_delta()` / `From<ChangeDelta<T>> for Delta<T>` let us drop back to the unified representation wherever the algorithm code needs it.

**Trade-off accepted:** We lose Java's ability to add virtual-dispatch behavior per subclass without touching the base type. In exchange we get exhaustiveness checking and no allocation for delta storage.

---

## AD-2: Exceptions → `Result<T, DiffError>`

**Java does this with:** Checked exceptions — `PatchFailedException`, `UnsupportedOperationException`, thrown across the call stack.

**We did this instead:** A single `DiffError` enum (`General`, `PatchFailed`, `UnsupportedOperation`) implementing `std::error::Error`, with type aliases `PatchError` and `PatchFailedException` kept for naming parity with the Java API. Every fallible operation (`verify_and_apply_to`, `apply_to`, `restore`, ...) returns `Result<_, DiffError>` instead of throwing.

**Why:** This is the idiomatic Rust equivalent of Java's checked exceptions — the compiler forces every caller to handle or propagate the error via `?`, which is the same "you must deal with this" guarantee Java gives you at compile time with `throws`, just expressed through the type system instead of the exception mechanism.

**Trade-off accepted:** We collapsed Java's exception *class hierarchy* into enum *variants* of one type. That's a coarser granularity — Java code that catches `PatchFailedException` specifically vs. a general `RuntimeException` doesn't have a direct one-to-one equivalent here. We judged this an acceptable simplification for a first port.

---

## AD-3: Interfaces → traits, with a blanket impl for closures

**Java does this with:** `DiffAlgorithm<T>` and `DiffAlgorithmListener` as interfaces that concrete classes (`MyersDiff`, etc.) implement.

**We did this instead:** `DiffAlgorithm<T>` and `DiffAlgorithmListener` as Rust traits, both with default method bodies (`diff()` defaults to calling `diff_with_listener()` with a no-op listener; `path_node()` defaults to delegating to `diff_step()`). We went a step further than a direct port and added a blanket `impl<T, F: Fn(&[T], &[T]) -> Vec<Change>> DiffAlgorithm<T> for F`, so a plain closure can be used anywhere a `DiffAlgorithm` is expected, without writing a wrapper struct.

**Why:** Default trait methods let us mirror Java's abstract-class-with-template-method pattern (shared boilerplate, one method left for implementors to fill in) without needing an actual base class. The closure blanket impl isn't in the Java original — we added it because in Rust, "a function is a valid instance of a single-method interface" is idiomatic and removes ceremony that Java requires (an anonymous class or lambda-implementing-interface) for the same use case.

---

## AD-4: In-place `Vec` mutation via `splice`, not item-by-item add/remove

**Java does this with:** `ArrayList.add()` / `remove()` calls, typically iterating to shift elements for insert/delete/change deltas.

**We did this instead:** `target.splice(position..position + size, replacement.iter().cloned())` for change/insert/delete application and restoration.

**Why:** `Vec::splice` replaces a range in a single operation with one internal shift of the tail elements, instead of Java's per-element `add`/`remove` calls, each of which can trigger its own shift. It also reads closer to "replace this chunk with that chunk," which matches the domain concept (a delta *is* a chunk replacement) more directly than a loop.

**Trade-off accepted:** We require `T: Clone` at these call sites (to clone replacement elements into the target vector), which Java doesn't need to think about explicitly since object references are copied by default there.

---

## AD-5: `serde` support added — not present in the Java original

**We did this:** Added `#[derive(Serialize, Deserialize)]` to `Delta<T>` and `DeltaType`, with a `serde(bound(...))` clause so the derive works generically over `T`.

**Why:** This has no Java equivalent to port — we added it deliberately, since a Rust diff/patch library that can't (de)serialize its own patch objects (e.g., to cache a computed patch, or send it over a wire) is missing something an idiomatic Rust crate would be expected to have. It was a scope decision, not a translation.

---

## AD-6: Module layout mirrors the Java package structure 1:1

**Decision:** `algorithm/`, `patch/`, `unifieddiff/`, `text/` map directly onto the original's `com.github.difflib.*` packages, file-for-file where practical (`chunk.rs` ↔ `Chunk.java`, `unified_diff_reader.rs` ↔ `UnifiedDiffReader.java`, etc.).

**Why:** With a codebase this size, being able to trace any Rust file back to its Java counterpart line-by-line was essential for verifying correctness under time pressure — several of our fixes (see below) came directly from instrumenting the real Java class and diffing its behavior against ours. A 1:1 layout made that comparison mechanical instead of a hunt.

---

## AD-7: Scope decision — fuzzy patch matching deferred, not architected around

**Decision:** We chose to *not* redesign the core patch-application path to accommodate fuzzy matching once it became clear it lived in a separate code path from the Myers/chunk-verification fix. Rather than reshaping `Delta`/`Chunk` architecture around an unfinished feature, we kept fuzzy matching (`apply_fuzzy_to_at`) as a bolt-on method and left it unresolved, so it wouldn't destabilize the parts that were already verified correct.

**Why:** With limited time, we prioritized architectural stability of the confirmed-working core (algorithm, patch, unified diff) over a redesign that might have fixed fuzzy matching but risked the rest. This is a deliberate trade-off, not an oversight — described further below.

---

## What Came Out of These Decisions (Verification Summary)

| Suite | Passed | Failed | Notes |
|---|---|---|---|
| `patch` | 8 | 0 | All passing |
| `diff_utils_test` | 6 | 0 | All passing |
| `generate_unified_diff_test` | 11 | 0 | All passing |
| `example` | 4 | 0 | All passing |
| `algorithm` | 9 | 1 | Core Myers path/coalescing correct; fuzzy matching still failing |
| `unifieddiff` | 40 | 1 | One failure on new-file header syntax (`@@ -1,0 @@` vs `@@ -0,0 @@`) |
| `integration_tests` | 34 | 2 | Fuzzy patch error variant + unicode wrap boundary |
| `text` | 16 | 36 | `DiffRowGenerator` / string utils diverge from Java semantics |

**Total: 128 passed / 39 failed / 1 ignored**, across 8 test binaries.

The Myers snake-collapsing fix (`PathNode::previous_snake`), the `Chunk::verify_chunk_at` rewrite, and the conflict-marker rewrite were all found by compiling and instrumenting the real upstream Java classes and tracing node-for-node against our Rust output — a direct payoff of AD-6.

---

## Known Gap: Fuzzy Matching (Not For Lack Of Trying)

Per AD-7, fuzzy patch application (`test_fuzzy_apply`, `test_fuzzy_patch_unsupported`) is still broken. We spent real time on it — it just lives in a separate code path from the core Myers fix, and we couldn't fully root-cause it before time ran out. It's the one place we hit a wall and made the call to stop rather than risk destabilizing everything else. In hindsight, we probably should have made that call earlier and redirected the time toward the `text` module instead, which was more tractable.

Other known gaps, for the record:
- `text::string_utils` and `DiffRowGenerator` diverge significantly from Java reference behavior (HTML entity handling, `<br/>` substitution, tab/space normalization direction) — the largest remaining chunk of work, at 36 failing tests.
- Unicode-safe text wrapping has a grapheme-boundary bug.
- One unified diff header-syntax mismatch on new-file diffs.

---

## Reflections

This was our first time taking on a project at this architectural scale — porting a mature, well-tested library across languages, under a hackathon clock, while trying to hold ourselves to matching upstream behavior rather than just "something that compiles." We were probably a little ambitious picking a project this big for a first attempt, and the fuzzy-matching wall is proof of that. But we're genuinely proud that the core algorithm, patch application, and unified diff generation are all solid and verified against the real Java source.

Thank you to the Post-Mortem Hackathon organizers for putting this event together — it pushed us to take on something bigger than we normally would have, and to sit with a hard bug instead of walking away from it. Thanks again for the chance to grow as individuals through this. Even the parts we couldn't fix taught us something.

;)