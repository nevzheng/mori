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
        if self.vcs != 0 {
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
        if self.vcs != 0 {
            let v = Vcs::try_from(self.vcs)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.vcs)))?;
            struct_ser.serialize_field("vcs", &v)?;
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
        const FIELDS: &[&str] = &[
            "url",
            "validate_only",
            "validateOnly",
            "jj_only",
            "jjOnly",
            "vcs",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Url,
            ValidateOnly,
            JjOnly,
            Vcs,
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
                            "vcs" => Ok(GeneratedField::Vcs),
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
                let mut vcs__ = None;
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
                        GeneratedField::Vcs => {
                            if vcs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vcs"));
                            }
                            vcs__ = Some(map_.next_value::<Vcs>()? as i32);
                        }
                    }
                }
                Ok(CloneRequest {
                    url: url__.unwrap_or_default(),
                    validate_only: validate_only__.unwrap_or_default(),
                    jj_only: jj_only__.unwrap_or_default(),
                    vcs: vcs__.unwrap_or_default(),
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
        if !self.context_dir.is_empty() {
            len += 1;
        }
        if self.vcs != 0 {
            len += 1;
        }
        if !self.warnings.is_empty() {
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
        if !self.context_dir.is_empty() {
            struct_ser.serialize_field("contextDir", &self.context_dir)?;
        }
        if self.vcs != 0 {
            let v = Vcs::try_from(self.vcs)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.vcs)))?;
            struct_ser.serialize_field("vcs", &v)?;
        }
        if !self.warnings.is_empty() {
            struct_ser.serialize_field("warnings", &self.warnings)?;
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
            "context_dir",
            "contextDir",
            "vcs",
            "warnings",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Repo,
            Path,
            FetchUrl,
            Colocated,
            TreeDir,
            ValidateOnly,
            ContextDir,
            Vcs,
            Warnings,
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
                            "contextDir" | "context_dir" => Ok(GeneratedField::ContextDir),
                            "vcs" => Ok(GeneratedField::Vcs),
                            "warnings" => Ok(GeneratedField::Warnings),
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
                let mut context_dir__ = None;
                let mut vcs__ = None;
                let mut warnings__ = None;
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
                        GeneratedField::ContextDir => {
                            if context_dir__.is_some() {
                                return Err(serde::de::Error::duplicate_field("contextDir"));
                            }
                            context_dir__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Vcs => {
                            if vcs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vcs"));
                            }
                            vcs__ = Some(map_.next_value::<Vcs>()? as i32);
                        }
                        GeneratedField::Warnings => {
                            if warnings__.is_some() {
                                return Err(serde::de::Error::duplicate_field("warnings"));
                            }
                            warnings__ = Some(map_.next_value()?);
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
                    context_dir: context_dir__.unwrap_or_default(),
                    vcs: vcs__.unwrap_or_default(),
                    warnings: warnings__.unwrap_or_default(),
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
        if self.vcs != 0 {
            len += 1;
        }
        if !self.warnings.is_empty() {
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
        if self.vcs != 0 {
            let v = Vcs::try_from(self.vcs)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.vcs)))?;
            struct_ser.serialize_field("vcs", &v)?;
        }
        if !self.warnings.is_empty() {
            struct_ser.serialize_field("warnings", &self.warnings)?;
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
        const FIELDS: &[&str] = &[
            "tree",
            "from",
            "validate_only",
            "validateOnly",
            "vcs",
            "warnings",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Tree,
            From,
            ValidateOnly,
            Vcs,
            Warnings,
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
                            "vcs" => Ok(GeneratedField::Vcs),
                            "warnings" => Ok(GeneratedField::Warnings),
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
                let mut vcs__ = None;
                let mut warnings__ = None;
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
                        GeneratedField::Vcs => {
                            if vcs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vcs"));
                            }
                            vcs__ = Some(map_.next_value::<Vcs>()? as i32);
                        }
                        GeneratedField::Warnings => {
                            if warnings__.is_some() {
                                return Err(serde::de::Error::duplicate_field("warnings"));
                            }
                            warnings__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreateTreeResponse {
                    tree: tree__,
                    from: from__.unwrap_or_default(),
                    validate_only: validate_only__.unwrap_or_default(),
                    vcs: vcs__.unwrap_or_default(),
                    warnings: warnings__.unwrap_or_default(),
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
impl serde::Serialize for Disk {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.free_bytes != 0 {
            len += 1;
        }
        if self.total_bytes != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mori.v1alpha1.Disk", len)?;
        if self.free_bytes != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser
                .serialize_field("freeBytes", ToString::to_string(&self.free_bytes).as_str())?;
        }
        if self.total_bytes != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field(
                "totalBytes",
                ToString::to_string(&self.total_bytes).as_str(),
            )?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Disk {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["free_bytes", "freeBytes", "total_bytes", "totalBytes"];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FreeBytes,
            TotalBytes,
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
                            "freeBytes" | "free_bytes" => Ok(GeneratedField::FreeBytes),
                            "totalBytes" | "total_bytes" => Ok(GeneratedField::TotalBytes),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Disk;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.Disk")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Disk, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut free_bytes__ = None;
                let mut total_bytes__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::FreeBytes => {
                            if free_bytes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("freeBytes"));
                            }
                            free_bytes__ = Some(
                                map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?
                                    .0,
                            );
                        }
                        GeneratedField::TotalBytes => {
                            if total_bytes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("totalBytes"));
                            }
                            total_bytes__ = Some(
                                map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?
                                    .0,
                            );
                        }
                    }
                }
                Ok(Disk {
                    free_bytes: free_bytes__.unwrap_or_default(),
                    total_bytes: total_bytes__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mori.v1alpha1.Disk", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DoctorRequest {
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
        if self.fix {
            len += 1;
        }
        if self.confirmed {
            len += 1;
        }
        if self.validate_only {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mori.v1alpha1.DoctorRequest", len)?;
        if !self.repo.is_empty() {
            struct_ser.serialize_field("repo", &self.repo)?;
        }
        if self.fix {
            struct_ser.serialize_field("fix", &self.fix)?;
        }
        if self.confirmed {
            struct_ser.serialize_field("confirmed", &self.confirmed)?;
        }
        if self.validate_only {
            struct_ser.serialize_field("validateOnly", &self.validate_only)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DoctorRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["repo", "fix", "confirmed", "validate_only", "validateOnly"];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Repo,
            Fix,
            Confirmed,
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
                            "fix" => Ok(GeneratedField::Fix),
                            "confirmed" => Ok(GeneratedField::Confirmed),
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
            type Value = DoctorRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.DoctorRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DoctorRequest, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut repo__ = None;
                let mut fix__ = None;
                let mut confirmed__ = None;
                let mut validate_only__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Repo => {
                            if repo__.is_some() {
                                return Err(serde::de::Error::duplicate_field("repo"));
                            }
                            repo__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Fix => {
                            if fix__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fix"));
                            }
                            fix__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Confirmed => {
                            if confirmed__.is_some() {
                                return Err(serde::de::Error::duplicate_field("confirmed"));
                            }
                            confirmed__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ValidateOnly => {
                            if validate_only__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validateOnly"));
                            }
                            validate_only__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(DoctorRequest {
                    repo: repo__.unwrap_or_default(),
                    fix: fix__.unwrap_or_default(),
                    confirmed: confirmed__.unwrap_or_default(),
                    validate_only: validate_only__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mori.v1alpha1.DoctorRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DoctorResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.findings.is_empty() {
            len += 1;
        }
        if !self.summary.is_empty() {
            len += 1;
        }
        if !self.fixed.is_empty() {
            len += 1;
        }
        if self.validate_only {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mori.v1alpha1.DoctorResponse", len)?;
        if !self.findings.is_empty() {
            struct_ser.serialize_field("findings", &self.findings)?;
        }
        if !self.summary.is_empty() {
            struct_ser.serialize_field("summary", &self.summary)?;
        }
        if !self.fixed.is_empty() {
            struct_ser.serialize_field("fixed", &self.fixed)?;
        }
        if self.validate_only {
            struct_ser.serialize_field("validateOnly", &self.validate_only)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DoctorResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "findings",
            "summary",
            "fixed",
            "validate_only",
            "validateOnly",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Findings,
            Summary,
            Fixed,
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
                            "findings" => Ok(GeneratedField::Findings),
                            "summary" => Ok(GeneratedField::Summary),
                            "fixed" => Ok(GeneratedField::Fixed),
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
            type Value = DoctorResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.DoctorResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DoctorResponse, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut findings__ = None;
                let mut summary__ = None;
                let mut fixed__ = None;
                let mut validate_only__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Findings => {
                            if findings__.is_some() {
                                return Err(serde::de::Error::duplicate_field("findings"));
                            }
                            findings__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Summary => {
                            if summary__.is_some() {
                                return Err(serde::de::Error::duplicate_field("summary"));
                            }
                            summary__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Fixed => {
                            if fixed__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fixed"));
                            }
                            fixed__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ValidateOnly => {
                            if validate_only__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validateOnly"));
                            }
                            validate_only__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(DoctorResponse {
                    findings: findings__.unwrap_or_default(),
                    summary: summary__.unwrap_or_default(),
                    fixed: fixed__.unwrap_or_default(),
                    validate_only: validate_only__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mori.v1alpha1.DoctorResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Finding {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.code.is_empty() {
            len += 1;
        }
        if self.severity != 0 {
            len += 1;
        }
        if !self.subject.is_empty() {
            len += 1;
        }
        if !self.message.is_empty() {
            len += 1;
        }
        if !self.fix.is_empty() {
            len += 1;
        }
        if self.auto_fixable {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mori.v1alpha1.Finding", len)?;
        if !self.code.is_empty() {
            struct_ser.serialize_field("code", &self.code)?;
        }
        if self.severity != 0 {
            let v = finding::Severity::try_from(self.severity).map_err(|_| {
                serde::ser::Error::custom(format!("Invalid variant {}", self.severity))
            })?;
            struct_ser.serialize_field("severity", &v)?;
        }
        if !self.subject.is_empty() {
            struct_ser.serialize_field("subject", &self.subject)?;
        }
        if !self.message.is_empty() {
            struct_ser.serialize_field("message", &self.message)?;
        }
        if !self.fix.is_empty() {
            struct_ser.serialize_field("fix", &self.fix)?;
        }
        if self.auto_fixable {
            struct_ser.serialize_field("autoFixable", &self.auto_fixable)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Finding {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "code",
            "severity",
            "subject",
            "message",
            "fix",
            "auto_fixable",
            "autoFixable",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Code,
            Severity,
            Subject,
            Message,
            Fix,
            AutoFixable,
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
                            "code" => Ok(GeneratedField::Code),
                            "severity" => Ok(GeneratedField::Severity),
                            "subject" => Ok(GeneratedField::Subject),
                            "message" => Ok(GeneratedField::Message),
                            "fix" => Ok(GeneratedField::Fix),
                            "autoFixable" | "auto_fixable" => Ok(GeneratedField::AutoFixable),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Finding;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.Finding")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Finding, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut code__ = None;
                let mut severity__ = None;
                let mut subject__ = None;
                let mut message__ = None;
                let mut fix__ = None;
                let mut auto_fixable__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Code => {
                            if code__.is_some() {
                                return Err(serde::de::Error::duplicate_field("code"));
                            }
                            code__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Severity => {
                            if severity__.is_some() {
                                return Err(serde::de::Error::duplicate_field("severity"));
                            }
                            severity__ = Some(map_.next_value::<finding::Severity>()? as i32);
                        }
                        GeneratedField::Subject => {
                            if subject__.is_some() {
                                return Err(serde::de::Error::duplicate_field("subject"));
                            }
                            subject__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Message => {
                            if message__.is_some() {
                                return Err(serde::de::Error::duplicate_field("message"));
                            }
                            message__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Fix => {
                            if fix__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fix"));
                            }
                            fix__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AutoFixable => {
                            if auto_fixable__.is_some() {
                                return Err(serde::de::Error::duplicate_field("autoFixable"));
                            }
                            auto_fixable__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Finding {
                    code: code__.unwrap_or_default(),
                    severity: severity__.unwrap_or_default(),
                    subject: subject__.unwrap_or_default(),
                    message: message__.unwrap_or_default(),
                    fix: fix__.unwrap_or_default(),
                    auto_fixable: auto_fixable__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mori.v1alpha1.Finding", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for finding::Severity {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "SEVERITY_UNSPECIFIED",
            Self::Info => "SEVERITY_INFO",
            Self::Warn => "SEVERITY_WARN",
            Self::Problem => "SEVERITY_PROBLEM",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for finding::Severity {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "SEVERITY_UNSPECIFIED",
            "SEVERITY_INFO",
            "SEVERITY_WARN",
            "SEVERITY_PROBLEM",
        ];

        struct GeneratedVisitor;

        impl serde::de::Visitor<'_> for GeneratedVisitor {
            type Value = finding::Severity;

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
                    "SEVERITY_UNSPECIFIED" => Ok(finding::Severity::Unspecified),
                    "SEVERITY_INFO" => Ok(finding::Severity::Info),
                    "SEVERITY_WARN" => Ok(finding::Severity::Warn),
                    "SEVERITY_PROBLEM" => Ok(finding::Severity::Problem),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for Fixed {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.code.is_empty() {
            len += 1;
        }
        if !self.subject.is_empty() {
            len += 1;
        }
        if !self.journal_entry.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mori.v1alpha1.Fixed", len)?;
        if !self.code.is_empty() {
            struct_ser.serialize_field("code", &self.code)?;
        }
        if !self.subject.is_empty() {
            struct_ser.serialize_field("subject", &self.subject)?;
        }
        if !self.journal_entry.is_empty() {
            struct_ser.serialize_field("journalEntry", &self.journal_entry)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Fixed {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["code", "subject", "journal_entry", "journalEntry"];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Code,
            Subject,
            JournalEntry,
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
                            "code" => Ok(GeneratedField::Code),
                            "subject" => Ok(GeneratedField::Subject),
                            "journalEntry" | "journal_entry" => Ok(GeneratedField::JournalEntry),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Fixed;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.Fixed")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Fixed, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut code__ = None;
                let mut subject__ = None;
                let mut journal_entry__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Code => {
                            if code__.is_some() {
                                return Err(serde::de::Error::duplicate_field("code"));
                            }
                            code__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Subject => {
                            if subject__.is_some() {
                                return Err(serde::de::Error::duplicate_field("subject"));
                            }
                            subject__ = Some(map_.next_value()?);
                        }
                        GeneratedField::JournalEntry => {
                            if journal_entry__.is_some() {
                                return Err(serde::de::Error::duplicate_field("journalEntry"));
                            }
                            journal_entry__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Fixed {
                    code: code__.unwrap_or_default(),
                    subject: subject__.unwrap_or_default(),
                    journal_entry: journal_entry__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mori.v1alpha1.Fixed", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GcItem {
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
        if !self.name.is_empty() {
            len += 1;
        }
        if !self.path.is_empty() {
            len += 1;
        }
        if self.class != 0 {
            len += 1;
        }
        if !self.reason.is_empty() {
            len += 1;
        }
        if !self.facts.is_empty() {
            len += 1;
        }
        if self.outcome != 0 {
            len += 1;
        }
        if !self.entry_id.is_empty() {
            len += 1;
        }
        if self.size_bytes != 0 {
            len += 1;
        }
        if self.kind != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mori.v1alpha1.GcItem", len)?;
        if !self.repo.is_empty() {
            struct_ser.serialize_field("repo", &self.repo)?;
        }
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.path.is_empty() {
            struct_ser.serialize_field("path", &self.path)?;
        }
        if self.class != 0 {
            let v = gc_item::Class::try_from(self.class).map_err(|_| {
                serde::ser::Error::custom(format!("Invalid variant {}", self.class))
            })?;
            struct_ser.serialize_field("class", &v)?;
        }
        if !self.reason.is_empty() {
            struct_ser.serialize_field("reason", &self.reason)?;
        }
        if !self.facts.is_empty() {
            struct_ser.serialize_field("facts", &self.facts)?;
        }
        if self.outcome != 0 {
            let v = gc_item::Outcome::try_from(self.outcome).map_err(|_| {
                serde::ser::Error::custom(format!("Invalid variant {}", self.outcome))
            })?;
            struct_ser.serialize_field("outcome", &v)?;
        }
        if !self.entry_id.is_empty() {
            struct_ser.serialize_field("entryId", &self.entry_id)?;
        }
        if self.size_bytes != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser
                .serialize_field("sizeBytes", ToString::to_string(&self.size_bytes).as_str())?;
        }
        if self.kind != 0 {
            let v = gc_item::Kind::try_from(self.kind)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.kind)))?;
            struct_ser.serialize_field("kind", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GcItem {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "repo",
            "name",
            "path",
            "class",
            "reason",
            "facts",
            "outcome",
            "entry_id",
            "entryId",
            "size_bytes",
            "sizeBytes",
            "kind",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Repo,
            Name,
            Path,
            Class,
            Reason,
            Facts,
            Outcome,
            EntryId,
            SizeBytes,
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
                            "repo" => Ok(GeneratedField::Repo),
                            "name" => Ok(GeneratedField::Name),
                            "path" => Ok(GeneratedField::Path),
                            "class" => Ok(GeneratedField::Class),
                            "reason" => Ok(GeneratedField::Reason),
                            "facts" => Ok(GeneratedField::Facts),
                            "outcome" => Ok(GeneratedField::Outcome),
                            "entryId" | "entry_id" => Ok(GeneratedField::EntryId),
                            "sizeBytes" | "size_bytes" => Ok(GeneratedField::SizeBytes),
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
            type Value = GcItem;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.GcItem")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GcItem, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut repo__ = None;
                let mut name__ = None;
                let mut path__ = None;
                let mut class__ = None;
                let mut reason__ = None;
                let mut facts__ = None;
                let mut outcome__ = None;
                let mut entry_id__ = None;
                let mut size_bytes__ = None;
                let mut kind__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
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
                        GeneratedField::Class => {
                            if class__.is_some() {
                                return Err(serde::de::Error::duplicate_field("class"));
                            }
                            class__ = Some(map_.next_value::<gc_item::Class>()? as i32);
                        }
                        GeneratedField::Reason => {
                            if reason__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reason"));
                            }
                            reason__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Facts => {
                            if facts__.is_some() {
                                return Err(serde::de::Error::duplicate_field("facts"));
                            }
                            facts__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Outcome => {
                            if outcome__.is_some() {
                                return Err(serde::de::Error::duplicate_field("outcome"));
                            }
                            outcome__ = Some(map_.next_value::<gc_item::Outcome>()? as i32);
                        }
                        GeneratedField::EntryId => {
                            if entry_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("entryId"));
                            }
                            entry_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SizeBytes => {
                            if size_bytes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sizeBytes"));
                            }
                            size_bytes__ = Some(
                                map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?
                                    .0,
                            );
                        }
                        GeneratedField::Kind => {
                            if kind__.is_some() {
                                return Err(serde::de::Error::duplicate_field("kind"));
                            }
                            kind__ = Some(map_.next_value::<gc_item::Kind>()? as i32);
                        }
                    }
                }
                Ok(GcItem {
                    repo: repo__.unwrap_or_default(),
                    name: name__.unwrap_or_default(),
                    path: path__.unwrap_or_default(),
                    class: class__.unwrap_or_default(),
                    reason: reason__.unwrap_or_default(),
                    facts: facts__.unwrap_or_default(),
                    outcome: outcome__.unwrap_or_default(),
                    entry_id: entry_id__.unwrap_or_default(),
                    size_bytes: size_bytes__.unwrap_or_default(),
                    kind: kind__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mori.v1alpha1.GcItem", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for gc_item::Class {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "CLASS_UNSPECIFIED",
            Self::Remove => "CLASS_REMOVE",
            Self::Blocked => "CLASS_BLOCKED",
            Self::Keep => "CLASS_KEEP",
            Self::Never => "CLASS_NEVER",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for gc_item::Class {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "CLASS_UNSPECIFIED",
            "CLASS_REMOVE",
            "CLASS_BLOCKED",
            "CLASS_KEEP",
            "CLASS_NEVER",
        ];

        struct GeneratedVisitor;

        impl serde::de::Visitor<'_> for GeneratedVisitor {
            type Value = gc_item::Class;

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
                    "CLASS_UNSPECIFIED" => Ok(gc_item::Class::Unspecified),
                    "CLASS_REMOVE" => Ok(gc_item::Class::Remove),
                    "CLASS_BLOCKED" => Ok(gc_item::Class::Blocked),
                    "CLASS_KEEP" => Ok(gc_item::Class::Keep),
                    "CLASS_NEVER" => Ok(gc_item::Class::Never),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for gc_item::Kind {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "KIND_UNSPECIFIED",
            Self::Tree => "KIND_TREE",
            Self::BazelLeftover => "KIND_BAZEL_LEFTOVER",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for gc_item::Kind {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["KIND_UNSPECIFIED", "KIND_TREE", "KIND_BAZEL_LEFTOVER"];

        struct GeneratedVisitor;

        impl serde::de::Visitor<'_> for GeneratedVisitor {
            type Value = gc_item::Kind;

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
                    "KIND_UNSPECIFIED" => Ok(gc_item::Kind::Unspecified),
                    "KIND_TREE" => Ok(gc_item::Kind::Tree),
                    "KIND_BAZEL_LEFTOVER" => Ok(gc_item::Kind::BazelLeftover),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for gc_item::Outcome {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "OUTCOME_UNSPECIFIED",
            Self::Removed => "OUTCOME_REMOVED",
            Self::WouldRemove => "OUTCOME_WOULD_REMOVE",
            Self::SkippedChanged => "OUTCOME_SKIPPED_CHANGED",
            Self::SkippedUnsaved => "OUTCOME_SKIPPED_UNSAVED",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for gc_item::Outcome {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "OUTCOME_UNSPECIFIED",
            "OUTCOME_REMOVED",
            "OUTCOME_WOULD_REMOVE",
            "OUTCOME_SKIPPED_CHANGED",
            "OUTCOME_SKIPPED_UNSAVED",
        ];

        struct GeneratedVisitor;

        impl serde::de::Visitor<'_> for GeneratedVisitor {
            type Value = gc_item::Outcome;

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
                    "OUTCOME_UNSPECIFIED" => Ok(gc_item::Outcome::Unspecified),
                    "OUTCOME_REMOVED" => Ok(gc_item::Outcome::Removed),
                    "OUTCOME_WOULD_REMOVE" => Ok(gc_item::Outcome::WouldRemove),
                    "OUTCOME_SKIPPED_CHANGED" => Ok(gc_item::Outcome::SkippedChanged),
                    "OUTCOME_SKIPPED_UNSAVED" => Ok(gc_item::Outcome::SkippedUnsaved),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for GcRequest {
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
        if self.offline {
            len += 1;
        }
        if self.apply {
            len += 1;
        }
        if self.confirmed {
            len += 1;
        }
        if !self.names.is_empty() {
            len += 1;
        }
        if self.max != 0 {
            len += 1;
        }
        if self.validate_only {
            len += 1;
        }
        if self.free_target_bytes != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mori.v1alpha1.GcRequest", len)?;
        if !self.repo.is_empty() {
            struct_ser.serialize_field("repo", &self.repo)?;
        }
        if self.offline {
            struct_ser.serialize_field("offline", &self.offline)?;
        }
        if self.apply {
            struct_ser.serialize_field("apply", &self.apply)?;
        }
        if self.confirmed {
            struct_ser.serialize_field("confirmed", &self.confirmed)?;
        }
        if !self.names.is_empty() {
            struct_ser.serialize_field("names", &self.names)?;
        }
        if self.max != 0 {
            struct_ser.serialize_field("max", &self.max)?;
        }
        if self.validate_only {
            struct_ser.serialize_field("validateOnly", &self.validate_only)?;
        }
        if self.free_target_bytes != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field(
                "freeTargetBytes",
                ToString::to_string(&self.free_target_bytes).as_str(),
            )?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GcRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "repo",
            "offline",
            "apply",
            "confirmed",
            "names",
            "max",
            "validate_only",
            "validateOnly",
            "free_target_bytes",
            "freeTargetBytes",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Repo,
            Offline,
            Apply,
            Confirmed,
            Names,
            Max,
            ValidateOnly,
            FreeTargetBytes,
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
                            "offline" => Ok(GeneratedField::Offline),
                            "apply" => Ok(GeneratedField::Apply),
                            "confirmed" => Ok(GeneratedField::Confirmed),
                            "names" => Ok(GeneratedField::Names),
                            "max" => Ok(GeneratedField::Max),
                            "validateOnly" | "validate_only" => Ok(GeneratedField::ValidateOnly),
                            "freeTargetBytes" | "free_target_bytes" => {
                                Ok(GeneratedField::FreeTargetBytes)
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
            type Value = GcRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.GcRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GcRequest, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut repo__ = None;
                let mut offline__ = None;
                let mut apply__ = None;
                let mut confirmed__ = None;
                let mut names__ = None;
                let mut max__ = None;
                let mut validate_only__ = None;
                let mut free_target_bytes__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Repo => {
                            if repo__.is_some() {
                                return Err(serde::de::Error::duplicate_field("repo"));
                            }
                            repo__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Offline => {
                            if offline__.is_some() {
                                return Err(serde::de::Error::duplicate_field("offline"));
                            }
                            offline__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Apply => {
                            if apply__.is_some() {
                                return Err(serde::de::Error::duplicate_field("apply"));
                            }
                            apply__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Confirmed => {
                            if confirmed__.is_some() {
                                return Err(serde::de::Error::duplicate_field("confirmed"));
                            }
                            confirmed__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Names => {
                            if names__.is_some() {
                                return Err(serde::de::Error::duplicate_field("names"));
                            }
                            names__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Max => {
                            if max__.is_some() {
                                return Err(serde::de::Error::duplicate_field("max"));
                            }
                            max__ = Some(
                                map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?
                                    .0,
                            );
                        }
                        GeneratedField::ValidateOnly => {
                            if validate_only__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validateOnly"));
                            }
                            validate_only__ = Some(map_.next_value()?);
                        }
                        GeneratedField::FreeTargetBytes => {
                            if free_target_bytes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("freeTargetBytes"));
                            }
                            free_target_bytes__ = Some(
                                map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?
                                    .0,
                            );
                        }
                    }
                }
                Ok(GcRequest {
                    repo: repo__.unwrap_or_default(),
                    offline: offline__.unwrap_or_default(),
                    apply: apply__.unwrap_or_default(),
                    confirmed: confirmed__.unwrap_or_default(),
                    names: names__.unwrap_or_default(),
                    max: max__.unwrap_or_default(),
                    validate_only: validate_only__.unwrap_or_default(),
                    free_target_bytes: free_target_bytes__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mori.v1alpha1.GcRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GcResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.items.is_empty() {
            len += 1;
        }
        if self.validate_only {
            len += 1;
        }
        if self.disk.is_some() {
            len += 1;
        }
        if !self.warnings.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mori.v1alpha1.GcResponse", len)?;
        if !self.items.is_empty() {
            struct_ser.serialize_field("items", &self.items)?;
        }
        if self.validate_only {
            struct_ser.serialize_field("validateOnly", &self.validate_only)?;
        }
        if let Some(v) = self.disk.as_ref() {
            struct_ser.serialize_field("disk", v)?;
        }
        if !self.warnings.is_empty() {
            struct_ser.serialize_field("warnings", &self.warnings)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GcResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["items", "validate_only", "validateOnly", "disk", "warnings"];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Items,
            ValidateOnly,
            Disk,
            Warnings,
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
                            "items" => Ok(GeneratedField::Items),
                            "validateOnly" | "validate_only" => Ok(GeneratedField::ValidateOnly),
                            "disk" => Ok(GeneratedField::Disk),
                            "warnings" => Ok(GeneratedField::Warnings),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GcResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.GcResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GcResponse, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut items__ = None;
                let mut validate_only__ = None;
                let mut disk__ = None;
                let mut warnings__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Items => {
                            if items__.is_some() {
                                return Err(serde::de::Error::duplicate_field("items"));
                            }
                            items__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ValidateOnly => {
                            if validate_only__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validateOnly"));
                            }
                            validate_only__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Disk => {
                            if disk__.is_some() {
                                return Err(serde::de::Error::duplicate_field("disk"));
                            }
                            disk__ = map_.next_value()?;
                        }
                        GeneratedField::Warnings => {
                            if warnings__.is_some() {
                                return Err(serde::de::Error::duplicate_field("warnings"));
                            }
                            warnings__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(GcResponse {
                    items: items__.unwrap_or_default(),
                    validate_only: validate_only__.unwrap_or_default(),
                    disk: disk__,
                    warnings: warnings__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mori.v1alpha1.GcResponse", FIELDS, GeneratedVisitor)
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
        if self.include_sizes {
            len += 1;
        }
        if self.skip_size_cache {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mori.v1alpha1.ListTreesRequest", len)?;
        if !self.repo.is_empty() {
            struct_ser.serialize_field("repo", &self.repo)?;
        }
        if self.include_sizes {
            struct_ser.serialize_field("includeSizes", &self.include_sizes)?;
        }
        if self.skip_size_cache {
            struct_ser.serialize_field("skipSizeCache", &self.skip_size_cache)?;
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
        const FIELDS: &[&str] = &[
            "repo",
            "include_sizes",
            "includeSizes",
            "skip_size_cache",
            "skipSizeCache",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Repo,
            IncludeSizes,
            SkipSizeCache,
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
                            "includeSizes" | "include_sizes" => Ok(GeneratedField::IncludeSizes),
                            "skipSizeCache" | "skip_size_cache" => {
                                Ok(GeneratedField::SkipSizeCache)
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
            type Value = ListTreesRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.ListTreesRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ListTreesRequest, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut repo__ = None;
                let mut include_sizes__ = None;
                let mut skip_size_cache__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Repo => {
                            if repo__.is_some() {
                                return Err(serde::de::Error::duplicate_field("repo"));
                            }
                            repo__ = Some(map_.next_value()?);
                        }
                        GeneratedField::IncludeSizes => {
                            if include_sizes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("includeSizes"));
                            }
                            include_sizes__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SkipSizeCache => {
                            if skip_size_cache__.is_some() {
                                return Err(serde::de::Error::duplicate_field("skipSizeCache"));
                            }
                            skip_size_cache__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ListTreesRequest {
                    repo: repo__.unwrap_or_default(),
                    include_sizes: include_sizes__.unwrap_or_default(),
                    skip_size_cache: skip_size_cache__.unwrap_or_default(),
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
        if self.disk.is_some() {
            len += 1;
        }
        if !self.warnings.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mori.v1alpha1.ListTreesResponse", len)?;
        if !self.repos.is_empty() {
            struct_ser.serialize_field("repos", &self.repos)?;
        }
        if !self.unmanaged_repos.is_empty() {
            struct_ser.serialize_field("unmanagedRepos", &self.unmanaged_repos)?;
        }
        if let Some(v) = self.disk.as_ref() {
            struct_ser.serialize_field("disk", v)?;
        }
        if !self.warnings.is_empty() {
            struct_ser.serialize_field("warnings", &self.warnings)?;
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
        const FIELDS: &[&str] = &[
            "repos",
            "unmanaged_repos",
            "unmanagedRepos",
            "disk",
            "warnings",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Repos,
            UnmanagedRepos,
            Disk,
            Warnings,
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
                            "disk" => Ok(GeneratedField::Disk),
                            "warnings" => Ok(GeneratedField::Warnings),
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
                let mut disk__ = None;
                let mut warnings__ = None;
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
                        GeneratedField::Disk => {
                            if disk__.is_some() {
                                return Err(serde::de::Error::duplicate_field("disk"));
                            }
                            disk__ = map_.next_value()?;
                        }
                        GeneratedField::Warnings => {
                            if warnings__.is_some() {
                                return Err(serde::de::Error::duplicate_field("warnings"));
                            }
                            warnings__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ListTreesResponse {
                    repos: repos__.unwrap_or_default(),
                    unmanaged_repos: unmanaged_repos__.unwrap_or_default(),
                    disk: disk__,
                    warnings: warnings__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mori.v1alpha1.ListTreesResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PushedBookmark {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.remote.is_empty() {
            len += 1;
        }
        if !self.name.is_empty() {
            len += 1;
        }
        if !self.commit_id.is_empty() {
            len += 1;
        }
        if self.on_remote {
            len += 1;
        }
        if self.landed {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mori.v1alpha1.PushedBookmark", len)?;
        if !self.remote.is_empty() {
            struct_ser.serialize_field("remote", &self.remote)?;
        }
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.commit_id.is_empty() {
            struct_ser.serialize_field("commitId", &self.commit_id)?;
        }
        if self.on_remote {
            struct_ser.serialize_field("onRemote", &self.on_remote)?;
        }
        if self.landed {
            struct_ser.serialize_field("landed", &self.landed)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PushedBookmark {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "remote",
            "name",
            "commit_id",
            "commitId",
            "on_remote",
            "onRemote",
            "landed",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Remote,
            Name,
            CommitId,
            OnRemote,
            Landed,
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
                            "remote" => Ok(GeneratedField::Remote),
                            "name" => Ok(GeneratedField::Name),
                            "commitId" | "commit_id" => Ok(GeneratedField::CommitId),
                            "onRemote" | "on_remote" => Ok(GeneratedField::OnRemote),
                            "landed" => Ok(GeneratedField::Landed),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PushedBookmark;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.PushedBookmark")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<PushedBookmark, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut remote__ = None;
                let mut name__ = None;
                let mut commit_id__ = None;
                let mut on_remote__ = None;
                let mut landed__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Remote => {
                            if remote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("remote"));
                            }
                            remote__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CommitId => {
                            if commit_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitId"));
                            }
                            commit_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::OnRemote => {
                            if on_remote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("onRemote"));
                            }
                            on_remote__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Landed => {
                            if landed__.is_some() {
                                return Err(serde::de::Error::duplicate_field("landed"));
                            }
                            landed__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(PushedBookmark {
                    remote: remote__.unwrap_or_default(),
                    name: name__.unwrap_or_default(),
                    commit_id: commit_id__.unwrap_or_default(),
                    on_remote: on_remote__.unwrap_or_default(),
                    landed: landed__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mori.v1alpha1.PushedBookmark", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RemoveTreeRequest {
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
        if !self.name.is_empty() {
            len += 1;
        }
        if !self.agent.is_empty() {
            len += 1;
        }
        if self.pinned {
            len += 1;
        }
        if self.validate_only {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mori.v1alpha1.RemoveTreeRequest", len)?;
        if !self.repo.is_empty() {
            struct_ser.serialize_field("repo", &self.repo)?;
        }
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.agent.is_empty() {
            struct_ser.serialize_field("agent", &self.agent)?;
        }
        if self.pinned {
            struct_ser.serialize_field("pinned", &self.pinned)?;
        }
        if self.validate_only {
            struct_ser.serialize_field("validateOnly", &self.validate_only)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RemoveTreeRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "repo",
            "name",
            "agent",
            "pinned",
            "validate_only",
            "validateOnly",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Repo,
            Name,
            Agent,
            Pinned,
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
                            "name" => Ok(GeneratedField::Name),
                            "agent" => Ok(GeneratedField::Agent),
                            "pinned" => Ok(GeneratedField::Pinned),
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
            type Value = RemoveTreeRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.RemoveTreeRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<RemoveTreeRequest, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut repo__ = None;
                let mut name__ = None;
                let mut agent__ = None;
                let mut pinned__ = None;
                let mut validate_only__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
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
                        GeneratedField::Agent => {
                            if agent__.is_some() {
                                return Err(serde::de::Error::duplicate_field("agent"));
                            }
                            agent__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Pinned => {
                            if pinned__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pinned"));
                            }
                            pinned__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ValidateOnly => {
                            if validate_only__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validateOnly"));
                            }
                            validate_only__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(RemoveTreeRequest {
                    repo: repo__.unwrap_or_default(),
                    name: name__.unwrap_or_default(),
                    agent: agent__.unwrap_or_default(),
                    pinned: pinned__.unwrap_or_default(),
                    validate_only: validate_only__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mori.v1alpha1.RemoveTreeRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RemoveTreeResponse {
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
        if self.workspace_forgotten {
            len += 1;
        }
        if self.directory_removed {
            len += 1;
        }
        if self.validate_only {
            len += 1;
        }
        if !self.journal_entry.is_empty() {
            len += 1;
        }
        if self.directory_gone {
            len += 1;
        }
        let mut struct_ser =
            serializer.serialize_struct("mori.v1alpha1.RemoveTreeResponse", len)?;
        if let Some(v) = self.tree.as_ref() {
            struct_ser.serialize_field("tree", v)?;
        }
        if self.workspace_forgotten {
            struct_ser.serialize_field("workspaceForgotten", &self.workspace_forgotten)?;
        }
        if self.directory_removed {
            struct_ser.serialize_field("directoryRemoved", &self.directory_removed)?;
        }
        if self.validate_only {
            struct_ser.serialize_field("validateOnly", &self.validate_only)?;
        }
        if !self.journal_entry.is_empty() {
            struct_ser.serialize_field("journalEntry", &self.journal_entry)?;
        }
        if self.directory_gone {
            struct_ser.serialize_field("directoryGone", &self.directory_gone)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RemoveTreeResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tree",
            "workspace_forgotten",
            "workspaceForgotten",
            "directory_removed",
            "directoryRemoved",
            "validate_only",
            "validateOnly",
            "journal_entry",
            "journalEntry",
            "directory_gone",
            "directoryGone",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Tree,
            WorkspaceForgotten,
            DirectoryRemoved,
            ValidateOnly,
            JournalEntry,
            DirectoryGone,
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
                            "workspaceForgotten" | "workspace_forgotten" => {
                                Ok(GeneratedField::WorkspaceForgotten)
                            }
                            "directoryRemoved" | "directory_removed" => {
                                Ok(GeneratedField::DirectoryRemoved)
                            }
                            "validateOnly" | "validate_only" => Ok(GeneratedField::ValidateOnly),
                            "journalEntry" | "journal_entry" => Ok(GeneratedField::JournalEntry),
                            "directoryGone" | "directory_gone" => Ok(GeneratedField::DirectoryGone),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RemoveTreeResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.RemoveTreeResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<RemoveTreeResponse, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut tree__ = None;
                let mut workspace_forgotten__ = None;
                let mut directory_removed__ = None;
                let mut validate_only__ = None;
                let mut journal_entry__ = None;
                let mut directory_gone__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Tree => {
                            if tree__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tree"));
                            }
                            tree__ = map_.next_value()?;
                        }
                        GeneratedField::WorkspaceForgotten => {
                            if workspace_forgotten__.is_some() {
                                return Err(serde::de::Error::duplicate_field(
                                    "workspaceForgotten",
                                ));
                            }
                            workspace_forgotten__ = Some(map_.next_value()?);
                        }
                        GeneratedField::DirectoryRemoved => {
                            if directory_removed__.is_some() {
                                return Err(serde::de::Error::duplicate_field("directoryRemoved"));
                            }
                            directory_removed__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ValidateOnly => {
                            if validate_only__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validateOnly"));
                            }
                            validate_only__ = Some(map_.next_value()?);
                        }
                        GeneratedField::JournalEntry => {
                            if journal_entry__.is_some() {
                                return Err(serde::de::Error::duplicate_field("journalEntry"));
                            }
                            journal_entry__ = Some(map_.next_value()?);
                        }
                        GeneratedField::DirectoryGone => {
                            if directory_gone__.is_some() {
                                return Err(serde::de::Error::duplicate_field("directoryGone"));
                            }
                            directory_gone__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(RemoveTreeResponse {
                    tree: tree__,
                    workspace_forgotten: workspace_forgotten__.unwrap_or_default(),
                    directory_removed: directory_removed__.unwrap_or_default(),
                    validate_only: validate_only__.unwrap_or_default(),
                    journal_entry: journal_entry__.unwrap_or_default(),
                    directory_gone: directory_gone__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct(
            "mori.v1alpha1.RemoveTreeResponse",
            FIELDS,
            GeneratedVisitor,
        )
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
        if self.vcs != 0 {
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
        if self.vcs != 0 {
            let v = Vcs::try_from(self.vcs)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.vcs)))?;
            struct_ser.serialize_field("vcs", &v)?;
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
        const FIELDS: &[&str] = &["repo", "path", "trees", "vcs"];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Repo,
            Path,
            Trees,
            Vcs,
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
                            "vcs" => Ok(GeneratedField::Vcs),
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
                let mut vcs__ = None;
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
                        GeneratedField::Vcs => {
                            if vcs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vcs"));
                            }
                            vcs__ = Some(map_.next_value::<Vcs>()? as i32);
                        }
                    }
                }
                Ok(RepoTrees {
                    repo: repo__.unwrap_or_default(),
                    path: path__.unwrap_or_default(),
                    trees: trees__.unwrap_or_default(),
                    vcs: vcs__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mori.v1alpha1.RepoTrees", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RestoreRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.entry_id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mori.v1alpha1.RestoreRequest", len)?;
        if !self.entry_id.is_empty() {
            struct_ser.serialize_field("entryId", &self.entry_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RestoreRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["entry_id", "entryId"];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            EntryId,
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
                            "entryId" | "entry_id" => Ok(GeneratedField::EntryId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RestoreRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.RestoreRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<RestoreRequest, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut entry_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::EntryId => {
                            if entry_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("entryId"));
                            }
                            entry_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(RestoreRequest {
                    entry_id: entry_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mori.v1alpha1.RestoreRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RestoreResponse {
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
        if !self.commit_id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mori.v1alpha1.RestoreResponse", len)?;
        if let Some(v) = self.tree.as_ref() {
            struct_ser.serialize_field("tree", v)?;
        }
        if !self.commit_id.is_empty() {
            struct_ser.serialize_field("commitId", &self.commit_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RestoreResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["tree", "commit_id", "commitId"];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Tree,
            CommitId,
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
                            "commitId" | "commit_id" => Ok(GeneratedField::CommitId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RestoreResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.RestoreResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<RestoreResponse, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut tree__ = None;
                let mut commit_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Tree => {
                            if tree__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tree"));
                            }
                            tree__ = map_.next_value()?;
                        }
                        GeneratedField::CommitId => {
                            if commit_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitId"));
                            }
                            commit_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(RestoreResponse {
                    tree: tree__,
                    commit_id: commit_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mori.v1alpha1.RestoreResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SkillFile {
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
        if self.action != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mori.v1alpha1.SkillFile", len)?;
        if !self.path.is_empty() {
            struct_ser.serialize_field("path", &self.path)?;
        }
        if self.action != 0 {
            let v = skill_file::Action::try_from(self.action).map_err(|_| {
                serde::ser::Error::custom(format!("Invalid variant {}", self.action))
            })?;
            struct_ser.serialize_field("action", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SkillFile {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["path", "action"];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Path,
            Action,
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
                            "action" => Ok(GeneratedField::Action),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SkillFile;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.SkillFile")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SkillFile, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut path__ = None;
                let mut action__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Path => {
                            if path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("path"));
                            }
                            path__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Action => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("action"));
                            }
                            action__ = Some(map_.next_value::<skill_file::Action>()? as i32);
                        }
                    }
                }
                Ok(SkillFile {
                    path: path__.unwrap_or_default(),
                    action: action__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mori.v1alpha1.SkillFile", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for skill_file::Action {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "ACTION_UNSPECIFIED",
            Self::Installed => "ACTION_INSTALLED",
            Self::Unchanged => "ACTION_UNCHANGED",
            Self::Updated => "ACTION_UPDATED",
            Self::KeptEdited => "ACTION_KEPT_EDITED",
            Self::SkippedNotOurs => "ACTION_SKIPPED_NOT_OURS",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for skill_file::Action {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "ACTION_UNSPECIFIED",
            "ACTION_INSTALLED",
            "ACTION_UNCHANGED",
            "ACTION_UPDATED",
            "ACTION_KEPT_EDITED",
            "ACTION_SKIPPED_NOT_OURS",
        ];

        struct GeneratedVisitor;

        impl serde::de::Visitor<'_> for GeneratedVisitor {
            type Value = skill_file::Action;

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
                    "ACTION_UNSPECIFIED" => Ok(skill_file::Action::Unspecified),
                    "ACTION_INSTALLED" => Ok(skill_file::Action::Installed),
                    "ACTION_UNCHANGED" => Ok(skill_file::Action::Unchanged),
                    "ACTION_UPDATED" => Ok(skill_file::Action::Updated),
                    "ACTION_KEPT_EDITED" => Ok(skill_file::Action::KeptEdited),
                    "ACTION_SKIPPED_NOT_OURS" => Ok(skill_file::Action::SkippedNotOurs),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for SyncSkillsRequest {
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
        let mut struct_ser = serializer.serialize_struct("mori.v1alpha1.SyncSkillsRequest", len)?;
        if self.validate_only {
            struct_ser.serialize_field("validateOnly", &self.validate_only)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SyncSkillsRequest {
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
            type Value = SyncSkillsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.SyncSkillsRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SyncSkillsRequest, V::Error>
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
                Ok(SyncSkillsRequest {
                    validate_only: validate_only__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mori.v1alpha1.SyncSkillsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SyncSkillsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.files.is_empty() {
            len += 1;
        }
        if self.validate_only {
            len += 1;
        }
        let mut struct_ser =
            serializer.serialize_struct("mori.v1alpha1.SyncSkillsResponse", len)?;
        if !self.files.is_empty() {
            struct_ser.serialize_field("files", &self.files)?;
        }
        if self.validate_only {
            struct_ser.serialize_field("validateOnly", &self.validate_only)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SyncSkillsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["files", "validate_only", "validateOnly"];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Files,
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
                            "files" => Ok(GeneratedField::Files),
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
            type Value = SyncSkillsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mori.v1alpha1.SyncSkillsResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SyncSkillsResponse, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut files__ = None;
                let mut validate_only__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Files => {
                            if files__.is_some() {
                                return Err(serde::de::Error::duplicate_field("files"));
                            }
                            files__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ValidateOnly => {
                            if validate_only__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validateOnly"));
                            }
                            validate_only__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(SyncSkillsResponse {
                    files: files__.unwrap_or_default(),
                    validate_only: validate_only__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct(
            "mori.v1alpha1.SyncSkillsResponse",
            FIELDS,
            GeneratedVisitor,
        )
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
        const FIELDS: &[&str] = &["id", "repo", "name", "path", "owner", "task", "lifetime"];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            Repo,
            Name,
            Path,
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
        if !self.bookmarks.is_empty() {
            len += 1;
        }
        if self.size_bytes != 0 {
            len += 1;
        }
        if self.size_partial {
            len += 1;
        }
        if !self.size_measured_at.is_empty() {
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
        if !self.bookmarks.is_empty() {
            struct_ser.serialize_field("bookmarks", &self.bookmarks)?;
        }
        if self.size_bytes != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser
                .serialize_field("sizeBytes", ToString::to_string(&self.size_bytes).as_str())?;
        }
        if self.size_partial {
            struct_ser.serialize_field("sizePartial", &self.size_partial)?;
        }
        if !self.size_measured_at.is_empty() {
            struct_ser.serialize_field("sizeMeasuredAt", &self.size_measured_at)?;
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
        const FIELDS: &[&str] = &[
            "tree",
            "status",
            "state",
            "bookmarks",
            "size_bytes",
            "sizeBytes",
            "size_partial",
            "sizePartial",
            "size_measured_at",
            "sizeMeasuredAt",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Tree,
            Status,
            State,
            Bookmarks,
            SizeBytes,
            SizePartial,
            SizeMeasuredAt,
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
                            "bookmarks" => Ok(GeneratedField::Bookmarks),
                            "sizeBytes" | "size_bytes" => Ok(GeneratedField::SizeBytes),
                            "sizePartial" | "size_partial" => Ok(GeneratedField::SizePartial),
                            "sizeMeasuredAt" | "size_measured_at" => {
                                Ok(GeneratedField::SizeMeasuredAt)
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
                let mut bookmarks__ = None;
                let mut size_bytes__ = None;
                let mut size_partial__ = None;
                let mut size_measured_at__ = None;
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
                        GeneratedField::Bookmarks => {
                            if bookmarks__.is_some() {
                                return Err(serde::de::Error::duplicate_field("bookmarks"));
                            }
                            bookmarks__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SizeBytes => {
                            if size_bytes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sizeBytes"));
                            }
                            size_bytes__ = Some(
                                map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?
                                    .0,
                            );
                        }
                        GeneratedField::SizePartial => {
                            if size_partial__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sizePartial"));
                            }
                            size_partial__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SizeMeasuredAt => {
                            if size_measured_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sizeMeasuredAt"));
                            }
                            size_measured_at__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(TreeRow {
                    tree: tree__,
                    status: status__.unwrap_or_default(),
                    state: state__,
                    bookmarks: bookmarks__.unwrap_or_default(),
                    size_bytes: size_bytes__.unwrap_or_default(),
                    size_partial: size_partial__.unwrap_or_default(),
                    size_measured_at: size_measured_at__.unwrap_or_default(),
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
impl serde::Serialize for Vcs {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "VCS_UNSPECIFIED",
            Self::Jj => "VCS_JJ",
            Self::Git => "VCS_GIT",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for Vcs {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["VCS_UNSPECIFIED", "VCS_JJ", "VCS_GIT"];

        struct GeneratedVisitor;

        impl serde::de::Visitor<'_> for GeneratedVisitor {
            type Value = Vcs;

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
                    "VCS_UNSPECIFIED" => Ok(Vcs::Unspecified),
                    "VCS_JJ" => Ok(Vcs::Jj),
                    "VCS_GIT" => Ok(Vcs::Git),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
