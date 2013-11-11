// Copyright 2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use version::{try_getting_version, try_getting_local_version,
              Version, NoVersion, split_version};
use std::hash::Streaming;
use std::{char, hash};
use messages::error;

/// Path-fragment identifier of a package such as
/// 'github.com/graydon/test'; path must be a relative
/// path with >=1 component.
#[deriving(Clone)]
pub struct PkgId {
    /// This is a path, on the local filesystem, referring to where the
    /// files for this package live. For example:
    /// github.com/mozilla/quux-whatever (it's assumed that if we're
    /// working with a package ID of this form, rustpkg has already cloned
    /// the sources into a local directory in the RUST_PATH).
    path: Path,
    /// Short name. This is the path's filestem, but we store it
    /// redundantly so as to not call get() everywhere (filestem() returns an
    /// option)
    /// The short name does not need to be a valid Rust identifier.
    /// Users can write: `extern mod foo = "...";` to get around the issue
    /// of package IDs whose short names aren't valid Rust identifiers.
    short_name: ~str,
    /// The requested package version.
    version: Version
}

impl Eq for PkgId {
    fn eq(&self, other: &PkgId) -> bool {
        self.path == other.path && self.version == other.version
    }
}

// n.b. This code is pretty silly; we should use the real URL library.
fn drop_url_scheme<'a>(s: &'a str) -> Option<&'a str> {
    let mut is_url = true;
    let mut seen = 0;
    let mut result = None;
    for substr in s.split_str_iter("://") {
        debug!("Scanning {}", substr);
        for c in substr.iter() {
            if !is_url_part(c) {
                is_url = false;
                break;
            }
        }
        if seen == 1 {
            let no_extension = substr.trim_right_chars(&|c: char| c != '.');
            result = Some(no_extension.slice_to(no_extension.len() - 1));
        }
        seen += 1;
    }
    if is_url && seen > 1 {
        result
    } else {
        None
    }
}

// n.b. This code is pretty silly; we should use the real URL library.
fn is_url_part(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '-' || ch == '_' || ch.is_digit() || ch == '.' || ch == '/' ||
        (ch > '\x7f' && (char::is_XID_start(ch) || char::is_XID_continue(ch)))
}

// Fails if this is not a legal package ID,
// after printing a hint
fn ensure_legal_package_id(s: &str) {
    let mut legal = true;
    for ch in s.iter() {
        // Hack to ignore everything after the optional '#'
        if ch == '#' {
            break;
        }
        if !is_url_part(ch) {
            legal = false;
            break;
        }
    }
    if !legal {
        let maybe_intended_path = drop_url_scheme(s);
        debug!("is {} a URL? {}", s, maybe_intended_path.is_some());

        for maybe_package_id in maybe_intended_path.iter() {
            error(format!("rustpkg operates on package IDs; did you mean to write \
                          `{}` instead of `{}`?",
                  *maybe_package_id,
                  s));
        }
        fail!("Can't parse {} as a package ID", s);
    }
}

impl PkgId {
    pub fn new(s: &str) -> PkgId {
        use conditions::bad_pkg_id::cond;

        // Make sure the path is a legal package ID -- it might not even
        // be a legal path, so we do this first
        ensure_legal_package_id(s);

        let mut given_version = None;

        // Did the user request a specific version?
        let s = match split_version(s) {
            Some((path, v)) => {
                given_version = Some(v);
                path
            }
            None => {
                s
            }
        };

        let path = Path::new(s);
        if !path.is_relative() {
            return cond.raise((path, ~"absolute pkgid"));
        }
        if path.filename().is_none() {
            return cond.raise((path, ~"0-length pkgid"));
        }
        let short_name = path.filestem_str().expect(format!("Strange path! {}", s));

        let version = match given_version {
            Some(v) => v,
            None => match try_getting_local_version(&path) {
                Some(v) => v,
                None => match try_getting_version(&path) {
                    Some(v) => v,
                    None => NoVersion
                }
            }
        };

        PkgId {
            path: path.clone(),
            short_name: short_name.to_owned(),
            version: version
        }
    }

    pub fn hash(&self) -> ~str {
        // FIXME (#9639): hash should take a &[u8] so we can hash the real path
        do self.path.display().with_str |s| {
            let vers = self.version.to_str();
            format!("{}-{}-{}", s, hash(s + vers), vers)
        }
    }

    pub fn short_name_with_version(&self) -> ~str {
        format!("{}{}", self.short_name, self.version.to_str())
    }

    /// True if the ID has multiple components
    pub fn is_complex(&self) -> bool {
        self.short_name.as_bytes() != self.path.as_vec()
    }

    pub fn prefixes_iter(&self) -> Prefixes {
        prefixes_iter(&self.path)
    }

    // This is the workcache function name for the *installed*
    // binaries for this package (as opposed to the built ones,
    // which are per-crate).
    pub fn install_tag(&self) -> ~str {
        format!("install({})", self.to_str())
    }
}

pub fn prefixes_iter(p: &Path) -> Prefixes {
    Prefixes {
        components: p.str_component_iter().map(|x|x.unwrap().to_owned()).to_owned_vec(),
        remaining: ~[]
    }
}

struct Prefixes {
    priv components: ~[~str],
    priv remaining: ~[~str]
}

impl Iterator<(Path, Path)> for Prefixes {
    #[inline]
    fn next(&mut self) -> Option<(Path, Path)> {
        if self.components.len() <= 1 {
            None
        }
        else {
            let last = self.components.pop();
            self.remaining.unshift(last);
            // converting to str and then back is a little unfortunate
            Some((Path::new(self.components.connect("/")),
                  Path::new(self.remaining.connect("/"))))
        }
    }
}

impl ToStr for PkgId {
    fn to_str(&self) -> ~str {
        // should probably use the filestem and not the whole path
        format!("{}-{}", self.path.as_str().unwrap(), self.version.to_str())
    }
}


pub fn write<W: Writer>(writer: &mut W, string: &str) {
    writer.write(string.as_bytes());
}

pub fn hash(data: ~str) -> ~str {
    let hasher = &mut hash::default_state();
    write(hasher, data);
    hasher.result_str()
}

