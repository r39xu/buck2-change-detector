/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

//! A simple interning utility for strings.
//! Significantly reduce the memory of repeated strings.

use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;

use derive_more::Display;
use equivalent::Equivalent;
use serde::de::Visitor;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use static_interner::Intern;
use static_interner::Interner;

type StrData = Key<Box<str>>;

static INTERNER: Interner<StrData> = Interner::new();

/// An interned string whose contents are stored only once.
// Eq/PartialEq are OK, because they short-circuit on the hash
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Display)]
pub struct InternString(Intern<StrData>);

struct InternStringVisitor;

impl<'de> Visitor<'de> for InternStringVisitor {
    type Value = InternString;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(InternString::new(v))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(InternString::from_string(v))
    }
}

impl<'de> Deserialize<'de> for InternString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(InternStringVisitor)
    }
}

impl Serialize for InternString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl Default for InternString {
    fn default() -> Self {
        Self::new("")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Display)]
struct Key<T>(T);

impl<'a> Hash for Key<&'a str> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.0.as_bytes())
    }
}

impl Hash for Key<String> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.0.as_bytes())
    }
}

impl Hash for Key<Box<str>> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.0.as_bytes())
    }
}

impl<'a> Hash for Key<(&'a str, &'a str, &'a str)> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.0.0.as_bytes());
        state.write(self.0.1.as_bytes());
        state.write(self.0.2.as_bytes());
    }
}

impl<'a> Equivalent<StrData> for Key<&'a str> {
    fn equivalent(&self, key: &StrData) -> bool {
        self.0 == &*key.0
    }
}

impl Equivalent<StrData> for Key<String> {
    fn equivalent(&self, key: &StrData) -> bool {
        self.0.as_str() == &*key.0
    }
}

impl<'a> Equivalent<StrData> for Key<(&'a str, &'a str, &'a str)> {
    fn equivalent(&self, key: &StrData) -> bool {
        let (a, b, c) = self.0;
        // Important to split the key into bytes, so that we don't get middle-of-UTF8 panics
        let xabc = key.0.as_bytes();
        if a.len() + b.len() + c.len() != xabc.len() {
            return false;
        }
        let (xa, xbc) = xabc.split_at(a.len());
        if a.as_bytes() != xa {
            //
            return false;
        }
        let (xb, xc) = xbc.split_at(b.len());
        b.as_bytes() == xb && c.as_bytes() == xc
    }
}

impl<'a> From<Key<&'a str>> for StrData {
    fn from(value: Key<&str>) -> Self {
        Key(value.0.into())
    }
}

impl From<Key<String>> for StrData {
    fn from(value: Key<String>) -> Self {
        Key(value.0.into_boxed_str())
    }
}

impl<'a> From<Key<(&'a str, &'a str, &'a str)>> for StrData {
    fn from(value: Key<(&str, &str, &str)>) -> Self {
        let mut res = String::with_capacity(value.0.0.len() + value.0.1.len() + value.0.2.len());
        res.push_str(value.0.0);
        res.push_str(value.0.1);
        res.push_str(value.0.2);
        Key(res.into_boxed_str())
    }
}

impl InternString {
    pub fn new(x: &str) -> Self {
        InternString(INTERNER.intern(Key(x)))
    }

    /// Equivalent to `new` with the three arguments concatenated.
    pub fn new3(x: &str, y: &str, z: &str) -> Self {
        InternString(INTERNER.intern(Key((x, y, z))))
    }

    pub fn from_string(x: String) -> Self {
        InternString(INTERNER.intern(Key(x)))
    }

    pub fn as_str(&self) -> &str {
        &self.0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intern_string() {
        assert_eq!(
            InternString::new("abcdef"),
            InternString::new3("ab", "cde", "f")
        );
        assert_ne!(
            InternString::new("abcdefgh"),
            InternString::new3("ab", "", "defg!")
        );
    }
}
