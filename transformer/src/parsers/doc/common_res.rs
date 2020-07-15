use rusty_v8 as v8;

#[derive(Debug)]
pub struct CommonRes {
    pub version_information: String,
    pub copyright: String,
}
pub fn parse(file_contents: &[u8]) -> Result<CommonRes, Box<dyn std::error::Error>> {
    let platform = v8::new_default_platform()
        .take()
        .ok_or("Error getting v8")?;
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    let mut isolate = v8::Isolate::new(Default::default());

    let mut handle_scope = v8::HandleScope::new(&mut isolate);
    let scope = handle_scope.enter();

    let context = v8::Context::new(scope);
    let mut context_scope = v8::ContextScope::new(scope, context);
    let scope = context_scope.enter();

    let code = v8::String::new_from_utf8(scope, file_contents, v8::NewStringType::Normal)
        .ok_or("Error creating code object")?;

    let mut script =
        v8::Script::compile(scope, context, code, None).ok_or("Error compiling Javascript")?;
    script
        .run(scope, context)
        .ok_or("Error running Javascript")?;

    let version_key = rusty_v8::String::new(scope, "ID_VersionInformation")
        .ok_or("Error creating version string key")?
        .into();
    let copyright_key = rusty_v8::String::new(scope, "ID_Copyright")
        .ok_or("Error creating copyright string key")?
        .into();
    let global = context.global(scope);
    let version_value = global
        .get(scope, context, version_key)
        .ok_or("Error getting version value")?;
    let copyright_value = global
        .get(scope, context, copyright_key)
        .ok_or("Error getting copyright value")?;

    if !version_value.is_string() {
        return Err("Expected version to be a string".into());
    }
    if !copyright_value.is_string() {
        return Err("Expected copyright to be a string".into());
    }

    let version_information = html2md::parse_html(
        &version_value
            .to_string(scope)
            .ok_or("Expected to get string of version")?
            .to_rust_string_lossy(scope),
    );
    let copyright = html2md::parse_html(
        &copyright_value
            .to_string(scope)
            .ok_or("Expected to get string of copyright")?
            .to_rust_string_lossy(scope),
    );
    Ok(CommonRes {
        version_information,
        copyright,
    })
}
