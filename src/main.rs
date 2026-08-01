use diffus::algorithm::{
    diff_algorithm_factory::{AlgorithmType, DiffAlgorithmFactory},
    myers::myers_linear::MyersDiffLinear,
    Change, DiffAlgorithm, MyersDiff,
};

fn main() {
    println!("=== Testing Diff Engine Setup ===\n");

    let source = vec!["apple", "banana", "cherry", "date"];
    let target = vec!["apple", "cherry", "dragonfruit", "date"];

    println!("Source: {:?}", source);
    println!("Target: {:?}\n", target);

    // 1. Direct Myers standard execution
    println!("--- 1. Testing Standard MyersDiff ---");
    let myers = MyersDiff::new();
    let changes: Vec<Change> = myers.compute_diff(&source, &target);
    println!("Standard Myers Changes count: {}", changes.len());
    for change in &changes {
        println!("  Change: {:?}", change);
    }

    // 2. Linear Myers Stub Execution
    println!("\n--- 2. Testing Stubbed MyersDiffLinear ---");
    let linear_myers = MyersDiffLinear::new();
    let linear_changes = linear_myers.compute_diff(&source, &target);
    println!("Linear Myers Changes count: {}", linear_changes.len());

    // 3. Factory Pattern Creation Test
    println!("\n--- 3. Testing DiffAlgorithmFactory ---");
    let algo_from_factory = DiffAlgorithmFactory::create::<&str>(AlgorithmType::Myers);
    let factory_changes = algo_from_factory.compute_diff(&source, &target);
    println!("Factory Myers Changes count: {}", factory_changes.len());

    println!("\n=== Verification Successful! All stubs and traits linked properly. ===");
}