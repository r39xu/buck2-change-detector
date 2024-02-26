/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

//! Helper functions for the supertd CLIs, so they are all consistent.
//! Supports things like args files.

use std::env::args_os;
use std::ffi::OsString;

use anyhow::Context as _;
use argfile::Argument;
use clap::Parser;

pub fn get_args() -> anyhow::Result<Vec<OsString>> {
    // Buck2 drops empty lines in arg files, so we should do the same.
    fn parse_file_skipping_blanks(content: &str, prefix: char) -> Vec<Argument> {
        let mut res = argfile::parse_fromfile(content, prefix);
        res.retain(|x| match x {
            Argument::PassThrough(arg) => !arg.is_empty(),
            _ => true,
        });
        res
    }

    argfile::expand_args_from(args_os(), parse_file_skipping_blanks, argfile::PREFIX)
        .context("When parsing arg files")
}

/// Set up tracing so it prints to stderr, and can be used for output.
/// Most things should use `info` and `debug` level for showing messages.
pub fn parse_args<T: Parser>() -> anyhow::Result<T> {
    Ok(T::parse_from(get_args()?))
}
