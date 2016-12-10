extern crate gcc;

fn main() {
    println!("cargo:rustc-link-lib=snappy");
    gcc::Config::new()
        .include("sparkey/src")
        .flag("-std=c99")
        .file("sparkey/src/MurmurHash3.c")
        .file("sparkey/src/buf.c")
        .file("sparkey/src/endiantools.c")
        .file("sparkey/src/hashalgorithms.c")
        .file("sparkey/src/hashheader.c")
        .file("sparkey/src/hashiter.c")
        .file("sparkey/src/hashreader.c")
        .file("sparkey/src/hashwriter.c")
        .file("sparkey/src/logheader.c")
        .file("sparkey/src/logreader.c")
        .file("sparkey/src/logwriter.c")
        .file("sparkey/src/returncodes.c")
        .file("sparkey/src/util.c")
        .compile("libsparkey.a");
}
