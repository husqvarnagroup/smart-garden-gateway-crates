// SPDX-FileCopyrightText: GARDENA GmbH
//
// SPDX-License-Identifier: MIT

fn main() {
    let dst = cmake::Config::new("../third_party/lsdl-serializer").build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=lsdl-serializer");
}
