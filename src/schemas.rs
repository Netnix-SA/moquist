use std::collections::HashMap;

use crate::{parse_template, values::{DataTypes, Dates, NumberExpressions, ObjectExpressions, StringExpressions}};

#[derive(Debug, Clone)]
pub struct Schema {
	pub name: String,
	pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
	pub name: String,
	pub datatype: DataTypes,
}

pub fn ingest_schemas(source: &serde_json::Value) -> HashMap<String, Schema> {
	let mut schemas = HashMap::new();

	if let Some(serde_json::Value::Object(jschemas)) = source.get("schemas") {
    	for (name, schema) in jschemas {
     		if let serde_json::Value::Object(schema) = schema {
     			let fields = ingest_schema(schema);

				schemas.insert(name.to_string(), Schema {
					name: name.to_string(),
					fields,
				});
       		}
     	}
    }

    schemas
}

pub fn ingest_schema(source: &serde_json::Map<String, serde_json::Value>) -> Vec<Field> {
	let jfields = source.get("fields").unwrap().as_object().unwrap();

	let mut fields = Vec::new();

	for (field_name, field) in jfields {
		if let serde_json::Value::Object(field) = field {
			if let Some(serde_json::Value::String(template)) = field.get("template") {
				fields.push(Field {
					name: field_name.to_string(),
					datatype: DataTypes::String(parse_template(template.as_str())),
				});
			}

			if let Some(serde_json::Value::Object(range)) = field.get("range") {
				if let (Some(serde_json::Value::Number(min)), Some(serde_json::Value::Number(max))) = (range.get("min"), range.get("max")) {
					fields.push(Field {
						name: field_name.to_string(),
						datatype: DataTypes::Number(NumberExpressions::Range(min.as_i64().unwrap(), max.as_i64().unwrap())),
					});
				}
			}

			if let Some(serde_json::Value::Object(date)) = field.get("date") {
				let frame = if let Some(serde_json::Value::String(frame)) = date.get("frame") {
					frame.as_str()
				} else {
					"now"
				};

				let frame = match frame {
					"now" => Dates::Now,
					"future" => Dates::Future,
					"soon" => Dates::Soon,
					"recent" => Dates::Recent,
					"past" => Dates::Past,
					_ => Dates::Now,
				};

				fields.push(Field {
					name: field_name.to_string(),
					datatype: DataTypes::String(vec![StringExpressions::Date(frame)]),
				});
			};

			if let Some(serde_json::Value::Object(items)) = field.get("items") {
				if let Some(serde_json::Value::String(schema)) = items.get("schema") {
					fields.push(Field {
						name: field_name.to_string(),
						datatype: DataTypes::Array(ObjectExpressions::Schema(schema.to_string())),
					});
				}
			}

			if let Some(serde_json::Value::Object(_)) = field.get("fields") {
				let sub_fields = ingest_schema(field);
				fields.push(Field {
					name: field_name.to_string(),
					datatype: DataTypes::Object(ObjectExpressions::Object(sub_fields)),
				});
			}

			if let Some(serde_json::Value::Array(values)) = field.get("enum") {
				let mut enum_values = Vec::new();

				for value in values {
					if let serde_json::Value::String(value) = value {
						enum_values.push(value.to_string());
					}
				}

				fields.push(Field {
					name: field_name.to_string(),
					datatype: DataTypes::Enum(enum_values),
				});
			}
		}
	}

	fields
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_ingest_schemas() {
		{
			let source = r#"
			{
				schemas: {
					Field: {
						fields: {
							name: { template: "${FIELD.name}" },
							value: { template: "${FIELD.value}" },
						},
					},
					Person: {
						fields: {
							id: { template: "${this.id::UUID}", },
							name: { template: "${FULL_NAME}" },
							risk: { range: { min: 1, max: 100, }, },
							fields: { items: { schema: "Field" } },
							title: { enum: ["Mr", "Mrs", "Ms", "Dr"] },
						},
					},
					Campaign: {
						fields: {
							id: { template: "${this.id::UUID}", },
							name: { template: "My ${ADJECTIVE} campaign", },
							start: { date: { frame: "recent" } },
							end: { date: { frame: "future" } },
							stats: {
								fields: {
									batch: { range: { min: 1, max: 10 } },
									commited: { range: { min: 1, max: 10 } },
									total: { range: { min: 1, max: 10 } },
								},
							},
						},
					},
				}
			}"#;

			let source = serde_json5::from_str::<serde_json::Value>(source).unwrap();

			let schemas = ingest_schemas(&source);

			assert_eq!(schemas.len(), 3);

			{
				let field = schemas.get("Field").unwrap();

				let name_field = field.fields.iter().find(|f| f.name == "name").unwrap();
				assert_eq!(name_field.datatype, DataTypes::String(vec![StringExpressions::Variable("FIELD.name".to_string())]));

				let value_field = field.fields.iter().find(|f| f.name == "value").unwrap();
				assert_eq!(value_field.datatype, DataTypes::String(vec![StringExpressions::Variable("FIELD.value".to_string())]));
			}

			{
				let person = schemas.get("Person").unwrap();

				let id_field = person.fields.iter().find(|f| f.name == "id").unwrap();
				assert_eq!(id_field.datatype, DataTypes::String(vec![StringExpressions::Variable("this.id::UUID".to_string())]));

				let name_field = person.fields.iter().find(|f| f.name == "name").unwrap();
				assert_eq!(name_field.datatype, DataTypes::String(vec![StringExpressions::Variable("FULL_NAME".to_string())]));

				let risk_field = person.fields.iter().find(|f| f.name == "risk").unwrap();
				assert_eq!(risk_field.datatype, DataTypes::Number(NumberExpressions::Range(1, 100)));

				let fields_field = person.fields.iter().find(|f| f.name == "fields").unwrap();
				assert_eq!(fields_field.datatype, DataTypes::Array(ObjectExpressions::Schema("Field".to_string())));

				let title_field = person.fields.iter().find(|f| f.name == "title").unwrap();
				assert_eq!(title_field.datatype, DataTypes::Enum(vec!["Mr".to_string(), "Mrs".to_string(), "Ms".to_string(), "Dr".to_string()]));
			}

			{
				let campaign = schemas.get("Campaign").unwrap();

				let id_field = campaign.fields.iter().find(|f| f.name == "id").unwrap();
				assert_eq!(id_field.datatype, DataTypes::String(vec![StringExpressions::Variable("this.id::UUID".to_string())]));

				let name_field = campaign.fields.iter().find(|f| f.name == "name").unwrap();
				assert_eq!(name_field.datatype, DataTypes::String(vec![StringExpressions::Literal("My ".to_string()), StringExpressions::Variable("ADJECTIVE".to_string()), StringExpressions::Literal(" campaign".to_string())]));

				let start_field = campaign.fields.iter().find(|f| f.name == "start").unwrap();
				assert_eq!(start_field.datatype, DataTypes::String(vec![StringExpressions::Date(Dates::Recent)]));

				let end_field = campaign.fields.iter().find(|f| f.name == "end").unwrap();
				assert_eq!(end_field.datatype, DataTypes::String(vec![StringExpressions::Date(Dates::Future)]));

				let stats_field = campaign.fields.iter().find(|f| f.name == "stats").unwrap();
				assert_eq!(stats_field.datatype, DataTypes::Object(ObjectExpressions::Object(vec![
					Field{ name: "batch".to_string(), datatype: DataTypes::Number(NumberExpressions::Range(1, 10)) },
					Field{ name: "commited".to_string(), datatype: DataTypes::Number(NumberExpressions::Range(1, 10)) },
						Field{ name: "total".to_string(), datatype: DataTypes::Number(NumberExpressions::Range(1, 10)) },
				])));
			}
		}
	}
}
