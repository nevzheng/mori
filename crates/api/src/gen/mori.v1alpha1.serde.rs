impl serde::Serialize for CloneRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.url.is_empty() {
            len += 1;
        }
        if self.validate_only {
            len += 1;
        }
        if self.jj_only {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mori.v1alpha1.CloneRequest", len)?;
        if !self.url.is_empty() {
            struct_ser.serialize_field("url", &self.url)?;
        }
        if self.validate_only {
            struct_ser.serialize_field("validateOnly", &self.validate_only)?;
        }
        if self.jj_only {
            struct_ser.serialize_field("jjOnly", &self.jj_only)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CloneRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["url", "validate_only", "validateOnly", "jj_only", "jjOnly"];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Url,
            ValidateOnly,
            JjOnly,
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
                            "url" => Ok(GeneratedField::Url),
                            "validateOnly" | "validate_only" => Ok(GeneratedField::ValidateOnly),
                            "jjOnly" | "jj_only" => Ok(GeneratedField::JjOnly),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CloneRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.CloneRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CloneRequest, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut url__ = None;
                let mut validate_only__ = None;
                let mut jj_only__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Url => {
                            if url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("url"));
                            }
                            url__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ValidateOnly => {
                            if validate_only__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validateOnly"));
                            }
                            validate_only__ = Some(map_.next_value()?);
                        }
                        GeneratedField::JjOnly => {
                            if jj_only__.is_some() {
                                return Err(serde::de::Error::duplicate_field("jjOnly"));
                            }
                            jj_only__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CloneRequest {
                    url: url__.unwrap_or_default(),
                    validate_only: validate_only__.unwrap_or_default(),
                    jj_only: jj_only__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mori.v1alpha1.CloneRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CloneResponse {
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
        if !self.fetch_url.is_empty() {
            len += 1;
        }
        if self.colocated {
            len += 1;
        }
        if !self.tree_dir.is_empty() {
            len += 1;
        }
        if self.validate_only {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mori.v1alpha1.CloneResponse", len)?;
        if !self.repo.is_empty() {
            struct_ser.serialize_field("repo", &self.repo)?;
        }
        if !self.path.is_empty() {
            struct_ser.serialize_field("path", &self.path)?;
        }
        if !self.fetch_url.is_empty() {
            struct_ser.serialize_field("fetchUrl", &self.fetch_url)?;
        }
        if self.colocated {
            struct_ser.serialize_field("colocated", &self.colocated)?;
        }
        if !self.tree_dir.is_empty() {
            struct_ser.serialize_field("treeDir", &self.tree_dir)?;
        }
        if self.validate_only {
            struct_ser.serialize_field("validateOnly", &self.validate_only)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CloneResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "repo",
            "path",
            "fetch_url",
            "fetchUrl",
            "colocated",
            "tree_dir",
            "treeDir",
            "validate_only",
            "validateOnly",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Repo,
            Path,
            FetchUrl,
            Colocated,
            TreeDir,
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
                            "repo" => Ok(GeneratedField::Repo),
                            "path" => Ok(GeneratedField::Path),
                            "fetchUrl" | "fetch_url" => Ok(GeneratedField::FetchUrl),
                            "colocated" => Ok(GeneratedField::Colocated),
                            "treeDir" | "tree_dir" => Ok(GeneratedField::TreeDir),
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
            type Value = CloneResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.CloneResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CloneResponse, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut repo__ = None;
                let mut path__ = None;
                let mut fetch_url__ = None;
                let mut colocated__ = None;
                let mut tree_dir__ = None;
                let mut validate_only__ = None;
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
                        GeneratedField::FetchUrl => {
                            if fetch_url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fetchUrl"));
                            }
                            fetch_url__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Colocated => {
                            if colocated__.is_some() {
                                return Err(serde::de::Error::duplicate_field("colocated"));
                            }
                            colocated__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TreeDir => {
                            if tree_dir__.is_some() {
                                return Err(serde::de::Error::duplicate_field("treeDir"));
                            }
                            tree_dir__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ValidateOnly => {
                            if validate_only__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validateOnly"));
                            }
                            validate_only__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CloneResponse {
                    repo: repo__.unwrap_or_default(),
                    path: path__.unwrap_or_default(),
                    fetch_url: fetch_url__.unwrap_or_default(),
                    colocated: colocated__.unwrap_or_default(),
                    tree_dir: tree_dir__.unwrap_or_default(),
                    validate_only: validate_only__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mori.v1alpha1.CloneResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateTreeRequest {
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
        if !self.task.is_empty() {
            len += 1;
        }
        if !self.agent.is_empty() {
            len += 1;
        }
        if !self.lifetime.is_empty() {
            len += 1;
        }
        if !self.from.is_empty() {
            len += 1;
        }
        if self.validate_only {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mori.v1alpha1.CreateTreeRequest", len)?;
        if !self.repo.is_empty() {
            struct_ser.serialize_field("repo", &self.repo)?;
        }
        if !self.task.is_empty() {
            struct_ser.serialize_field("task", &self.task)?;
        }
        if !self.agent.is_empty() {
            struct_ser.serialize_field("agent", &self.agent)?;
        }
        if !self.lifetime.is_empty() {
            struct_ser.serialize_field("lifetime", &self.lifetime)?;
        }
        if !self.from.is_empty() {
            struct_ser.serialize_field("from", &self.from)?;
        }
        if self.validate_only {
            struct_ser.serialize_field("validateOnly", &self.validate_only)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateTreeRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "repo",
            "task",
            "agent",
            "lifetime",
            "from",
            "validate_only",
            "validateOnly",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Repo,
            Task,
            Agent,
            Lifetime,
            From,
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
                            "repo" => Ok(GeneratedField::Repo),
                            "task" => Ok(GeneratedField::Task),
                            "agent" => Ok(GeneratedField::Agent),
                            "lifetime" => Ok(GeneratedField::Lifetime),
                            "from" => Ok(GeneratedField::From),
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
            type Value = CreateTreeRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.CreateTreeRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateTreeRequest, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut repo__ = None;
                let mut task__ = None;
                let mut agent__ = None;
                let mut lifetime__ = None;
                let mut from__ = None;
                let mut validate_only__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Repo => {
                            if repo__.is_some() {
                                return Err(serde::de::Error::duplicate_field("repo"));
                            }
                            repo__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Task => {
                            if task__.is_some() {
                                return Err(serde::de::Error::duplicate_field("task"));
                            }
                            task__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Agent => {
                            if agent__.is_some() {
                                return Err(serde::de::Error::duplicate_field("agent"));
                            }
                            agent__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Lifetime => {
                            if lifetime__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lifetime"));
                            }
                            lifetime__ = Some(map_.next_value()?);
                        }
                        GeneratedField::From => {
                            if from__.is_some() {
                                return Err(serde::de::Error::duplicate_field("from"));
                            }
                            from__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ValidateOnly => {
                            if validate_only__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validateOnly"));
                            }
                            validate_only__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreateTreeRequest {
                    repo: repo__.unwrap_or_default(),
                    task: task__.unwrap_or_default(),
                    agent: agent__.unwrap_or_default(),
                    lifetime: lifetime__.unwrap_or_default(),
                    from: from__.unwrap_or_default(),
                    validate_only: validate_only__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mori.v1alpha1.CreateTreeRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateTreeResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.tree.is_some() {
            len += 1;
        }
        if !self.from.is_empty() {
            len += 1;
        }
        if self.validate_only {
            len += 1;
        }
        let mut struct_ser =
            serializer.serialize_struct("mori.v1alpha1.CreateTreeResponse", len)?;
        if let Some(v) = self.tree.as_ref() {
            struct_ser.serialize_field("tree", v)?;
        }
        if !self.from.is_empty() {
            struct_ser.serialize_field("from", &self.from)?;
        }
        if self.validate_only {
            struct_ser.serialize_field("validateOnly", &self.validate_only)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateTreeResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["tree", "from", "validate_only", "validateOnly"];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Tree,
            From,
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
                            "tree" => Ok(GeneratedField::Tree),
                            "from" => Ok(GeneratedField::From),
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
            type Value = CreateTreeResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.CreateTreeResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateTreeResponse, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut tree__ = None;
                let mut from__ = None;
                let mut validate_only__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Tree => {
                            if tree__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tree"));
                            }
                            tree__ = map_.next_value()?;
                        }
                        GeneratedField::From => {
                            if from__.is_some() {
                                return Err(serde::de::Error::duplicate_field("from"));
                            }
                            from__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ValidateOnly => {
                            if validate_only__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validateOnly"));
                            }
                            validate_only__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreateTreeResponse {
                    tree: tree__,
                    from: from__.unwrap_or_default(),
                    validate_only: validate_only__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct(
            "mori.v1alpha1.CreateTreeResponse",
            FIELDS,
            GeneratedVisitor,
        )
    }
}
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
impl serde::Serialize for ListTreesRequest {
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
        let mut struct_ser = serializer.serialize_struct("mori.v1alpha1.ListTreesRequest", len)?;
        if !self.repo.is_empty() {
            struct_ser.serialize_field("repo", &self.repo)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ListTreesRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["repo"];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Repo,
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ListTreesRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.ListTreesRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ListTreesRequest, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut repo__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Repo => {
                            if repo__.is_some() {
                                return Err(serde::de::Error::duplicate_field("repo"));
                            }
                            repo__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ListTreesRequest {
                    repo: repo__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mori.v1alpha1.ListTreesRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ListTreesResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.repos.is_empty() {
            len += 1;
        }
        if !self.unmanaged_repos.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mori.v1alpha1.ListTreesResponse", len)?;
        if !self.repos.is_empty() {
            struct_ser.serialize_field("repos", &self.repos)?;
        }
        if !self.unmanaged_repos.is_empty() {
            struct_ser.serialize_field("unmanagedRepos", &self.unmanaged_repos)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ListTreesResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["repos", "unmanaged_repos", "unmanagedRepos"];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Repos,
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
                            "repos" => Ok(GeneratedField::Repos),
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
            type Value = ListTreesResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.ListTreesResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ListTreesResponse, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut repos__ = None;
                let mut unmanaged_repos__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Repos => {
                            if repos__.is_some() {
                                return Err(serde::de::Error::duplicate_field("repos"));
                            }
                            repos__ = Some(map_.next_value()?);
                        }
                        GeneratedField::UnmanagedRepos => {
                            if unmanaged_repos__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unmanagedRepos"));
                            }
                            unmanaged_repos__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ListTreesResponse {
                    repos: repos__.unwrap_or_default(),
                    unmanaged_repos: unmanaged_repos__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mori.v1alpha1.ListTreesResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RepoTrees {
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
        if !self.trees.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mori.v1alpha1.RepoTrees", len)?;
        if !self.repo.is_empty() {
            struct_ser.serialize_field("repo", &self.repo)?;
        }
        if !self.path.is_empty() {
            struct_ser.serialize_field("path", &self.path)?;
        }
        if !self.trees.is_empty() {
            struct_ser.serialize_field("trees", &self.trees)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RepoTrees {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["repo", "path", "trees"];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Repo,
            Path,
            Trees,
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
                            "trees" => Ok(GeneratedField::Trees),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RepoTrees;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.RepoTrees")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<RepoTrees, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut repo__ = None;
                let mut path__ = None;
                let mut trees__ = None;
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
                        GeneratedField::Trees => {
                            if trees__.is_some() {
                                return Err(serde::de::Error::duplicate_field("trees"));
                            }
                            trees__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(RepoTrees {
                    repo: repo__.unwrap_or_default(),
                    path: path__.unwrap_or_default(),
                    trees: trees__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mori.v1alpha1.RepoTrees", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Tree {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.id.is_empty() {
            len += 1;
        }
        if !self.repo.is_empty() {
            len += 1;
        }
        if !self.name.is_empty() {
            len += 1;
        }
        if !self.path.is_empty() {
            len += 1;
        }
        if !self.role.is_empty() {
            len += 1;
        }
        if !self.owner.is_empty() {
            len += 1;
        }
        if !self.task.is_empty() {
            len += 1;
        }
        if !self.lifetime.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mori.v1alpha1.Tree", len)?;
        if !self.id.is_empty() {
            struct_ser.serialize_field("id", &self.id)?;
        }
        if !self.repo.is_empty() {
            struct_ser.serialize_field("repo", &self.repo)?;
        }
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.path.is_empty() {
            struct_ser.serialize_field("path", &self.path)?;
        }
        if !self.role.is_empty() {
            struct_ser.serialize_field("role", &self.role)?;
        }
        if !self.owner.is_empty() {
            struct_ser.serialize_field("owner", &self.owner)?;
        }
        if !self.task.is_empty() {
            struct_ser.serialize_field("task", &self.task)?;
        }
        if !self.lifetime.is_empty() {
            struct_ser.serialize_field("lifetime", &self.lifetime)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Tree {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id", "repo", "name", "path", "role", "owner", "task", "lifetime",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            Repo,
            Name,
            Path,
            Role,
            Owner,
            Task,
            Lifetime,
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
                            "id" => Ok(GeneratedField::Id),
                            "repo" => Ok(GeneratedField::Repo),
                            "name" => Ok(GeneratedField::Name),
                            "path" => Ok(GeneratedField::Path),
                            "role" => Ok(GeneratedField::Role),
                            "owner" => Ok(GeneratedField::Owner),
                            "task" => Ok(GeneratedField::Task),
                            "lifetime" => Ok(GeneratedField::Lifetime),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Tree;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.Tree")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Tree, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                let mut repo__ = None;
                let mut name__ = None;
                let mut path__ = None;
                let mut role__ = None;
                let mut owner__ = None;
                let mut task__ = None;
                let mut lifetime__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Repo => {
                            if repo__.is_some() {
                                return Err(serde::de::Error::duplicate_field("repo"));
                            }
                            repo__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Path => {
                            if path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("path"));
                            }
                            path__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Role => {
                            if role__.is_some() {
                                return Err(serde::de::Error::duplicate_field("role"));
                            }
                            role__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Owner => {
                            if owner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("owner"));
                            }
                            owner__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Task => {
                            if task__.is_some() {
                                return Err(serde::de::Error::duplicate_field("task"));
                            }
                            task__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Lifetime => {
                            if lifetime__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lifetime"));
                            }
                            lifetime__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Tree {
                    id: id__.unwrap_or_default(),
                    repo: repo__.unwrap_or_default(),
                    name: name__.unwrap_or_default(),
                    path: path__.unwrap_or_default(),
                    role: role__.unwrap_or_default(),
                    owner: owner__.unwrap_or_default(),
                    task: task__.unwrap_or_default(),
                    lifetime: lifetime__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mori.v1alpha1.Tree", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TreeRow {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.tree.is_some() {
            len += 1;
        }
        if self.status != 0 {
            len += 1;
        }
        if self.state.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mori.v1alpha1.TreeRow", len)?;
        if let Some(v) = self.tree.as_ref() {
            struct_ser.serialize_field("tree", v)?;
        }
        if self.status != 0 {
            let v = tree_row::Status::try_from(self.status).map_err(|_| {
                serde::ser::Error::custom(format!("Invalid variant {}", self.status))
            })?;
            struct_ser.serialize_field("status", &v)?;
        }
        if let Some(v) = self.state.as_ref() {
            struct_ser.serialize_field("state", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TreeRow {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["tree", "status", "state"];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Tree,
            Status,
            State,
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
                            "tree" => Ok(GeneratedField::Tree),
                            "status" => Ok(GeneratedField::Status),
                            "state" => Ok(GeneratedField::State),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TreeRow;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.TreeRow")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TreeRow, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut tree__ = None;
                let mut status__ = None;
                let mut state__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Tree => {
                            if tree__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tree"));
                            }
                            tree__ = map_.next_value()?;
                        }
                        GeneratedField::Status => {
                            if status__.is_some() {
                                return Err(serde::de::Error::duplicate_field("status"));
                            }
                            status__ = Some(map_.next_value::<tree_row::Status>()? as i32);
                        }
                        GeneratedField::State => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("state"));
                            }
                            state__ = map_.next_value()?;
                        }
                    }
                }
                Ok(TreeRow {
                    tree: tree__,
                    status: status__.unwrap_or_default(),
                    state: state__,
                })
            }
        }
        deserializer.deserialize_struct("mori.v1alpha1.TreeRow", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for tree_row::Status {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "STATUS_UNSPECIFIED",
            Self::Tree => "STATUS_TREE",
            Self::Missing => "STATUS_MISSING",
            Self::Foreign => "STATUS_FOREIGN",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for tree_row::Status {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "STATUS_UNSPECIFIED",
            "STATUS_TREE",
            "STATUS_MISSING",
            "STATUS_FOREIGN",
        ];

        struct GeneratedVisitor;

        impl serde::de::Visitor<'_> for GeneratedVisitor {
            type Value = tree_row::Status;

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
                    "STATUS_UNSPECIFIED" => Ok(tree_row::Status::Unspecified),
                    "STATUS_TREE" => Ok(tree_row::Status::Tree),
                    "STATUS_MISSING" => Ok(tree_row::Status::Missing),
                    "STATUS_FOREIGN" => Ok(tree_row::Status::Foreign),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for TreeState {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.change.is_empty() {
            len += 1;
        }
        if self.changed {
            len += 1;
        }
        if self.unpushed != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mori.v1alpha1.TreeState", len)?;
        if !self.change.is_empty() {
            struct_ser.serialize_field("change", &self.change)?;
        }
        if self.changed {
            struct_ser.serialize_field("changed", &self.changed)?;
        }
        if self.unpushed != 0 {
            struct_ser.serialize_field("unpushed", &self.unpushed)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TreeState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["change", "changed", "unpushed"];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Change,
            Changed,
            Unpushed,
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
                            "change" => Ok(GeneratedField::Change),
                            "changed" => Ok(GeneratedField::Changed),
                            "unpushed" => Ok(GeneratedField::Unpushed),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TreeState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.TreeState")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TreeState, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut change__ = None;
                let mut changed__ = None;
                let mut unpushed__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Change => {
                            if change__.is_some() {
                                return Err(serde::de::Error::duplicate_field("change"));
                            }
                            change__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Changed => {
                            if changed__.is_some() {
                                return Err(serde::de::Error::duplicate_field("changed"));
                            }
                            changed__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Unpushed => {
                            if unpushed__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unpushed"));
                            }
                            unpushed__ = Some(
                                map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?
                                    .0,
                            );
                        }
                    }
                }
                Ok(TreeState {
                    change: change__.unwrap_or_default(),
                    changed: changed__.unwrap_or_default(),
                    unpushed: unpushed__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mori.v1alpha1.TreeState", FIELDS, GeneratedVisitor)
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
