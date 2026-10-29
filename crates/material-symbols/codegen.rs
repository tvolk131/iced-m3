//! Shared by the companion build script and the maintainer's xtask.
use std::collections::BTreeSet;

pub fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
}

pub fn from_json(bytes: &[u8]) -> Result<String, String> {
    let metadata: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
    let values = metadata
        .get("names")
        .and_then(serde_json::Value::as_array)
        .ok_or("Catalog must contain a names array")?;
    let names = values
        .iter()
        .map(|name| {
            name.as_str()
                .map(String::from)
                .ok_or("Catalog names must be strings")
        })
        .collect::<Result<Vec<_>, _>>()?;
    registry(&names)
}

pub fn registry(names: &[String]) -> Result<String, String> {
    if names.is_empty() || names.len() > usize::from(u16::MAX) + 1 {
        return Err("Catalog must contain 1..=65536 names for the u16 icon index".into());
    }
    if !names.windows(2).all(|pair| pair[0] < pair[1]) {
        return Err("Catalog names must be sorted and unique".into());
    }
    let mut source = String::from(
        "// Generated from assets/catalog.json by build.rs; do not edit.\ncatalog! {\n",
    );
    let mut variants = BTreeSet::new();
    for name in names {
        if !valid_name(name) {
            return Err(format!("Invalid upstream name {name:?}"));
        }
        let mut variant = String::new();
        for part in name.split('_').filter(|part| !part.is_empty()) {
            variant.push_str(&part[..1].to_ascii_uppercase());
            variant.push_str(&part[1..]);
        }
        if variant.is_empty() || variant == "Self" {
            return Err(format!("Invalid Rust variant for {name:?}"));
        }
        if variant.as_bytes()[0].is_ascii_digit() {
            variant.insert_str(0, "Icon");
        }
        if !variants.insert(variant.clone()) {
            return Err(format!("Rust name collision: {variant}"));
        }
        source.push_str(&format!("    {variant} => \"{name}\",\n"));
    }
    source.push_str("}\n");
    Ok(source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_inventory_generates_deterministic_typed_names() {
        let data = br#"{"names":["10k","arrow_back","self_improvement"]}"#;
        let source = from_json(data).unwrap();
        assert!(source.contains("Icon10k => \"10k\""));
        assert!(source.contains("ArrowBack => \"arrow_back\""));
        assert!(source.contains("SelfImprovement => \"self_improvement\""));
        assert_eq!(source, from_json(data).unwrap());
    }

    #[test]
    fn malformed_inventory_or_invalid_identifiers_fail_before_compilation() {
        for data in [
            "not json",
            "{}",
            r#"{"names":"home"}"#,
            r#"{"names":[4]}"#,
            r#"{"names":[]}"#,
            r#"{"names":["inbox","home"]}"#,
            r#"{"names":["home","home"]}"#,
            r#"{"names":["_"]}"#,
            r#"{"names":["self"]}"#,
            r#"{"names":["Home"]}"#,
        ] {
            assert!(from_json(data.as_bytes()).is_err(), "{data}");
        }
        let error = from_json(br#"{"names":["a__b","a_b"]}"#).unwrap_err();
        assert!(error.contains("Rust name collision"), "{error}");
    }
}
