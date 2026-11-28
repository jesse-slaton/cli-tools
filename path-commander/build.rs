//! Embeds a Windows VERSIONINFO resource into `pc.exe`.
//!
//! SignPath Foundation requires every signed binary to carry file metadata
//! (product name, product version) that its signing policy can check, so the
//! release build must stamp these fields. Values come from Cargo.toml, which
//! keeps the embedded version in step with the release tag.

fn main() {
    // Without these, Cargo reruns this script (and recompiles pc) whenever any
    // file in the package changes. Package metadata read through env!() is
    // already tracked by Cargo itself.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=PC_REQUIRE_VERSIONINFO");

    // CARGO_CFG_TARGET_OS is the TARGET: only Windows targets get a resource,
    // so a host-target `cargo check` on macOS/Linux is a no-op.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let version = env!("CARGO_PKG_VERSION");

    let mut res = winresource::WindowsResource::new();
    res.set("ProductName", "Path Commander")
        .set("FileDescription", env!("CARGO_PKG_DESCRIPTION"))
        .set("ProductVersion", version)
        .set("FileVersion", version)
        .set("OriginalFilename", "pc.exe")
        .set("InternalName", "pc.exe")
        .set(
            "LegalCopyright",
            "Copyright (c) 2025-2026 Jesse Slaton. MIT License.",
        );

    if let Err(err) = res.compile() {
        // Host vs target: inside a build script, cfg!(windows) describes the
        // HOST that runs this script, not the target being built. A Windows
        // host (CI and the release runner) has the SDK's rc.exe, so a failure
        // there is a real error and must stop the build rather than produce a
        // binary the signing policy will reject. A macOS/Linux host
        // cross-checking a Windows target usually lacks llvm-rc, so it only
        // warns unless PC_REQUIRE_VERSIONINFO is set to a non-empty value.
        let required = std::env::var_os("PC_REQUIRE_VERSIONINFO").is_some_and(|v| !v.is_empty());
        if cfg!(windows) || required {
            panic!("failed to embed Windows version resource: {err}");
        }
        println!("cargo:warning=VERSIONINFO not embedded ({err}); this binary is not releasable");
    }
}
