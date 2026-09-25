//! mori's API types, generated from `proto/mori/v1alpha1`, the single source of the API.
//!
//! Never edit `src/gen/` by hand: change the proto, then run `bazel run //tools/protogen`.
//! `//tools/protogen:up_to_date_test` fails if the generated code is stale.

/// The `mori.v1alpha1` package. Alpha: no stability guarantees; anything may change in any release.
#[allow(clippy::all, clippy::pedantic, clippy::restriction, missing_docs)]
pub mod v1alpha1 {
    include!("gen/mori.v1alpha1.rs");
    include!("gen/mori.v1alpha1.serde.rs");
}

#[cfg(test)]
mod tests {
    use super::v1alpha1::{CreatedPath, InitResponse, created_path::Kind};

    #[test]
    fn init_response_uses_the_proto3_json_mapping() {
        let response = InitResponse {
            root: "/home/acme/mori".into(),
            already_initialized: true,
            validate_only: false,
            created: vec![CreatedPath {
                path: "/home/acme/mori/repos".into(),
                kind: Kind::Directory.into(),
            }],
            unmanaged_repos: vec![],
        };

        let json = serde_json::to_value(&response).unwrap();

        // lowerCamelCase names, enums as strings, default values omitted.
        assert_eq!(
            json,
            serde_json::json!({
                "root": "/home/acme/mori",
                "alreadyInitialized": true,
                "created": [{"path": "/home/acme/mori/repos", "kind": "KIND_DIRECTORY"}],
            })
        );
    }
}
