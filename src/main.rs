use my_diff_crate::algorithm::myers::myers::MyersDiff;
use my_diff_crate::algorithm::DiffAlgorithm;
use my_diff_crate::patch::patch_failed_exception::PatchFailedException;
use my_diff_crate::patch::Patch;
use my_diff_crate::text::delta_merge::delta_merge_utils::DeltaMergeUtils;
use my_diff_crate::text::delta_merge::inline_delta_merge_info::InlineDeltaMergeInfo;
use my_diff_crate::text::diff_row_generator::DiffRowGenerator;
use my_diff_crate::text::string_utils;

fn main() -> Result<(), PatchFailedException> {
    println!("=== 1. Core Patch & Diff Demo ===");

    // Convert string slices to owned Strings
    let source: Vec<String> = vec!["A", "B", "C", "D"]
        .into_iter()
        .map(String::from)
        .collect();
    let target: Vec<String> = vec!["A", "C", "D", "E"]
        .into_iter()
        .map(String::from)
        .collect();

    // Compute diff using Myers algorithm
    let myers = MyersDiff::<String>::default();
    let changes = myers.diff(&source, &target);

    println!("Computed {} change(s):", changes.len());
    for change in &changes {
        println!("  {:?}", change);
    }
    println!("---");

    // Generate a patch from computed changes
    let patch = Patch::generate(&source, &target, &changes, false);

    // Apply patch to source
    let patched = patch.apply_to(&source)?;
    println!("Patched result : {:?}", patched);
    assert_eq!(patched, target, "Patched source must match target!");

    // Restore target back to original source
    let restored = patch.restore(&target)?;
    println!("Restored result: {:?}", restored);
    assert_eq!(restored, source, "Restored target must match original!");

    println!("\n=== 2. Text Utilities (StringUtils) ===");

    let html_snippet = "fn compare(a: i32, b: i32) -> bool { a < b && b > 0 }\t// check";
    let normalized = string_utils::normalize(html_snippet);
    println!("Original  : {}", html_snippet);
    println!("Normalized: {}\n", normalized);

    let long_text = "The quick brown fox jumps over the lazy dog";
    let wrapped = string_utils::wrap_text(long_text, 12);
    println!("Wrapped text (width 12):\n{}\n", wrapped);

    println!("=== 3. Side-by-Side View (DiffRowGenerator) ===");

    // Instantiate DiffRowGenerator via DiffRowGenerator::create()
    let generator = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .inline_diff_by_word(true)
        .build();

    let diff_rows = generator.generate_diff_rows(&source, &target);

    println!("{:<6} | {:<25} | {:<25}", "TAG", "OLD", "NEW");
    println!("{:-<6}-|-{:-<25}-|-{:-<25}", "", "", "");
    for row in &diff_rows {
        println!(
            "{:<6?} | {:<25} | {:<25}",
            row.tag(),
            row.old_line(),
            row.new_line()
        );
    }

    println!("\n=== 4. Inline Delta Merging (DeltaMergeUtils) ===");

    // Construct InlineDeltaMergeInfo using deltas from existing patch
    let merge_info = InlineDeltaMergeInfo::new(
        patch.deltas().to_vec(),
        source.clone(),
        target.clone(),
    );

    // Demonstrate DeltaMergeUtils by merging adjacent deltas
    let merged_deltas = DeltaMergeUtils::merge_inline_deltas(&merge_info, |equalities| {
        equalities.iter().all(|s| s.trim().is_empty())
    });

    println!(
        "Original delta count: {} | Merged delta count: {}",
        patch.deltas().len(),
        merged_deltas.len()
    );

    println!("\n✅ All operations completed successfully!");
    Ok(())
}