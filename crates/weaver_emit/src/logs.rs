// SPDX-License-Identifier: Apache-2.0

//! Translations from Weaver to OTel for logs.

use crate::attributes::get_attribute_name_value;
use opentelemetry::logs::{AnyValue, Logger, LoggerProvider, LogRecord};
use opentelemetry::{Array, Value};
use weaver_forge::registry::ResolvedRegistry;
use weaver_semconv::group::GroupType;

/// Convert an OpenTelemetry Value to an AnyValue for logs
fn value_to_any_value(value: &Value) -> AnyValue {
    match value {
        Value::Bool(b) => AnyValue::Boolean(*b),
        Value::I64(i) => AnyValue::Int(*i),
        Value::F64(f) => AnyValue::Double(*f),
        Value::String(s) => AnyValue::String(s.clone().into()),
        Value::Array(arr) => match arr {
            Array::Bool(bools) => {
                AnyValue::ListAny(Box::new(
                    bools.iter().map(|b| AnyValue::Boolean(*b)).collect(),
                ))
            }
            Array::I64(ints) => {
                AnyValue::ListAny(Box::new(
                    ints.iter().map(|i| AnyValue::Int(*i)).collect(),
                ))
            }
            Array::F64(floats) => {
                AnyValue::ListAny(Box::new(
                    floats.iter().map(|f| AnyValue::Double(*f)).collect(),
                ))
            }
            Array::String(strings) => {
                AnyValue::ListAny(Box::new(
                    strings.iter().map(|s| AnyValue::String(s.clone().into())).collect(),
                ))
            }
            _ => AnyValue::String("unsupported_type".into()),
        },
        _ => AnyValue::String("unsupported_value_type".into()),
    }
}

/// Uses the provided logger_provider to emit logs for all the defined
/// events (logs) in the registry
pub(crate) fn emit_logs_for_registry<P: LoggerProvider>(
    registry: &ResolvedRegistry,
    logger_provider: &P,
) {
    let logger = logger_provider.logger("weaver");

    // Emit each log/event to the OTLP receiver.
    for group in registry.groups.iter() {
        if group.r#type == GroupType::Event {
            let mut log_record = logger.create_log_record();

            // Set the log body to the event name or ID
            log_record.set_body(
                group.name.clone()
                    .unwrap_or_else(|| group.id.clone())
                    .into()
            );

            // Add attributes from the group
            for attribute in &group.attributes {
                let kv = get_attribute_name_value(attribute);
                let any_value = value_to_any_value(&kv.value);
                log_record.add_attribute(kv.key.to_string(), any_value);
            }

            logger.emit(log_record);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use weaver_forge::registry::ResolvedGroup;
    use weaver_resolved_schema::attribute::Attribute;
    use weaver_semconv::attribute::{AttributeType, Examples, PrimitiveOrArrayTypeSpec, RequirementLevel};
    use weaver_semconv::stability::Stability;

    #[test]
    fn test_emit_logs_for_registry() {
        let registry = ResolvedRegistry {
            registry_url: "TEST".to_owned(),
            groups: vec![
                ResolvedGroup {
                    id: "event.example".to_owned(),
                    r#type: GroupType::Event,
                    brief: "Test event".to_owned(),
                    note: "".to_owned(),
                    prefix: "".to_owned(),
                    extends: None,
                    stability: Some(Stability::Development),
                    deprecated: None,
                    attributes: vec![Attribute {
                        name: "test.attribute".to_owned(),
                        r#type: AttributeType::PrimitiveOrArray(PrimitiveOrArrayTypeSpec::String),
                        examples: Some(Examples::String("value".to_owned())),
                        brief: "Test attribute".to_owned(),
                        tag: None,
                        requirement_level: RequirementLevel::Required,
                        sampling_relevant: None,
                        note: "".to_owned(),
                        stability: Some(Stability::Stable),
                        deprecated: None,
                        prefix: false,
                        tags: None,
                        value: None,
                        annotations: None,
                        role: Default::default(),
                    }],
                    span_kind: None,
                    events: vec![],
                    metric_name: None,
                    instrument: None,
                    unit: None,
                    name: Some("example.event".to_owned()),
                    lineage: None,
                    display_name: None,
                    body: None,
                    entity_associations: vec![],
                    annotations: None,
                },
            ],
        };

        // This test just verifies the function can be called without panicking
        emit_logs_for_registry(&registry);
    }
}
