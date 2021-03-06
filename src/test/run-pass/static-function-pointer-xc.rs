// Copyright 2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// xfail-fast
// aux-build:static-function-pointer-aux.rs
extern mod aux = "static-function-pointer-aux";

fn f(x: int) -> int { x }

pub fn main() {
    assert_eq!(aux::F(42), -42);
    unsafe {
        assert_eq!(aux::MutF(42), -42);
        aux::MutF = f;
        assert_eq!(aux::MutF(42), 42);
        aux::MutF = aux::f;
        assert_eq!(aux::MutF(42), -42);
    }
}
