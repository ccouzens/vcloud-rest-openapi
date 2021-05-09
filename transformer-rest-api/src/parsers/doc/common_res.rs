use anyhow::{bail, Context, Result};
use rusty_v8 as v8;

#[derive(Debug)]
pub struct CommonRes {
    pub version_information: String,
    pub copyright: String,
}
pub fn parse(file_contents: &[u8]) -> Result<CommonRes> {
    let platform = v8::new_default_platform()
        .take()
        .context("Error getting v8")?;
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    let mut isolate = v8::Isolate::new(Default::default());

    let mut handle_scope = v8::HandleScope::new(&mut isolate);
    let scope = handle_scope.enter();

    let context = v8::Context::new(scope);
    let mut context_scope = v8::ContextScope::new(scope, context);
    let scope = context_scope.enter();

    let code = v8::String::new_from_utf8(scope, file_contents, v8::NewStringType::Normal)
        .context("Error creating code object")?;

    let mut script =
        v8::Script::compile(scope, context, code, None).context("Error compiling Javascript")?;
    script
        .run(scope, context)
        .context("Error running Javascript")?;

    let version_key = rusty_v8::String::new(scope, "ID_VersionInformation")
        .context("Error creating version string key")?
        .into();
    let copyright_key = rusty_v8::String::new(scope, "ID_Copyright")
        .context("Error creating copyright string key")?
        .into();
    let global = context.global(scope);
    let version_value = global
        .get(scope, context, version_key)
        .context("Error getting version value")?;
    let copyright_value = global
        .get(scope, context, copyright_key)
        .context("Error getting copyright value")?;

    if !version_value.is_string() {
        bail!("Expected version to be a string");
    }
    if !copyright_value.is_string() {
        bail!("Expected copyright to be a string");
    }

    let version_information = html2md::parse_html(
        &version_value
            .to_string(scope)
            .context("Expected to get string of version")?
            .to_rust_string_lossy(scope),
    );
    let copyright = html2md::parse_html(
        &copyright_value
            .to_string(scope)
            .context("Expected to get string of copyright")?
            .to_rust_string_lossy(scope),
    );
    Ok(CommonRes {
        version_information,
        copyright,
    })
}
