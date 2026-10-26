fn main() {
    let config = slint_build::CompilerConfiguration::new()
        .with_style("fluent-light".into())
        // translations/<lang>/LC_MESSAGES/grandshop.po, keyed on the English text only
        .with_bundled_translations("translations")
        .with_default_translation_context(slint_build::DefaultTranslationContext::None);
    slint_build::compile_with_config("ui/app-window.slint", config).expect("Slint build failed");

    // Icon of the .exe in Explorer and the taskbar
    #[cfg(windows)]
    winresource::WindowsResource::new()
        .set_icon("assets/icons/icon.ico")
        .compile()
        .expect("embedding the Windows icon failed");
}
