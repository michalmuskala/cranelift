//! Test runner.
//!
//! This module implements the `TestRunner` struct which manages executing tests as well as
//! scanning directories for tests.

use std::ffi::OsStr;
use std::path::PathBuf;
use std::error::Error;
use CommandResult;

pub struct TestRunner {
    // Directories that have not yet been scanned.
    dir_stack: Vec<PathBuf>,

    // Filenames of tests to run.
    test_list: Vec<PathBuf>,

    errors: usize,
}

impl TestRunner {
    /// Create a new blank TrstRunner.
    pub fn new() -> TestRunner {
        TestRunner {
            dir_stack: Vec::new(),
            test_list: Vec::new(),
            errors: 0,
        }
    }

    /// Add a directory path to be scanned later.
    ///
    /// If `dir` turns out to be a regular file, it is silently ignored.
    /// Otherwise, any problems reading the directory are reported.
    pub fn push_dir<P: Into<PathBuf>>(&mut self, dir: P) {
        self.dir_stack.push(dir.into());
    }

    /// Add a test to be executed later.
    ///
    /// Any problems reading `file` as a test case file will be reported as a test failure.
    pub fn push_test<P: Into<PathBuf>>(&mut self, file: P) {
        self.test_list.push(file.into());
    }

    /// Scan any directories pushed so far.
    /// Push any potential test cases found.
    pub fn scan_dirs(&mut self) {
        // This recursive search tries to minimize statting in a directory hierarchy containing
        // mostly test cases.
        //
        // - Directory entries with a "cton" extension are presumed to be test case files.
        // - Directory entries with no extension are presumed to be subdirectories.
        // - Anything else is ignored.
        //
        while let Some(dir) = self.dir_stack.pop() {
            match dir.read_dir() {
                Err(err) => {
                    // Fail silently if `dir` was actually a regular file.
                    // This lets us skip spurious extensionless files without statting everything
                    // needlessly.
                    if !dir.is_file() {
                        self.path_error(dir, err);
                    }
                }
                Ok(entries) => {
                    // Read all directory entries. Avoid statting.
                    for entry_result in entries {
                        match entry_result {
                            Err(err) => {
                                // Not sure why this would happen. `read_dir` succeeds, but there's
                                // a problem with an entry. I/O error during a getdirentries
                                // syscall seems to be the reason. The implementation in
                                // libstd/sys/unix/fs.rs seems to suggest that breaking now would
                                // be a good idea, or the iterator could keep returning the same
                                // error forever.
                                self.path_error(dir, err);
                                break;
                            }
                            Ok(entry) => {
                                let path = entry.path();
                                // Recognize directories and tests by extension.
                                // Yes, this means we ignore directories with '.' in their name.
                                match path.extension().and_then(OsStr::to_str) {
                                    Some("cton") => self.push_test(path),
                                    Some(_) => {}
                                    None => self.push_dir(path),
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Report an error related to a path.
    fn path_error<E: Error>(&mut self, path: PathBuf, err: E) {
        self.errors += 1;
        println!("path-error {}: {}", path.to_string_lossy(), err);
    }

    /// Scan pushed directories for tests and run them.
    pub fn run(&mut self) -> CommandResult {
        self.scan_dirs();
        match self.errors {
            0 => Ok(()),
            1 => Err("1 failure".to_string()),
            n => Err(format!("{} failures", n)),
        }
    }
}