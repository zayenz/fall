extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate neon;
extern crate neon_serde;
extern crate fall_tree;
extern crate fall_editor;
extern crate lang_rust;
extern crate lang_fall;

use std::iter;

use neon::vm::{Call, JsResult, Lock};
use neon::mem::Handle;
use neon::scope::Scope;
use neon::js::{JsString, JsInteger, JsNull, JsValue, JsFunction};
use neon::js::class::{Class, JsClass};
use neon::task::Task;

use fall_tree::{TextEditBuilder, Text, TextRange, TextEdit};
use fall_editor::{EditorSupport, EditorFile};

mod support;

use self::support::{arg1, ret};
use neon_serde::from_value;

const LANGUAGES: &[EditorSupport] = &[
    lang_fall::FALL_EDITOR_SUPPORT,
    lang_rust::RUST_EDITOR_SUPPORT,
];

declare_types! {
    pub class JsSupport for EditorSupport {
        init(call) {
            let scope = call.scope;
            let idx: usize = arg1(scope, &call.arguments)?;
            Ok(LANGUAGES[idx])
        }

        method parse(call) {
            let scope = call.scope;
            let text = call.arguments.require(scope, 0)?.check::<JsString>()?;
            let support = call.arguments.this(scope);

            let class: Handle<JsClass<JsFile>> = JsFile::class(scope)?;
            let ctor: Handle<JsFunction<JsFile>> = class.constructor(scope)?;
            let ctor_args = iter::once(support.upcast()).chain(iter::once(text.upcast()));
            let file = ctor.construct::<_, JsValue, _>(scope, ctor_args)?;
            Ok(file.upcast())
        }
    }

    pub class JsFile for EditorFile {
        init(call) {
            let scope = call.scope;
            let file = match call.arguments.len() {
                2 => {
                    let mut support = call.arguments.require(scope, 0)?.check::<JsSupport>()?;
                    let text = call.arguments.require(scope, 1)?.check::<JsString>()?.value();
                    support.grab(move |support| support.parse(&text))
                }
                3 => {
                    let mut file: Handle<JsFile> = call.arguments.require(scope, 0)?.check::<JsFile>()?;
                    let edits = call.arguments.require(scope, 1)?;
                    let edits: Vec<VsEdit> = from_value(scope, edits)?;
                    file.grab(|file| {
                        let edits = from_vs_edits(file.file().text(), edits);
                        file.edit(&edits)
                    })
                }
                _ => panic!("Bad ctor invocation")
            };
            Ok(file)
        }

        method edit(call) {
            let scope = call.scope;

            let this = call.arguments.this(scope);
            let edits = call.arguments.require(scope, 0)?;

            let class: Handle<JsClass<JsFile>> = JsFile::class(scope)?;
            let constructor: Handle<JsFunction<JsFile>> = class.constructor(scope)?;
            let args = iter::once(this.upcast()).chain(iter::once(edits)).chain(iter::once(JsNull::new().upcast()));
            let file = constructor.construct(scope, args)?;
            Ok(file.upcast())
        }

        method syntaxTree(call) {
            let scope = call.scope;
            let tree = call.arguments.this(scope).grab(move |file| {
                file.syntax_tree()
            });
            ret(scope, tree)
        }

        method structure(call) {
            let scope = call.scope;
            let structure = call.arguments.this(scope).grab(|file| file.structure());
            ret(scope, structure)
        }

        method highlight(call) {
            let scope = call.scope;
            let highlights = call.arguments.this(scope).grab(move |file| {
                file.highlight()
            });
            ret(scope, highlights)
        }

        method metrics(call) {
            let scope = call.scope;
            let metrics = call.arguments.this(scope).grab(move |file| {
                file.metrics()
            });
            ret(scope, metrics)
        }
    }
}

register_module!(m, {
    m.export("status", status)?;
    m.export("supportForExtension", support_for_extension)?;
    Ok(())
});

fn support_for_extension(call: Call) -> JsResult<JsValue> {
    let scope = call.scope;
    let ext: String = arg1(scope, &call.arguments)?;
    let idx = match LANGUAGES.iter().position(|s| s.extension == ext) {
        None => return Ok(JsNull::new().upcast()),
        Some(idx) => idx,
    };

    let arg: Handle<JsValue> = JsInteger::new(scope, idx as i32).upcast();
    let class: Handle<JsClass<JsSupport>> = JsSupport::class(scope)?;
    let constructor: Handle<JsFunction<JsSupport>> = class.constructor(scope)?;
    let sup = constructor.construct::<_, JsValue, _>(scope, ::std::iter::once(arg))?;
    Ok(sup.upcast())
}

fn status(call: Call) -> JsResult<JsValue> {
    let scope = call.scope;
    ret(scope, "Hello from Rust")
}

#[derive(Serialize, Deserialize)]
struct VsEdit {
    delete: TextRange,
    insert: String,
}

fn from_vs_edits(text: Text, edits: Vec<VsEdit>) -> TextEdit {
    let mut edit = TextEditBuilder::new(text);
    for e in edits {
        edit.replace(e.delete, e.insert)
    }
    edit.build()
}
