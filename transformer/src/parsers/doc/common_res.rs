use rusty_v8 as v8;
use unhtml::FromHtml;

#[derive(Debug)]
pub struct CommonRes {
    pub version_information: String,
    pub copyright: String,
}
fn decode_html_attributes(input: &str) -> String {
    #[derive(FromHtml, Debug)]
    struct HtmlText(#[html(attr = "inner")] String);
    HtmlText::from_html(&input).unwrap().0
}
pub fn parse(file_contents: &[u8]) -> CommonRes {
    let platform = v8::new_default_platform().unwrap();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    let mut isolate = v8::Isolate::new(Default::default());

    let mut handle_scope = v8::HandleScope::new(&mut isolate);
    let scope = handle_scope.enter();

    let context = v8::Context::new(scope);
    let global = context.global(scope);
    let mut context_scope = v8::ContextScope::new(scope, context);
    let scope = context_scope.enter();

    let code = v8::String::new_from_utf8(scope, file_contents, v8::NewStringType::Normal).unwrap();

    let mut script = v8::Script::compile(scope, context, code, None).unwrap();
    script.run(scope, context).unwrap();

    let version_key = rusty_v8::String::new(scope, "ID_VersionInformation")
        .unwrap()
        .into();
    let copyright_key = rusty_v8::String::new(scope, "ID_Copyright").unwrap().into();
    assert!(context
        .global(scope)
        .get(scope, context, version_key)
        .unwrap()
        .is_string());
    assert!(context
        .global(scope)
        .get(scope, context, copyright_key)
        .unwrap()
        .is_string());

    let version_information = decode_html_attributes(
        &global
            .get(scope, context, version_key)
            .unwrap()
            .to_string(scope)
            .unwrap()
            .to_rust_string_lossy(scope),
    );
    let copyright = decode_html_attributes(
        &global
            .get(scope, context, copyright_key)
            .unwrap()
            .to_string(scope)
            .unwrap()
            .to_rust_string_lossy(scope),
    );
    CommonRes {
        version_information,
        copyright,
    }
}
