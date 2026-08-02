use std::fs;
use std::path::Path;

use my_diff_crate::algorithm::myers::MyersDiffWithLinearSpace;
use my_diff_crate::diff_utils::DiffUtils;
use my_diff_crate::text::delta_merge::inline_delta_merge_info::InlineDeltaMergeInfo;
use my_diff_crate::text::delta_merge::delta_merge_utils::DeltaMergeUtils;
use my_diff_crate::text::string_utils::StringUtils;
use my_diff_crate::{DiffRow, DiffRowGenerator, Tag};

fn split(content: &str) -> Vec<String> {
    content.lines().map(|s| s.to_string()).collect()
}

fn print_rows(rows: &[DiffRow]) {
    for row in rows {
        println!("{:?}", row);
    }
}

fn assert_inline_diff_result(
    generator: &DiffRowGenerator,
    original: &str,
    revised: &str,
    expected: &str,
) {
    let rows = generator.generate_diff_rows(
        &[original.to_string()],
        &[revised.to_string()],
    );
    print_rows(&rows);

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].old_line(), expected);
}

#[test]
fn test_generator_default() {
    let first = "anything \n \nother";
    let second = "anything\n\nother";

    let generator = DiffRowGenerator::create()
        .column_width(usize::MAX) // do not wrap
        .build();
    let rows = generator.generate_diff_rows(&split(first), &split(second));
    print_rows(&rows);

    assert_eq!(rows.len(), 3);
}

#[test]
fn test_normalize_list() {
    let generator = DiffRowGenerator::create().build();
    assert_eq!(
        vec!["    test".to_string()],
        generator.normalize_lines(&["\ttest".to_string()])
    );
}

#[test]
fn test_generator_default2() {
    let first = "anything \n \nother";
    let second = "anything\n\nother";

    let generator = DiffRowGenerator::create()
        .column_width(0) // do not wrap
        .build();
    let rows = generator.generate_diff_rows(&split(first), &split(second));
    print_rows(&rows);

    assert_eq!(rows.len(), 3);
}

#[test]
fn test_generator_inline_diff() {
    let first = "anything \n \nother";
    let second = "anything\n\nother";

    let generator = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .column_width(usize::MAX) // do not wrap
        .build();
    let rows = generator.generate_diff_rows(&split(first), &split(second));
    print_rows(&rows);

    assert_eq!(rows.len(), 3);
    assert!(rows[0].old_line().find("<span").is_some());
}

#[test]
fn test_generator_ignore_whitespaces() {
    let first = "anything \n \nother\nmore lines";
    let second = "anything\n\nother\nsome more lines";

    let generator = DiffRowGenerator::create()
        .ignore_white_spaces(true)
        .column_width(usize::MAX) // do not wrap
        .build();
    let rows = generator.generate_diff_rows(&split(first), &split(second));
    print_rows(&rows);

    assert_eq!(rows.len(), 4);
    assert_eq!(rows[0].tag(), Tag::Equal);
    assert_eq!(rows[1].tag(), Tag::Equal);
    assert_eq!(rows[2].tag(), Tag::Equal);
    assert_eq!(rows[3].tag(), Tag::Change);
}

#[test]
fn test_generator_with_word_wrap() {
    let first = "anything \n \nother";
    let second = "anything\n\nother";

    let generator = DiffRowGenerator::create().column_width(5).build();
    let rows = generator.generate_diff_rows(&split(first), &split(second));
    print_rows(&rows);

    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].to_string(), "[CHANGE,anyth<br/>ing ,anyth<br/>ing]");
    assert_eq!(rows[1].to_string(), "[CHANGE, ,]");
    assert_eq!(rows[2].to_string(), "[EQUAL,other,other]");
}

#[test]
fn test_generator_with_merge() {
    let first = "anything \n \nother";
    let second = "anything\n\nother";

    let generator = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .merge_original_revised(true)
        .build();
    let rows = generator.generate_diff_rows(&split(first), &split(second));
    print_rows(&rows);

    assert_eq!(rows.len(), 3);
    assert_eq!(
        rows[0].to_string(),
        "[CHANGE,anything<span class=\"editOldInline\"> </span>,anything]"
    );
    assert_eq!(
        rows[1].to_string(),
        "[CHANGE,<span class=\"editOldInline\"> </span>,]"
    );
    assert_eq!(rows[2].to_string(), "[EQUAL,other,other]");
}

#[test]
fn test_generator_with_merge2() {
    let generator = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .merge_original_revised(true)
        .build();
    let rows = generator.generate_diff_rows(
        &["Test".to_string()],
        &["ester".to_string()],
    );
    print_rows(&rows);

    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].to_string(),
        "[CHANGE,<span class=\"editOldInline\">T</span>est<span class=\"editNewInline\">er</span>,ester]"
    );
}

#[test]
fn test_generator_with_merge3() {
    let first = "test\nanything \n \nother";
    let second = "anything\n\nother\ntest\ntest2";

    let generator = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .merge_original_revised(true)
        .build();
    let rows = generator.generate_diff_rows(&split(first), &split(second));
    print_rows(&rows);

    assert_eq!(rows.len(), 6);
    assert_eq!(
        rows[0].to_string(),
        "[CHANGE,<span class=\"editOldInline\">test</span>,anything]"
    );
    assert_eq!(
        rows[1].to_string(),
        "[CHANGE,anything<span class=\"editOldInline\"> </span>,]"
    );
    assert_eq!(
        rows[2].to_string(),
        "[DELETE,<span class=\"editOldInline\"> </span>,]"
    );
    assert_eq!(rows[3].to_string(), "[EQUAL,other,other]");
    assert_eq!(
        rows[4].to_string(),
        "[INSERT,<span class=\"editNewInline\">test</span>,test]"
    );
    assert_eq!(
        rows[5].to_string(),
        "[INSERT,<span class=\"editNewInline\">test2</span>,test2]"
    );
}

#[test]
fn test_generator_with_merge_by_word4() {
    let generator = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .merge_original_revised(true)
        .inline_diff_by_word(true)
        .build();
    let rows = generator.generate_diff_rows(
        &["Test".to_string()],
        &["ester".to_string()],
    );
    print_rows(&rows);

    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].to_string(),
        "[CHANGE,<span class=\"editOldInline\">Test</span><span class=\"editNewInline\">ester</span>,ester]"
    );
}

#[test]
fn test_generator_with_merge_by_word5() {
    let generator = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .merge_original_revised(true)
        .inline_diff_by_word(true)
        .column_width(80)
        .build();
    let rows = generator.generate_diff_rows(
        &["Test feature".to_string()],
        &["ester feature best".to_string()],
    );
    print_rows(&rows);

    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].to_string(),
        "[CHANGE,<span class=\"editOldInline\">Test</span><span class=\"editNewInline\">ester</span> <br/>feature<span class=\"editNewInline\"> best</span>,ester feature best]"
    );
}

#[test]
fn test_split_string() {
    let list = DiffRowGenerator::split_string_preserve_delimiter(
        "test,test2",
        &my_diff_crate::text::diff_row_generator::SPLIT_BY_WORD_PATTERN,
    );
    assert_eq!(list.len(), 3);
    assert_eq!(format!("{:?}", list), "[\"test\", \",\", \"test2\"]");
}

#[test]
fn test_split_string2() {
    let list = DiffRowGenerator::split_string_preserve_delimiter(
        "test , test2",
        &my_diff_crate::text::diff_row_generator::SPLIT_BY_WORD_PATTERN,
    );
    println!("{:?}", list);
    assert_eq!(list.len(), 5);
    assert_eq!(format!("{:?}", list), "[\"test\", \" \", \",\", \" \", \"test2\"]");
}

#[test]
fn test_split_string3() {
    let list = DiffRowGenerator::split_string_preserve_delimiter(
        "test,test2,",
        &my_diff_crate::text::diff_row_generator::SPLIT_BY_WORD_PATTERN,
    );
    println!("{:?}", list);
    assert_eq!(list.len(), 4);
    assert_eq!(format!("{:?}", list), "[\"test\", \",\", \"test2\", \"\"]");
}

#[test]
fn test_generator_example1() {
    let generator = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .merge_original_revised(true)
        .inline_diff_by_word(true)
        .old_tag(|_tag, opening| if opening { "~".to_string() } else { "~".to_string() })
        .new_tag(|_tag, opening| if opening { "**".to_string() } else { "**".to_string() })
        .build();
    let rows = generator.generate_diff_rows(
        &["This is a test senctence.".to_string()],
        &["This is a test for diffutils.".to_string()],
    );

    println!("{}", rows[0].old_line());

    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].old_line(),
        "This is a test ~senctence~**for diffutils**."
    );
}

#[test]
fn test_generator_example2() {
    let generator = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .inline_diff_by_word(true)
        .old_tag(|_tag, opening| if opening { "~".to_string() } else { "~".to_string() })
        .new_tag(|_tag, opening| if opening { "**".to_string() } else { "**".to_string() })
        .build();
    let rows = generator.generate_diff_rows(
        &[
            "This is a test senctence.".to_string(),
            "This is the second line.".to_string(),
            "And here is the finish.".to_string(),
        ],
        &[
            "This is a test for diffutils.".to_string(),
            "This is the second line.".to_string(),
        ],
    );

    println!("|original|new|");
    println!("|--------|---|");
    for row in &rows {
        println!("|{}|{}|", row.old_line(), row.new_line());
    }

    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].old_line(), "This is a test ~senctence~.");
    assert_eq!(rows[0].new_line(), "This is a test **for diffutils**.");
}

#[test]
fn test_generator_unchanged() {
    let first = "anything \n \nother";
    let second = "anything\n\nother";

    let generator = DiffRowGenerator::create()
        .column_width(5)
        .report_lines_unchanged(true)
        .build();
    let rows = generator.generate_diff_rows(&split(first), &split(second));
    print_rows(&rows);

    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].to_string(), "[CHANGE,anything ,anything]");
    assert_eq!(rows[1].to_string(), "[CHANGE, ,]");
    assert_eq!(rows[2].to_string(), "[EQUAL,other,other]");
}

#[test]
fn test_generator_issue14() {
    let generator = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .merge_original_revised(true)
        .inline_diff_by_splitter(|line| {
            DiffRowGenerator::split_string_preserve_delimiter(
                line,
                &regex::Regex::new(",").unwrap(),
            )
        })
        .old_tag(|_tag, opening| if opening { "~".to_string() } else { "~".to_string() })
        .new_tag(|_tag, opening| if opening { "**".to_string() } else { "**".to_string() })
        .build();
    let rows = generator.generate_diff_rows(
        &["J. G. Feldstein, Chair".to_string()],
        &["T. P. Pastor, Chair".to_string()],
    );

    println!("{}", rows[0].old_line());

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].old_line(), "~J. G. Feldstein~**T. P. Pastor**, Chair");
}

#[test]
fn test_generator_issue15() {
    let generator = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .inline_diff_by_word(true)
        .old_tag(|_tag, opening| if opening { "~".to_string() } else { "~".to_string() })
        .new_tag(|_tag, opening| if opening { "**".to_string() } else { "**".to_string() })
        .build();

    let list_one = split(&fs::read_to_string("target/test-classes/mocks/issue15_1.txt").unwrap_or_default());
    let list_two = split(&fs::read_to_string("target/test-classes/mocks/issue15_2.txt").unwrap_or_default());

    let rows = generator.generate_diff_rows(&list_one, &list_two);

    if !rows.is_empty() {
        assert_eq!(rows.len(), 9);

        for row in &rows {
            println!("|{}| {} |", row.old_line(), row.new_line());
            if !row.old_line().starts_with("TABLE_NAME") {
                assert!(row.new_line().starts_with("**ACTIONS_C16913**"));
                assert!(row.old_line().starts_with("~ACTIONS_C1700"));
            }
        }
    }
}

#[test]
fn test_generator_issue22() {
    let generator = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .inline_diff_by_word(true)
        .old_tag(|_tag, opening| if opening { "~".to_string() } else { "~".to_string() })
        .new_tag(|_tag, opening| if opening { "**".to_string() } else { "**".to_string() })
        .build();
    let aa = "This is a test senctence.";
    let bb = "This is a test for diffutils.\nThis is the second line.";
    let rows = generator.generate_diff_rows(&split(aa), &split(bb));

    assert_eq!(
        format!("{:?}", rows),
        "[[CHANGE,This is a test ~senctence~.,This is a test **for diffutils**.], [INSERT,,**This is the second line.**]]"
    );
}

#[test]
fn test_generator_issue22_2() {
    let generator = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .inline_diff_by_word(true)
        .old_tag(|_tag, opening| if opening { "~".to_string() } else { "~".to_string() })
        .new_tag(|_tag, opening| if opening { "**".to_string() } else { "**".to_string() })
        .build();
    let aa = "This is a test for diffutils.\nThis is the second line.";
    let bb = "This is a test senctence.";
    let rows = generator.generate_diff_rows(&split(aa), &split(bb));

    assert_eq!(
        format!("{:?}", rows),
        "[[CHANGE,This is a test ~for diffutils~.,This is a test **senctence**.], [DELETE,~This is the second line.~,]]"
    );
}

#[test]
fn test_generator_issue22_3() {
    let generator = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .inline_diff_by_word(true)
        .old_tag(|_tag, opening| if opening { "~".to_string() } else { "~".to_string() })
        .new_tag(|_tag, opening| if opening { "**".to_string() } else { "**".to_string() })
        .build();
    let aa = "This is a test senctence.";
    let bb = "This is a test for diffutils.\nThis is the second line.\nAnd one more.";
    let rows = generator.generate_diff_rows(&split(aa), &split(bb));

    assert_eq!(
        format!("{:?}", rows),
        "[[CHANGE,This is a test ~senctence~.,This is a test **for diffutils**.], [INSERT,,**This is the second line.**], [INSERT,,**And one more.**]]"
    );
}

#[test]
fn test_generator_issue41_default_normalizer() {
    let generator = DiffRowGenerator::create().build();
    let rows = generator.generate_diff_rows(&["<".to_string()], &["<".to_string()]);
    assert_eq!(format!("{:?}", rows), "[[EQUAL,&lt;,&lt;]]");
}

#[test]
fn test_generator_issue41_user_normalizer() {
    let generator = DiffRowGenerator::create()
        .line_normalizer(|str_val| str_val.replace('\t', "    "))
        .build();
    let rows = generator.generate_diff_rows(&["<".to_string()], &["<".to_string()]);
    assert_eq!(format!("{:?}", rows), "[[EQUAL,<,<]]");

    let rows2 = generator.generate_diff_rows(&["\t<".to_string()], &["<".to_string()]);
    assert_eq!(format!("{:?}", rows2), "[[CHANGE,    <,<]]");
}

#[test]
fn test_generation_issue44_report_lines_unchanged_problem() {
    let generator = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .report_lines_unchanged(true)
        .old_tag(|_tag, opening| if opening { "~~".to_string() } else { "~~".to_string() })
        .new_tag(|_tag, opening| if opening { "**".to_string() } else { "**".to_string() })
        .build();
    let rows = generator.generate_diff_rows(
        &["<dt>To do</dt>".to_string()],
        &["<dt>Done</dt>".to_string()],
    );
    assert_eq!(
        format!("{:?}", rows),
        "[[CHANGE,<dt>~~T~~o~~ do~~</dt>,<dt>**D**o**ne**</dt>]]"
    );
}

#[test]
fn test_ignore_whitespace_issue66() {
    let generator = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .inline_diff_by_word(true)
        .ignore_white_spaces(true)
        .merge_original_revised(true)
        .old_tag(|_tag, opening| if opening { "~".to_string() } else { "~".to_string() })
        .new_tag(|_tag, opening| if opening { "**".to_string() } else { "**".to_string() })
        .build();

    let rows = generator.generate_diff_rows(
        &["This\tis\ta\ttest.".to_string()],
        &["This is a test".to_string()],
    );

    assert_eq!(rows[0].old_line(), "This    is    a    test~.~");
}

#[test]
fn test_ignore_whitespace_issue66_2() {
    let generator = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .inline_diff_by_word(true)
        .ignore_white_spaces(true)
        .merge_original_revised(true)
        .old_tag(|_tag, opening| if opening { "~".to_string() } else { "~".to_string() })
        .new_tag(|_tag, opening| if opening { "**".to_string() } else { "**".to_string() })
        .build();

    let rows = generator.generate_diff_rows(
        &["This  is  a  test.".to_string()],
        &["This is a test".to_string()],
    );

    assert_eq!(rows[0].old_line(), "This  is  a  test~.~");
}

#[test]
fn test_ignore_whitespace_issue64() {
    let generator = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .inline_diff_by_word(true)
        .ignore_white_spaces(true)
        .merge_original_revised(true)
        .old_tag(|_tag, opening| if opening { "~".to_string() } else { "~".to_string() })
        .new_tag(|_tag, opening| if opening { "**".to_string() } else { "**".to_string() })
        .build();

    let rows = generator.generate_diff_rows(
        &split("test\n\ntestline"),
        &split("A new text line\n\nanother one"),
    );

    let old_lines: Vec<String> = rows.iter().map(|r| r.old_line().to_string()).collect();
    assert_eq!(
        old_lines,
        vec!["~test~**A new text line**", "", "~testline~**another one**"]
    );
}

#[test]
fn test_replace_diffs_issue63() {
    let generator = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .inline_diff_by_word(true)
        .merge_original_revised(true)
        .old_tag(|_tag, opening| if opening { "~".to_string() } else { "~".to_string() })
        .new_tag(|_tag, opening| if opening { "**".to_string() } else { "**".to_string() })
        .process_diffs(|s| s.replace(' ', "/"))
        .build();

    let rows = generator.generate_diff_rows(
        &["This  is  a  test.".to_string()],
        &["This is a test".to_string()],
    );

    assert_eq!(rows[0].old_line(), "This~//~**/**is~//~**/**a~//~**/**test~.~");
}

#[test]
fn test_problem_too_many_diff_rows_issue65() {
    let generator = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .report_lines_unchanged(true)
        .old_tag(|_tag, opening| if opening { "~".to_string() } else { "~".to_string() })
        .new_tag(|_tag, opening| if opening { "**".to_string() } else { "**".to_string() })
        .merge_original_revised(true)
        .inline_diff_by_word(false)
        .replace_original_linefeed_in_changes_with_spaces(true)
        .build();

    let diff_rows = generator.generate_diff_rows(
        &[
            "Ich möchte nicht mit einem Bot sprechen.".to_string(),
            "Ich soll das schon wieder wiederholen?".to_string(),
        ],
        &[
            "Ich möchte nicht mehr mit dir sprechen. Leite mich weiter.".to_string(),
            "Kannst du mich zum Kundendienst weiterleiten?".to_string(),
        ],
    );

    print_rows(&diff_rows);
    assert_eq!(diff_rows.len(), 2);
}

#[test]
fn test_problem_too_many_diff_rows_issue65_no_merge() {
    let generator = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .report_lines_unchanged(true)
        .old_tag(|_tag, opening| if opening { "~".to_string() } else { "~".to_string() })
        .new_tag(|_tag, opening| if opening { "**".to_string() } else { "**".to_string() })
        .merge_original_revised(false)
        .inline_diff_by_word(false)
        .build();

    let diff_rows = generator.generate_diff_rows(
        &[
            "Ich möchte nicht mit einem Bot sprechen.".to_string(),
            "Ich soll das schon wieder wiederholen?".to_string(),
        ],
        &[
            "Ich möchte nicht mehr mit dir sprechen. Leite mich weiter.".to_string(),
            "Kannst du mich zum Kundendienst weiterleiten?".to_string(),
        ],
    );

    println!("{:?}", diff_rows);
    assert_eq!(diff_rows.len(), 2);
}

#[test]
fn test_problem_too_many_diff_rows_issue65_diff_by_word() {
    let generator = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .report_lines_unchanged(true)
        .old_tag(|_tag, opening| if opening { "~".to_string() } else { "~".to_string() })
        .new_tag(|_tag, opening| if opening { "**".to_string() } else { "**".to_string() })
        .merge_original_revised(true)
        .inline_diff_by_word(true)
        .build();

    let diff_rows = generator.generate_diff_rows(
        &[
            "Ich möchte nicht mit einem Bot sprechen.".to_string(),
            "Ich soll das schon wieder wiederholen?".to_string(),
        ],
        &[
            "Ich möchte nicht mehr mit dir sprechen. Leite mich weiter.".to_string(),
            "Kannst du mich zum Kundendienst weiterleiten?".to_string(),
        ],
    );

    println!("{:?}", diff_rows);
    assert_eq!(diff_rows.len(), 2);
}

#[test]
fn test_problem_too_many_diff_rows_issue65_no_inline_diff() {
    let generator = DiffRowGenerator::create()
        .show_inline_diffs(false)
        .report_lines_unchanged(true)
        .old_tag(|_tag, opening| if opening { "~".to_string() } else { "~".to_string() })
        .new_tag(|_tag, opening| if opening { "**".to_string() } else { "**".to_string() })
        .merge_original_revised(true)
        .inline_diff_by_word(false)
        .build();

    let diff_rows = generator.generate_diff_rows(
        &[
            "Ich möchte nicht mit einem Bot sprechen.".to_string(),
            "Ich soll das schon wieder wiederholen?".to_string(),
        ],
        &[
            "Ich möchte nicht mehr mit dir sprechen. Leite mich weiter.".to_string(),
            "Kannst du mich zum Kundendienst weiterleiten?".to_string(),
        ],
    );

    println!("{:?}", diff_rows);
    assert_eq!(diff_rows.len(), 2);
}

#[test]
fn test_linefeed_in_standard_tags_with_line_width_issue81() {
    let original = split(
        "American bobtail jaguar. American bobtail bombay but turkish angora and tomcat.\n\
         Russian blue leopard. Lion. Tabby scottish fold for russian blue, so savannah yet lynx. Tomcat singapura, cheetah.\n\
         Bengal tiger panther but singapura but bombay munchkin for cougar."
    );
    let revised = split(
        "bobtail jaguar. American bobtail turkish angora and tomcat.\n\
         Russian blue leopard. Lion. Tabby scottish folded for russian blue, so savannah yettie? lynx. Tomcat singapura, cheetah.\n\
         Bengal tiger panther but singapura but bombay munchkin for cougar. And more."
    );

    let generator = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .ignore_white_spaces(true)
        .column_width(100)
        .build();
    let deltas = generator.generate_diff_rows(&original, &revised);

    println!("{:?}", deltas);
}

#[test]
fn test_issue86_wrong_inline_diff() {
    let original_path = Path::new("target/test-classes/com/github/difflib/text/issue_86_original.txt");
    let revised_path = Path::new("target/test-classes/com/github/difflib/text/issue_86_revised.txt");

    if original_path.exists() && revised_path.exists() {
        let original = fs::read_to_string(original_path).unwrap_or_default();
        let revised = fs::read_to_string(revised_path).unwrap_or_default();

        let generator = DiffRowGenerator::create()
            .show_inline_diffs(true)
            .merge_original_revised(true)
            .inline_diff_by_word(true)
            .old_tag(|_tag, opening| if opening { "~".to_string() } else { "~".to_string() })
            .new_tag(|_tag, opening| if opening { "**".to_string() } else { "**".to_string() })
            .build();
        let rows = generator.generate_diff_rows(&split(&original), &split(&revised));

        rows.iter()
            .filter(|item| item.tag() != Tag::Equal)
            .for_each(|item| println!("{:?}", item));
    }
}

#[test]
fn test_correct_change_issue114() {
    let original = vec!["A".to_string(), "B".to_string(), "C".to_string(), "D".to_string(), "E".to_string()];
    let revised = vec!["a".to_string(), "C".to_string(), "".to_string(), "E".to_string()];

    let generator = DiffRowGenerator::create()
        .show_inline_diffs(false)
        .inline_diff_by_word(true)
        .old_tag(|_tag, opening| if opening { "~".to_string() } else { "~".to_string() })
        .new_tag(|_tag, opening| if opening { "**".to_string() } else { "**".to_string() })
        .build();
    let rows = generator.generate_diff_rows(&original, &revised);

    for diff in &rows {
        println!("{:?}", diff);
    }

    let tags: Vec<String> = rows.iter().map(|item| format!("{:?}", item.tag())).collect();
    assert_eq!(tags, vec!["Change", "Delete", "Equal", "Change", "Equal"]);
}

#[test]
fn test_correct_change_issue114_2() {
    let original = vec!["A".to_string(), "B".to_string(), "C".to_string(), "D".to_string(), "E".to_string()];
    let revised = vec!["a".to_string(), "C".to_string(), "".to_string(), "E".to_string()];

    let generator = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .inline_diff_by_word(true)
        .old_tag(|_tag, opening| if opening { "~".to_string() } else { "~".to_string() })
        .new_tag(|_tag, opening| if opening { "**".to_string() } else { "**".to_string() })
        .build();
    let rows = generator.generate_diff_rows(&original, &revised);

    for diff in &rows {
        println!("{:?}", diff);
    }

    let tags: Vec<String> = rows.iter().map(|item| format!("{:?}", item.tag())).collect();
    assert_eq!(tags, vec!["Change", "Delete", "Equal", "Change", "Equal"]);
    assert_eq!(rows[1].to_string(), "[DELETE,~B~,]");
}

#[test]
fn test_issue119_wrong_context_length() {
    let original_path = Path::new("target/test-classes/com/github/difflib/text/issue_119_original.txt");
    let revised_path = Path::new("target/test-classes/com/github/difflib/text/issue_119_revised.txt");

    if original_path.exists() && revised_path.exists() {
        let original = fs::read_to_string(original_path).unwrap_or_default();
        let revised = fs::read_to_string(revised_path).unwrap_or_default();

        let generator = DiffRowGenerator::create()
            .show_inline_diffs(true)
            .merge_original_revised(true)
            .inline_diff_by_word(true)
            .old_tag(|_tag, opening| if opening { "~".to_string() } else { "~".to_string() })
            .new_tag(|_tag, opening| if opening { "**".to_string() } else { "**".to_string() })
            .build();
        let rows = generator.generate_diff_rows(&split(&original), &split(&revised));

        rows.iter()
            .filter(|item| item.tag() != Tag::Equal)
            .for_each(|item| println!("{:?}", item));
    }
}

#[test]
fn test_issue129_with_delta_decompression() {
    let lines1 = vec![
        "apple1".to_string(),
        "apple2".to_string(),
        "apple3".to_string(),
        "A man named Frankenstein abc to Switzerland for cookies!".to_string(),
        "banana1".to_string(),
        "banana2".to_string(),
        "banana3".to_string(),
    ];
    let lines2 = vec![
        "apple1".to_string(),
        "apple2".to_string(),
        "apple3".to_string(),
        "A man named Frankenstein".to_string(),
        "xyz".to_string(),
        "to Switzerland for cookies!".to_string(),
        "banana1".to_string(),
        "banana2".to_string(),
        "banana3".to_string(),
    ];

    let txt = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .old_tag(|tag, is_opening| {
            if is_opening { format!("==old{:?}==>", tag) } else { "<==old==".to_string() }
        })
        .new_tag(|tag, is_opening| {
            if is_opening { format!("==new{:?}==>", tag) } else { "<==new==".to_string() }
        })
        .build()
        .generate_diff_rows(&lines1, &lines2)
        .iter()
        .map(|row| format!("{:?}", row.tag()).to_uppercase())
        .collect::<Vec<_>>()
        .join(" ");

    assert_eq!(
        txt,
        "EQUAL EQUAL EQUAL CHANGE INSERT INSERT EQUAL EQUAL EQUAL"
    );
}

#[test]
fn test_issue129_skip_delta_decompression() {
    let lines1 = vec![
        "apple1".to_string(),
        "apple2".to_string(),
        "apple3".to_string(),
        "A man named Frankenstein abc to Switzerland for cookies!".to_string(),
        "banana1".to_string(),
        "banana2".to_string(),
        "banana3".to_string(),
    ];
    let lines2 = vec![
        "apple1".to_string(),
        "apple2".to_string(),
        "apple3".to_string(),
        "A man named Frankenstein".to_string(),
        "xyz".to_string(),
        "to Switzerland for cookies!".to_string(),
        "banana1".to_string(),
        "banana2".to_string(),
        "banana3".to_string(),
    ];

    let txt = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .decompress_deltas(false)
        .old_tag(|tag, is_opening| {
            if is_opening { format!("==old{:?}==>", tag) } else { "<==old==".to_string() }
        })
        .new_tag(|tag, is_opening| {
            if is_opening { format!("==new{:?}==>", tag) } else { "<==new==".to_string() }
        })
        .build()
        .generate_diff_rows(&lines1, &lines2)
        .iter()
        .map(|row| format!("{:?}", row.tag()).to_uppercase())
        .collect::<Vec<_>>()
        .join(" ");

    assert_eq!(
        txt,
        "EQUAL EQUAL EQUAL CHANGE CHANGE CHANGE EQUAL EQUAL EQUAL"
    );
}

#[test]
fn test_issue129_skip_whitespace_changes() {
    let original_path = Path::new("target/test-classes/com/github/difflib/text/issue129_1.txt");
    let revised_path = Path::new("target/test-classes/com/github/difflib/text/issue129_2.txt");

    if original_path.exists() && revised_path.exists() {
        let original = fs::read_to_string(original_path).unwrap_or_default();
        let revised = fs::read_to_string(revised_path).unwrap_or_default();

        let generator = DiffRowGenerator::create()
            .show_inline_diffs(true)
            .merge_original_revised(true)
            .inline_diff_by_word(true)
            .ignore_white_spaces(true)
            .old_tag_simple(|is_opening| {
                if is_opening { "==old==>".to_string() } else { "<==old==".to_string() }
            })
            .new_tag(|tag, is_opening| {
                if is_opening { format!("==new{:?}==>", tag) } else { "<==new==".to_string() }
            })
            .build();
        let rows = generator.generate_diff_rows(&split(&original), &split(&revised));

        assert_eq!(rows.len(), 13);
        rows.iter()
            .filter(|item| item.tag() != Tag::Equal)
            .for_each(|item| println!("{:?}", item));
    }
}

#[test]
fn test_generator_with_whitespace_delta_merge() {
    let generator = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .merge_original_revised(true)
        .inline_diff_by_word(true)
        .old_tag(|_tag, opening| if opening { "~".to_string() } else { "~".to_string() })
        .new_tag(|_tag, opening| if opening { "**".to_string() } else { "**".to_string() })
        .line_normalizer(StringUtils::html_entities)
        .inline_delta_merger_arc(my_diff_crate::text::diff_row_generator::WHITESPACE_EQUALITIES_MERGER.clone())
        .build();

    assert_inline_diff_result(&generator, "No diff", "No diff", "No diff");
    assert_inline_diff_result(
        &generator,
        " x whitespace before diff",
        " y whitespace before diff",
        " ~x~**y** whitespace before diff",
    );
    assert_inline_diff_result(
        &generator,
        "Whitespace after diff x ",
        "Whitespace after diff y ",
        "Whitespace after diff ~x~**y** ",
    );
    assert_inline_diff_result(
        &generator,
        "Diff x x between",
        "Diff y y between",
        "Diff ~x x~**y y** between",
    );
    assert_inline_diff_result(
        &generator,
        "Hello \t world",
        "Hi \t universe",
        "~Hello \t world~**Hi \t universe**",
    );
    assert_inline_diff_result(
        &generator,
        "The quick brown fox jumps over the lazy dog",
        "A lazy dog jumps over a fox",
        "~The quick brown fox ~**A lazy dog **jumps over ~the lazy dog~**a fox**",
    );
}

#[test]
fn test_generator_with_merging_deltas_for_short_equalities() {
    let short_equalities_merger = |delta_merge_info: &InlineDeltaMergeInfo<String>| {
        DeltaMergeUtils::merge_inline_deltas(delta_merge_info, |equalities| {
            equalities.iter().map(|s| s.len()).sum::<usize>() < 6
        })
    };

    let generator = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .merge_original_revised(true)
        .inline_diff_by_word(true)
        .old_tag(|_tag, opening| if opening { "~".to_string() } else { "~".to_string() })
        .new_tag(|_tag, opening| if opening { "**".to_string() } else { "**".to_string() })
        .inline_delta_merger(short_equalities_merger)
        .build();

    assert_inline_diff_result(&generator, "No diff", "No diff", "No diff");
    assert_inline_diff_result(&generator, "aaa bbb ccc", "xxx bbb zzz", "~aaa bbb ccc~**xxx bbb zzz**");
    assert_inline_diff_result(&generator, "aaa bbbb ccc", "xxx bbbb zzz", "~aaa~**xxx** bbbb ~ccc~**zzz**");
}

#[test]
fn test_issue188_hang_on_examples() {
    let zip_path = Path::new("target/test-classes/com/github/difflib/text/test.zip");
    if zip_path.exists() {
        let original_path = Path::new("target/test-classes/com/github/difflib/text/old.html");
        let revised_path = Path::new("target/test-classes/com/github/difflib/text/new.html");

        if original_path.exists() && revised_path.exists() {
            let original = split(&fs::read_to_string(original_path).unwrap_or_default());
            let revised = split(&fs::read_to_string(revised_path).unwrap_or_default());

            let generator = DiffRowGenerator::create()
                .line_normalizer(|line| line.to_string())
                .show_inline_diffs(true)
                .merge_original_revised(true)
                .inline_diff_by_word(true)
                .decompress_deltas(true)
                .old_tag(|_tag, f| if f { "<s style=\"background-color: #bbbbbb\">".to_string() } else { "</s>".to_string() })
                .new_tag(|_tag, f| if f { "<b style=\"background-color: #aaffaa\">".to_string() } else { "</b>".to_string() })
                .build();

            let patch = DiffUtils::diff_with_algorithm(&original, &revised, &MyersDiffWithLinearSpace::default(), None, false);
            let rows = generator.generate_diff_rows_from_patch(&original, &mut patch.clone());

            println!("{:?}", rows);
        }
    }
}