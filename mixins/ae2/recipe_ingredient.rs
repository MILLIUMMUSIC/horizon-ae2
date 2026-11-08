// SPDX-License-Identifier: LGPL-3.0-or-later
// Original item/tag ingredient objects -> the native ingredient transport.
// This is a binding, not the complete Minecraft Ingredient/holder codec.
use serde_json::Value;

pub fn normalize(value: &Value) -> Option<Value> {
    match value {
        Value::String(_) => Some(value.clone()),
        Value::Object(o) if o.len() == 1 => o
            .get("item")
            .and_then(Value::as_str)
            .map(|s| Value::String(s.into()))
            .or_else(|| {
                o.get("tag")
                    .and_then(Value::as_str)
                    .map(|s| Value::String(format!("#{s}")))
            }),
        Value::Array(a) => {
            let all = a.iter().map(normalize).collect::<Option<Vec<_>>>()?;
            // Native Ingredient::parse silently filters nonstring alternatives.
            // Refuse them here rather than changing the source set of matches.
            all.iter()
                .all(Value::is_string)
                .then_some(Value::Array(all))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn original_item_and_tag_objects_keep_their_native_meaning() {
        assert_eq!(
            normalize(&json!({"item":"ae2:charged_certus_quartz_crystal"})),
            Some(json!("ae2:charged_certus_quartz_crystal"))
        );
        assert_eq!(
            normalize(&json!({"tag":"c:dusts/redstone"})),
            Some(json!("#c:dusts/redstone"))
        );
    }
    #[test]
    fn alternatives_preserve_order_and_unknown_shapes_refuse_the_whole_set() {
        assert_eq!(
            normalize(&json!([{"item":"minecraft:redstone"},{"tag":"c:gems/quartz"}])),
            Some(json!(["minecraft:redstone", "#c:gems/quartz"]))
        );
        for value in [
            json!([{"item":"minecraft:redstone"},{"unknown":"x"}]),
            json!([[{"item":"minecraft:redstone"}]]),
            json!({"item":"a","tag":"b"}),
        ] {
            assert_eq!(normalize(&value), None);
        }
    }
}
