impl serde::Serialize for CreatedPath {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.path.is_empty() {
            len += 1;
        }
        if self.kind != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mori.v1alpha1.CreatedPath", len)?;
        if !self.path.is_empty() {
            struct_ser.serialize_field("path", &self.path)?;
        }
        if self.kind != 0 {
            let v = created_path::Kind::try_from(self.kind)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.kind)))?;
            struct_ser.serialize_field("kind", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreatedPath {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["path", "kind"];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Path,
            Kind,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(
                        &self,
                        formatter: &mut std::fmt::Formatter<'_>,
                    ) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "path" => Ok(GeneratedField::Path),
                            "kind" => Ok(GeneratedField::Kind),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreatedPath;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.CreatedPath")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreatedPath, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut path__ = None;
                let mut kind__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Path => {
                            if path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("path"));
                            }
                            path__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Kind => {
                            if kind__.is_some() {
                                return Err(serde::de::Error::duplicate_field("kind"));
                            }
                            kind__ = Some(map_.next_value::<created_path::Kind>()? as i32);
                        }
                    }
                }
                Ok(CreatedPath {
                    path: path__.unwrap_or_default(),
                    kind: kind__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mori.v1alpha1.CreatedPath", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for created_path::Kind {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "KIND_UNSPECIFIED",
            Self::Directory => "KIND_DIRECTORY",
            Self::File => "KIND_FILE",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for created_path::Kind {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["KIND_UNSPECIFIED", "KIND_DIRECTORY", "KIND_FILE"];

        struct GeneratedVisitor;

        impl serde::de::Visitor<'_> for GeneratedVisitor {
            type Value = created_path::Kind;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "KIND_UNSPECIFIED" => Ok(created_path::Kind::Unspecified),
                    "KIND_DIRECTORY" => Ok(created_path::Kind::Directory),
                    "KIND_FILE" => Ok(created_path::Kind::File),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for InitRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.validate_only {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mori.v1alpha1.InitRequest", len)?;
        if self.validate_only {
            struct_ser.serialize_field("validateOnly", &self.validate_only)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for InitRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["validate_only", "validateOnly"];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ValidateOnly,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(
                        &self,
                        formatter: &mut std::fmt::Formatter<'_>,
                    ) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "validateOnly" | "validate_only" => Ok(GeneratedField::ValidateOnly),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = InitRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.InitRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<InitRequest, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut validate_only__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ValidateOnly => {
                            if validate_only__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validateOnly"));
                            }
                            validate_only__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(InitRequest {
                    validate_only: validate_only__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mori.v1alpha1.InitRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for InitResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.root.is_empty() {
            len += 1;
        }
        if self.already_initialized {
            len += 1;
        }
        if self.validate_only {
            len += 1;
        }
        if !self.created.is_empty() {
            len += 1;
        }
        if !self.unmanaged_repos.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mori.v1alpha1.InitResponse", len)?;
        if !self.root.is_empty() {
            struct_ser.serialize_field("root", &self.root)?;
        }
        if self.already_initialized {
            struct_ser.serialize_field("alreadyInitialized", &self.already_initialized)?;
        }
        if self.validate_only {
            struct_ser.serialize_field("validateOnly", &self.validate_only)?;
        }
        if !self.created.is_empty() {
            struct_ser.serialize_field("created", &self.created)?;
        }
        if !self.unmanaged_repos.is_empty() {
            struct_ser.serialize_field("unmanagedRepos", &self.unmanaged_repos)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for InitResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "root",
            "already_initialized",
            "alreadyInitialized",
            "validate_only",
            "validateOnly",
            "created",
            "unmanaged_repos",
            "unmanagedRepos",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Root,
            AlreadyInitialized,
            ValidateOnly,
            Created,
            UnmanagedRepos,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(
                        &self,
                        formatter: &mut std::fmt::Formatter<'_>,
                    ) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "root" => Ok(GeneratedField::Root),
                            "alreadyInitialized" | "already_initialized" => {
                                Ok(GeneratedField::AlreadyInitialized)
                            }
                            "validateOnly" | "validate_only" => Ok(GeneratedField::ValidateOnly),
                            "created" => Ok(GeneratedField::Created),
                            "unmanagedRepos" | "unmanaged_repos" => {
                                Ok(GeneratedField::UnmanagedRepos)
                            }
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = InitResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.InitResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<InitResponse, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut root__ = None;
                let mut already_initialized__ = None;
                let mut validate_only__ = None;
                let mut created__ = None;
                let mut unmanaged_repos__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Root => {
                            if root__.is_some() {
                                return Err(serde::de::Error::duplicate_field("root"));
                            }
                            root__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AlreadyInitialized => {
                            if already_initialized__.is_some() {
                                return Err(serde::de::Error::duplicate_field(
                                    "alreadyInitialized",
                                ));
                            }
                            already_initialized__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ValidateOnly => {
                            if validate_only__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validateOnly"));
                            }
                            validate_only__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Created => {
                            if created__.is_some() {
                                return Err(serde::de::Error::duplicate_field("created"));
                            }
                            created__ = Some(map_.next_value()?);
                        }
                        GeneratedField::UnmanagedRepos => {
                            if unmanaged_repos__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unmanagedRepos"));
                            }
                            unmanaged_repos__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(InitResponse {
                    root: root__.unwrap_or_default(),
                    already_initialized: already_initialized__.unwrap_or_default(),
                    validate_only: validate_only__.unwrap_or_default(),
                    created: created__.unwrap_or_default(),
                    unmanaged_repos: unmanaged_repos__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mori.v1alpha1.InitResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UnmanagedRepo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.repo.is_empty() {
            len += 1;
        }
        if !self.path.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mori.v1alpha1.UnmanagedRepo", len)?;
        if !self.repo.is_empty() {
            struct_ser.serialize_field("repo", &self.repo)?;
        }
        if !self.path.is_empty() {
            struct_ser.serialize_field("path", &self.path)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UnmanagedRepo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["repo", "path"];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Repo,
            Path,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(
                        &self,
                        formatter: &mut std::fmt::Formatter<'_>,
                    ) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "repo" => Ok(GeneratedField::Repo),
                            "path" => Ok(GeneratedField::Path),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UnmanagedRepo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.UnmanagedRepo")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UnmanagedRepo, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut repo__ = None;
                let mut path__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Repo => {
                            if repo__.is_some() {
                                return Err(serde::de::Error::duplicate_field("repo"));
                            }
                            repo__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Path => {
                            if path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("path"));
                            }
                            path__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(UnmanagedRepo {
                    repo: repo__.unwrap_or_default(),
                    path: path__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mori.v1alpha1.UnmanagedRepo", FIELDS, GeneratedVisitor)
    }
}
