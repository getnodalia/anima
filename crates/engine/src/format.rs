//! The `.anima` native format: a versioned JSON envelope around the document.

use crate::EngineError;
use anima_doc::{Document, path_table};
use anima_geom::Path;
use craft_anicraft_format::PayloadDecoder;
pub use craft_anicraft_format::{FORMAT_NAME, FORMAT_VERSION, MAX_FILE_BYTES};

/// The extension Ánima saves its documents with.
pub const EXTENSION: &str = "anima";
/// The extension documents saved before Ánima had its own name carry. They are the same format
/// and keep opening, from the file dialog, a drop or a template folder.
pub const LEGACY_EXTENSION: &str = "anicraft";

/// Whether `ext` names a native document, `.anima` or the older one, in any case.
pub fn is_native_extension(ext: &str) -> bool {
    ext.eq_ignore_ascii_case(EXTENSION) || ext.eq_ignore_ascii_case(LEGACY_EXTENSION)
}
use serde::Deserialize;
use std::sync::Arc;

/// Ánima document deserialization uses its own path-table scope while the shared format crate
/// controls when a payload is decoded in the envelope stream.
struct DocumentDecoder;

impl PayloadDecoder<Arc<Path>> for DocumentDecoder {
    type Document = Document;

    fn decode<'de, D>(&mut self, deserializer: D, paths: Option<Vec<Arc<Path>>>) -> Result<Self::Document, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let _scope = paths.map(path_table::begin_read);
        Document::deserialize(deserializer)
    }
}

pub fn to_json(doc: &Document) -> Result<String, EngineError> {
    let paths = path_table::build_table(doc);
    let generator = format!("Ánima {}", env!("CARGO_PKG_VERSION"));
    path_table::write_with_paths(&paths, || craft_anicraft_format::write(&generator, &paths, doc)).map_err(|e| EngineError::Format(e.to_string()))
}

pub fn from_json(text: &str) -> Result<Document, EngineError> {
    let env = craft_anicraft_format::read::<Arc<Path>, _>(text, &mut DocumentDecoder)
        .map_err(|e| EngineError::Format(format!("not a valid .anima file: {e}")))?;
    if env.format != FORMAT_NAME {
        return Err(EngineError::Format(format!("unexpected format '{}'", env.format)));
    }
    if env.version > FORMAT_VERSION {
        return Err(EngineError::Format(format!("file version {} is newer than this Ánima (supports {FORMAT_VERSION})", env.version)));
    }
    let mut doc = env.document.ok_or_else(|| EngineError::Format("the file has no document".into()))?;
    doc.sanitize();
    Ok(doc)
}

pub fn load(path: &std::path::Path) -> Result<Document, EngineError> {
    let len = crate::vfs::len(path)?;
    if len > MAX_FILE_BYTES {
        return Err(EngineError::Format(format!("file is {len} bytes; the limit is {MAX_FILE_BYTES}")));
    }
    from_json(&crate::vfs::read_to_string(path)?)
}

pub fn save(doc: &Document, path: &std::path::Path) -> Result<(), EngineError> {
    let json = to_json(doc)?;
    // Write next to the target, then rename: a crash mid-write never corrupts the old file.
    let tmp = path.with_extension("anima.tmp");
    crate::vfs::write(&tmp, json)?;
    crate::vfs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anima_doc::sample;

    #[test]
    fn documents_saved_before_the_rename_are_native_documents() {
        let older = ["anicraft", "AniCraft"]; // rebrand: keep
        for ext in ["anima", "ANIMA"].into_iter().chain(older) {
            assert!(is_native_extension(ext), "{ext}");
        }
        for ext in ["fla", "xfl", "json", "", "anim"] {
            assert!(!is_native_extension(ext), "{ext}");
        }
        let old = std::path::Path::new("walk cycle.anicraft"); // rebrand: keep
        assert_eq!(crate::cmds_drop::kind_of(old), crate::cmds_drop::DropKind::Document);
        // The file carries the same signature under either extension.
        let doc = sample::demo();
        assert!(from_json(&to_json(&doc).expect("save")).is_ok());
        assert_eq!(FORMAT_NAME, "anicraft", "renaming the signature would make every saved document unreadable"); // rebrand: keep
    }

    #[test]
    fn round_trip_is_lossless() {
        let d = sample::demo();
        let back = from_json(&to_json(&d).unwrap()).unwrap();
        assert_eq!(d, back);
    }

    #[test]
    fn shared_paths_are_written_once_and_shared_again_on_load() {
        let d = anima_doc::sample::stress(2, 40, 48); // 4 keyframes per element share a path
        let json = to_json(&d).unwrap();
        let env: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(env["version"], 3);
        assert_eq!(env["paths"].as_array().unwrap().len(), 80, "one entry per distinct path");
        let back = from_json(&json).unwrap();
        assert_eq!(d, back);
        let l = &back.timeline(0).layers[0];
        let (anima_doc::ElementKind::Shape(a), anima_doc::ElementKind::Shape(b)) =
            (&l.keyframes[0].elements[0].kind, &l.keyframes[3].elements[0].kind)
        else {
            panic!()
        };
        assert!(Arc::ptr_eq(&a.path, &b.path), "sharing restored on load");
    }

    #[test]
    fn duplicate_envelope_keys_keep_last_values_and_path_table_scope() {
        let doc = sample::stress(2, 40, 48);
        let wire: serde_json::Value = serde_json::from_str(&to_json(&doc).unwrap()).unwrap();
        let paths = wire["paths"].as_array().unwrap();
        let mut earlier_paths = paths.clone();
        earlier_paths.reverse();
        assert_ne!(earlier_paths.as_slice(), paths.as_slice());
        let json = format!(
            "{{\"format\":\"other\",\"format\":\"anicraft\",\"version\":99,\"version\":3,\"paths\":{},\"paths\":{},\"document\":{},\"ignored\":true}}",
            serde_json::to_string(&earlier_paths).unwrap(),
            serde_json::to_string(paths).unwrap(),
            serde_json::to_string(&wire["document"]).unwrap(),
        );
        assert_eq!(from_json(&json).unwrap(), doc);
    }

    #[test]
    fn malformed_earlier_duplicate_document_keeps_error_precedence_and_location() {
        let doc = sample::demo();
        let wire: serde_json::Value = serde_json::from_str(&to_json(&doc).unwrap()).unwrap();
        let later_document = serde_json::to_string(&wire["document"]).unwrap();
        let json = format!("{{\n  \"format\":\"anicraft\",\n  \"version\":3,\n  \"document\":null,\n  \"document\":{later_document}\n}}");
        let error = from_json(&json).err().map(|e| e.to_string()).unwrap_or_default();
        assert!(error.contains("not a valid .anima file: invalid type: null"), "{error}");
        assert!(error.contains("line 4"), "payload diagnostics keep source locations: {error}");
    }

    #[test]
    fn absent_document_keeps_the_native_format_error() {
        let error = from_json(r#"{"format":"anicraft","version":3}"#).err().map(|e| e.to_string()).unwrap_or_default();
        assert!(error.contains("the file has no document"), "{error}");
    }

    #[test]
    fn version_1_files_still_load() {
        let v1 = r#"{"format":"anicraft","version":1,"document":{"timeline":{"layers":[{"id":1,"name":"L","keyframes":[{"start":0,"duration":2,"elements":[
            {"id":2,"kind":{"Shape":{"path":{"cmds":[{"MoveTo":{"x":0.0,"y":0.0}},{"LineTo":{"x":10.0,"y":0.0}},{"LineTo":{"x":10.0,"y":10.0}},"Close"]},"fill":{"Solid":{"r":1.0,"g":0.0,"b":0.0,"a":1.0}}}}}]}]}]},"next_id":3}}"#;
        let d = from_json(v1).unwrap();
        assert_eq!(d.frame_count(), 2);
        assert_eq!(anima_doc::evaluate(&d, 1).items.len(), 1);
    }

    #[test]
    fn bad_path_references_are_errors() {
        let bad = r#"{"format":"anicraft","version":2,"paths":["M0 0L1 1Z"],"document":{"timeline":{"layers":[{"id":1,"name":"L","keyframes":[{"start":0,"duration":1,"elements":[
            {"id":2,"kind":{"Shape":{"path":7}}}]}]}]}}}"#;
        assert!(from_json(bad).is_err());
        assert!(from_json(r#"{"format":"anicraft","version":2,"paths":["M0 0 Q"],"document":{}}"#).is_err());
    }

    #[test]
    fn rejects_garbage_and_future_versions() {
        assert!(from_json("{").is_err());
        assert!(from_json(r#"{"format":"other","version":1,"document":{}}"#).is_err());
        assert!(from_json(r#"{"format":"anicraft","version":99,"document":{}}"#).is_err());
    }

    #[test]
    fn missing_fields_default() {
        let d = from_json(
            r#"{"format":"anicraft","version":1,"document":{"stage":{"width":0,"height":50,"fps":-3,"background":{"r":2,"g":0,"b":0,"a":1}}}}"#,
        )
        .unwrap();
        assert_eq!(d.stage.width, 1);
        assert_eq!(d.stage.fps, 24.0);
        assert_eq!(d.stage.background.r, 1.0);
    }
}
