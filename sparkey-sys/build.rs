extern crate gcc;

fn main() {
    gcc::compile_library("libsparkey.a", &[
        "sparkey/src/MurmurHash3.c",
        "sparkey/src/MurmurHash3.h",
        "sparkey/src/buf.c",
        "sparkey/src/buf.h",
        "sparkey/src/endiantools.c",
        "sparkey/src/endiantools.h",
        "sparkey/src/hashalgorithms.c",
        "sparkey/src/hashalgorithms.h",
        "sparkey/src/hashheader.c",
        "sparkey/src/hashheader.h",
        "sparkey/src/hashiter.c",
        "sparkey/src/hashiter.h",
        "sparkey/src/hashreader.c",
        "sparkey/src/hashwriter.c",
        "sparkey/src/logheader.c",
        "sparkey/src/logheader.h",
        "sparkey/src/logreader.c",
        "sparkey/src/logwriter.c",
        "sparkey/src/returncodes.c",
        "sparkey/src/sparkey-internal.h",
        "sparkey/src/sparkey.h",
        "sparkey/src/util.c",
        "sparkey/src/util.h",
    ]);
}
