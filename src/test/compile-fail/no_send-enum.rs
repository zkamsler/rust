// Copyright 2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::kinds::marker;

enum Foo {
    A(marker::NoSend)
}

fn bar<T: Send>(_: T) {}

fn main() {
    let x = A(marker::NoSend);
    bar(x); //~ ERROR instantiating a type parameter with an incompatible type `Foo`, which does not fulfill `Send`
}
