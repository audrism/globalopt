// Compiles the GlobalMinimum Fortran library (vendored under
// fortran/vendor, synced from upstream/GlobalMinimumFortran/real.8 by
// tools/sync_fortran.sh) plus the bridge files into a static library
// linked into this crate.  Enabled by the "fortran" feature (default).
//
// Requires gfortran on PATH at build time (set FC to override).

use std::env;
use std::path::PathBuf;
use std::process::Command;

const ALGO_FILES: &[&str] = &[
    "anal1.f", "anal2.f", "bayes1.f", "exkor.f", "extr.f", "flexi.f",
    "glopt.f", "lbayes.f", "lpmin.f", "mig1.f", "mig2.f", "mivar4.f",
    "reqp.f", "unt.f",
];

fn main() {
    if env::var("CARGO_FEATURE_FORTRAN").is_err() {
        return;
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let fc = env::var("FC").unwrap_or_else(|_| "gfortran".to_string());

    println!("cargo:rerun-if-changed=fortran");
    println!("cargo:rerun-if-env-changed=FC");

    let mut objects = Vec::new();
    let mut sources: Vec<PathBuf> = ALGO_FILES
        .iter()
        .map(|f| PathBuf::from("fortran/vendor").join(f))
        .collect();
    sources.push(PathBuf::from("fortran/gm_util.f"));
    sources.push(PathBuf::from("fortran/gm_fi.f"));

    for src in &sources {
        let obj = out_dir.join(format!(
            "{}.o",
            src.file_stem().unwrap().to_string_lossy()
        ));
        let status = Command::new(&fc)
            .args([
                "-c",
                "-O2",
                "-fPIC",
                "-std=legacy",
                // REQUIRED: the real.8 tree assumes f77 -r8 style
                // promotion (see docs/FORTRAN_INTERFACES.md).
                "-fdefault-real-8",
                "-fdefault-double-8",
            ])
            .arg(src)
            .arg("-o")
            .arg(&obj)
            .status()
            .unwrap_or_else(|e| panic!("failed to run {fc}: {e}"));
        assert!(status.success(), "gfortran failed on {}", src.display());
        objects.push(obj);
    }

    let lib = out_dir.join("libgmfortran.a");
    let mut ar = Command::new(env::var("AR").unwrap_or_else(|_| "ar".to_string()));
    ar.arg("crs").arg(&lib);
    ar.args(&objects);
    assert!(ar.status().expect("failed to run ar").success());

    cc::Build::new()
        .file("fortran/gm_shim.c")
        .include("fortran")
        .compile("gmshim");

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=gmfortran");

    // Link the gfortran runtime; ask the compiler where it lives.
    if let Ok(output) = Command::new(&fc)
        .args(["-print-file-name=libgfortran.so"])
        .output()
    {
        let path = String::from_utf8_lossy(&output.stdout);
        let path = path.trim();
        if let Some(dir) = PathBuf::from(path).parent() {
            if dir.components().count() > 0 {
                println!("cargo:rustc-link-search=native={}", dir.display());
            }
        }
    }
    println!("cargo:rustc-link-lib=dylib=gfortran");
}
