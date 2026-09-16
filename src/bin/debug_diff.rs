use java_diff_utils_rs::algorithm::myers::myers::MyersDiff;
use java_diff_utils_rs::algorithm::myers::myers_linear::MyersDiffWithLinearSpace;
use java_diff_utils_rs::algorithm::DiffAlgorithm;
use java_diff_utils_rs::patch::Patch;

fn main() {
    let linear = MyersDiffWithLinearSpace::<&str>::default();
    let quad = MyersDiff::<&str>::default();

    let source = vec!["aaa", "bbb", "ccc"];
    let target = vec!["aaa", "zzz", "ccc"];
    let c1 = linear.diff(&source, &target);
    let p1 = Patch::generate(&source, &target, &c1, false);
    println!("Linear change case - {} deltas:", p1.deltas().len());
    for d in p1.deltas() {
        println!(
            "  {:?} src=pos{} lines={:?}  tgt=pos{} lines={:?}",
            d.delta_type(),
            d.source().position(),
            d.source().lines(),
            d.target().position(),
            d.target().lines()
        );
    }
    let c2 = quad.diff(&source, &target);
    let p2 = Patch::generate(&source, &target, &c2, false);
    println!("Quad change case - {} deltas:", p2.deltas().len());
    for d in p2.deltas() {
        println!(
            "  {:?} src=pos{} lines={:?}  tgt=pos{} lines={:?}",
            d.delta_type(),
            d.source().position(),
            d.source().lines(),
            d.target().position(),
            d.target().lines()
        );
    }

    println!();
    let source2 = vec!["The", "dog", "is", "brown"];
    let target2 = vec!["The", "fox", "is", "down"];
    let c3 = linear.diff(&source2, &target2);
    let p3 = Patch::generate(&source2, &target2, &c3, false);
    println!("Linear type_names - {} deltas:", p3.deltas().len());
    for d in p3.deltas() {
        println!(
            "  {:?} src=pos{} lines={:?}  tgt=pos{} lines={:?}",
            d.delta_type(),
            d.source().position(),
            d.source().lines(),
            d.target().position(),
            d.target().lines()
        );
    }
    let c4 = quad.diff(&source2, &target2);
    let p4 = Patch::generate(&source2, &target2, &c4, false);
    println!("Quad type_names - {} deltas:", p4.deltas().len());
    for d in p4.deltas() {
        println!(
            "  {:?} src=pos{} lines={:?}  tgt=pos{} lines={:?}",
            d.delta_type(),
            d.source().position(),
            d.source().lines(),
            d.target().position(),
            d.target().lines()
        );
    }
}
