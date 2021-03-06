//! File tests.
//!
//! This crate contains the main test driver as well as implementations of the
//! available filetest commands.

#![deny(
    missing_docs,
    trivial_numeric_casts,
    unused_extern_crates,
    unstable_features
)]
#![warn(unused_import_braces)]
#![cfg_attr(feature = "cargo-clippy", allow(type_complexity))]
#![cfg_attr(
    feature = "cargo-clippy",
    warn(
        clippy::mut_mut,
        clippy::nonminimal_bool,
        clippy::option_map_unwrap_or,
        clippy::option_map_unwrap_or_else,
        clippy::unicode_not_nfc,
        clippy::use_self
    )
)]

extern crate cranelift_codegen;
extern crate cranelift_preopt;
extern crate cranelift_reader;
extern crate file_per_thread_logger;
extern crate filecheck;
extern crate num_cpus;
#[macro_use]
extern crate log;

use cranelift_codegen::timing;
use cranelift_reader::TestCommand;
use runner::TestRunner;
use std::path::Path;
use std::time;

mod concurrent;
mod match_directive;
mod runner;
mod runone;
mod subtest;

mod test_binemit;
mod test_cat;
mod test_compile;
mod test_dce;
mod test_domtree;
mod test_legalizer;
mod test_licm;
mod test_postopt;
mod test_preopt;
mod test_print_cfg;
mod test_regalloc;
mod test_shrink;
mod test_simple_gvn;
mod test_simple_preopt;
mod test_verifier;

/// The result of running the test in a file.
type TestResult = Result<time::Duration, String>;

/// Main entry point for `clif-util test`.
///
/// Take a list of filenames which can be either `.clif` files or directories.
///
/// Files are interpreted as test cases and executed immediately.
///
/// Directories are scanned recursively for test cases ending in `.clif`. These test cases are
/// executed on background threads.
///
pub fn run(verbose: bool, report_times: bool, files: &[String]) -> TestResult {
    let mut runner = TestRunner::new(verbose, report_times);

    for path in files.iter().map(Path::new) {
        if path.is_file() {
            runner.push_test(path);
        } else {
            runner.push_dir(path);
        }
    }

    runner.start_threads();
    runner.run()
}

/// Used for 'pass' subcommand.
/// Commands are interpreted as test and executed.
///
/// Directories are scanned recursively for test cases ending in `.clif`.
///
pub fn run_passes(
    verbose: bool,
    report_times: bool,
    passes: &[String],
    target: &str,
    file: &str,
) -> TestResult {
    let mut runner = TestRunner::new(verbose, /* report_times */ false);

    let path = Path::new(file);
    if path == Path::new("-") || path.is_file() {
        runner.push_test(path);
    } else {
        runner.push_dir(path);
    }

    let result = runner.run_passes(passes, target);
    if report_times {
        println!("{}", timing::take_current());
    }
    result
}

/// Create a new subcommand trait object to match `parsed.command`.
///
/// This function knows how to create all of the possible `test <foo>` commands that can appear in
/// a `.clif` test file.
fn new_subtest(parsed: &TestCommand) -> subtest::SubtestResult<Box<subtest::SubTest>> {
    match parsed.command {
        "binemit" => test_binemit::subtest(parsed),
        "cat" => test_cat::subtest(parsed),
        "compile" => test_compile::subtest(parsed),
        "dce" => test_dce::subtest(parsed),
        "domtree" => test_domtree::subtest(parsed),
        "legalizer" => test_legalizer::subtest(parsed),
        "licm" => test_licm::subtest(parsed),
        "postopt" => test_postopt::subtest(parsed),
        "simple_preopt" => test_simple_preopt::subtest(parsed),
        "print-cfg" => test_print_cfg::subtest(parsed),
        "regalloc" => test_regalloc::subtest(parsed),
        "shrink" => test_shrink::subtest(parsed),
        "simple-gvn" => test_simple_gvn::subtest(parsed),
        "verifier" => test_verifier::subtest(parsed),
        "preopt" => test_preopt::subtest(parsed),
        _ => Err(format!("unknown test command '{}'", parsed.command)),
    }
}
