//! Embeds a Windows VERSIONINFO resource into `pc.exe`.
//!
//! SignPath Foundation requires every signed binary to carry file metadata
//! (product name, product version) that its signing policy can check, so the
//! release build must stamp these fields. Values come from Cargo.toml, which
//! keeps the embedded version in step with the release tag.

fn main() {
    // Only Windows targets get a resource; `cargo check` on other hosts is a no-op.
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
        // Fail the build rather than ship a binary the signing policy will reject.
        panic!("failed to embed Windows version resource: {err}");
    }
}
