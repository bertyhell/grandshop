fn main() {
    let config = slint_build::CompilerConfiguration::new()
        .with_style("fluent-light".into())
        // translations/<lang>/LC_MESSAGES/grandshop.po, keyed on the English text only
        .with_bundled_translations("translations")
        .with_default_translation_context(slint_build::DefaultTranslationContext::None);
    slint_build::compile_with_config("ui/app-window.slint", config).expect("Slint build failed");

    // The border styles are embedded with include_dir!, which cargo doesn't watch by itself
    println!("cargo:rerun-if-changed=assets/borders");

    // ort's prebuilt ONNX Runtime uses @available checks in its CoreML provider, which call
    // ___isPlatformVersionAtLeast from clang's runtime. rustc links with -nodefaultlibs, so
    // that runtime has to be added by hand.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        let output = std::process::Command::new("clang")
            .arg("--print-resource-dir")
            .output()
            .expect("running clang --print-resource-dir failed");
        let resource_dir = String::from_utf8(output.stdout).expect("clang resource dir isn't UTF-8");
        let clang_rt = std::path::Path::new(resource_dir.trim()).join("lib/darwin/libclang_rt.osx.a");
        assert!(clang_rt.exists(), "{} not found", clang_rt.display());
        println!("cargo:rustc-link-arg={}", clang_rt.display());
    }

    // Icon of the .exe in Explorer and the taskbar
    #[cfg(windows)]
    winresource::WindowsResource::new()
        .set_icon("assets/icons/icon.ico")
        .compile()
        .expect("embedding the Windows icon failed");
}
