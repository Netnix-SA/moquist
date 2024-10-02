use std::collections::HashMap;

use crate::{parse_template, values::{ArrayExpressions, DataTypes, Dates, NumberExpressions, NumberPrimitive, ObjectExpressions, StringExpressions}};

#[derive(Debug, Clone)]
pub struct Schema {
	pub name: String,
	pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq)]
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
		// Read Moquist schema, anything else, non object is treated as a literal
		ingest_field(field, &mut fields, field_name);
	}

	fields
}

fn ingest_field(field: &serde_json::Value, fields: &mut Vec<Field>, field_name: &String) {
	fields.push(Field {
		name: field_name.to_string(),
		datatype: ingest_value(field),
	});
}

/// Proceses a Moquist JSON description of a field
/// This function is recursive since it can process arrays
fn ingest_value(value: &serde_json::Value) -> DataTypes {
    match value {
	    serde_json::Value::Object(field) => {
		    // Read string-like value
		    if let Some(serde_json::Value::String(template)) = field.get("template") {
			    return DataTypes::String(StringExpressions::Template(parse_template(template.as_str())));
		    }

		    // Read range-like value
		    if let Some(range) = field.get("range") {
			    match range {
				    serde_json::Value::Object(range) => { // Object form { min, max }
					    if let (Some(serde_json::Value::Number(min)), Some(serde_json::Value::Number(max))) = (range.get("min"), range.get("max")) {
						    return DataTypes::Number(NumberExpressions::Range(NumberPrimitive::Integer(min.as_i64().unwrap()), NumberPrimitive::Integer(max.as_i64().unwrap())));
					    }
				    },
				    serde_json::Value::Array(min_max) => { // Array form [min, max]
					    if let (Some(serde_json::Value::Number(min)), Some(serde_json::Value::Number(max))) = (min_max.get(0), min_max.get(1)) {
						    return DataTypes::Number(NumberExpressions::Range(NumberPrimitive::Integer(min.as_i64().unwrap()), NumberPrimitive::Integer(max.as_i64().unwrap())));
					    }
				    },
				    _ => {},
			    }
		    }

		    // Read date-like value
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

			    return DataTypes::String(StringExpressions::Date(frame));
		    };

		    // Read array-like value
		    if let Some(serde_json::Value::Object(items)) = field.get("items") {
			    if let Some(serde_json::Value::String(schema)) = items.get("schema") {
				    return DataTypes::Array(ArrayExpressions::Generated(Box::new(DataTypes::Object(ObjectExpressions::Schema(schema.to_string())))));
			    }
		    }

		    // Read object-like value
		    if let Some(serde_json::Value::Object(_)) = field.get("fields") {
			    let sub_fields = ingest_schema(field);
			    return DataTypes::Object(ObjectExpressions::Object(sub_fields));
		    }

		    // Read enum-like value
		    if let Some(serde_json::Value::Array(elements)) = field.get("enum").or(field.get("values")) {
			    let mut enum_values = Vec::new();

			    for value in elements {
				    if let serde_json::Value::String(value) = value {
					    enum_values.push(value.to_string());
				    }
			    }

			    return DataTypes::Enum(enum_values);
		    } else {
			    return DataTypes::Null;
			}
	    }
	    a => {
		    match a {
			    serde_json::Value::String(template) => {
				    return DataTypes::String(StringExpressions::Literal(template.to_owned()));
			    },
			    serde_json::Value::Number(value) => {
				    if let Some(value) = value.as_f64() {
					    return DataTypes::Number(NumberExpressions::Literal(NumberPrimitive::NonInteger(value)));
				    } else if let Some(value) = value.as_i64() {
					    return DataTypes::Number(NumberExpressions::Literal(NumberPrimitive::Integer(value)));
				    } else {
					    return DataTypes::Null;
				    }
			    },
			    serde_json::Value::Bool(value) => {
				    return DataTypes::Boolean(*value);
			    },
			    serde_json::Value::Null => {
				    return DataTypes::Null;
			    },
				serde_json::Value::Array(elements) => {
				    return DataTypes::Array(ArrayExpressions::Literal(elements.iter().map(|element| ingest_value(element)).collect()));
			    },
			    _ => {
				    return DataTypes::Null;
			    },
		    };
	    },
    }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_ingest_null_literal() {
		let source = r#"
		{
			schemas: {
				Schema: {
					fields: {
						field: null,
					},
				},
			},
		}"#;

		let source = serde_json5::from_str::<serde_json::Value>(source).unwrap();
		let schemas = ingest_schemas(&source);

		assert_eq!(schemas.len(), 1);

		let schema = schemas.get("Schema").unwrap();

		assert_eq!(schema.fields.len(), 1);

		let field = &schema.fields[0];

		assert_eq!(field.name, "field");
		assert_eq!(field.datatype, DataTypes::Null);
	}

	#[test]
	fn test_ingest_bool_literal() {
		let source = r#"
		{
			schemas: {
				Schema: {
					fields: {
						field: true,
					},
				},
			},
		}"#;

		let source = serde_json5::from_str::<serde_json::Value>(source).unwrap();
		let schemas = ingest_schemas(&source);

		assert_eq!(schemas.len(), 1);

		let schema = schemas.get("Schema").unwrap();

		assert_eq!(schema.fields.len(), 1);

		let field = &schema.fields[0];

		assert_eq!(field.name, "field");
		assert_eq!(field.datatype, DataTypes::Boolean(true));
	}

	#[test]
	fn test_ingest_string_literal() {
		let source = r#"
		{
			schemas: {
				Schema: {
					fields: {
						field: "literal",
					},
				},
			},
		}"#;

		let source = serde_json5::from_str::<serde_json::Value>(source).unwrap();
		let schemas = ingest_schemas(&source);

		assert_eq!(schemas.len(), 1);

		let schema = schemas.get("Schema").unwrap();

		assert_eq!(schema.fields.len(), 1);

		let field = &schema.fields[0];

		assert_eq!(field.name, "field");
		assert_eq!(field.datatype, DataTypes::String(StringExpressions::Literal("literal".to_string())));
	}

	#[test]
	fn test_ingest_number_literal() {
		let source = r#"
		{
			schemas: {
				Schema: {
					fields: {
						field: 3.141,
					},
				},
			},
		}"#;

		let source = serde_json5::from_str::<serde_json::Value>(source).unwrap();
		let schemas = ingest_schemas(&source);

		assert_eq!(schemas.len(), 1);

		let schema = schemas.get("Schema").unwrap();

		assert_eq!(schema.fields.len(), 1);

		let field = &schema.fields[0];

		assert_eq!(field.name, "field");
		assert_eq!(field.datatype, DataTypes::Number(NumberExpressions::Literal(NumberPrimitive::NonInteger(3.141))));
	}

	#[test]
	fn test_ingest_array_literal() {
		let source = r#"
		{
			schemas: {
				Schema: {
					fields: {
						field: [null, true, "literal", 3.141],
					},
				},
			},
		}"#;

		let source = serde_json5::from_str::<serde_json::Value>(source).unwrap();
		let schemas = ingest_schemas(&source);

		assert_eq!(schemas.len(), 1);

		let schema = schemas.get("Schema").unwrap();

		assert_eq!(schema.fields.len(), 1);

		{
			let field = &schema.fields[0];

			assert_eq!(field.name, "field");
			assert_eq!(field.datatype, DataTypes::Array(ArrayExpressions::Literal(vec![DataTypes::Null, DataTypes::Boolean(true), DataTypes::String(StringExpressions::Literal("literal".to_string())), DataTypes::Number(NumberExpressions::Literal(NumberPrimitive::NonInteger(3.141)))])));
		}
	}

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
									commited: { range: [1, 10] },
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
				assert_eq!(name_field.datatype, DataTypes::String(StringExpressions::Template(vec![StringExpressions::Variable("FIELD.name".to_string())])));

				let value_field = field.fields.iter().find(|f| f.name == "value").unwrap();
				assert_eq!(value_field.datatype, DataTypes::String(StringExpressions::Template(vec![StringExpressions::Variable("FIELD.value".to_string())])));
			}

			{
				let person = schemas.get("Person").unwrap();

				let id_field = person.fields.iter().find(|f| f.name == "id").unwrap();
				assert_eq!(id_field.datatype, DataTypes::String(StringExpressions::Template(vec![StringExpressions::Variable("this.id::UUID".to_string())])));

				let name_field = person.fields.iter().find(|f| f.name == "name").unwrap();
				assert_eq!(name_field.datatype, DataTypes::String(StringExpressions::Template(vec![StringExpressions::Variable("FULL_NAME".to_string())])));

				let risk_field = person.fields.iter().find(|f| f.name == "risk").unwrap();
				assert_eq!(risk_field.datatype, DataTypes::Number(NumberExpressions::Range(NumberPrimitive::Integer(1), NumberPrimitive::Integer(100))));

				let fields_field = person.fields.iter().find(|f| f.name == "fields").unwrap();
				assert_eq!(fields_field.datatype, DataTypes::Array(ArrayExpressions::Generated(Box::new(DataTypes::Object(ObjectExpressions::Schema("Field".to_string()))))));

				let title_field = person.fields.iter().find(|f| f.name == "title").unwrap();
				assert_eq!(title_field.datatype, DataTypes::Enum(vec!["Mr".to_string(), "Mrs".to_string(), "Ms".to_string(), "Dr".to_string()]));
			}

			{
				let campaign = schemas.get("Campaign").unwrap();

				let id_field = campaign.fields.iter().find(|f| f.name == "id").unwrap();
				assert_eq!(id_field.datatype, DataTypes::String(StringExpressions::Template(vec![StringExpressions::Variable("this.id::UUID".to_string())])));

				let name_field = campaign.fields.iter().find(|f| f.name == "name").unwrap();
				assert_eq!(name_field.datatype, DataTypes::String(StringExpressions::Template(vec![StringExpressions::Literal("My ".to_string()), StringExpressions::Variable("ADJECTIVE".to_string()), StringExpressions::Literal(" campaign".to_string())])));

				let start_field = campaign.fields.iter().find(|f| f.name == "start").unwrap();
				assert_eq!(start_field.datatype, DataTypes::String(StringExpressions::Date(Dates::Recent)));

				let end_field = campaign.fields.iter().find(|f| f.name == "end").unwrap();
				assert_eq!(end_field.datatype, DataTypes::String(StringExpressions::Date(Dates::Future)));

				let stats_field = campaign.fields.iter().find(|f| f.name == "stats").unwrap();
				assert_eq!(stats_field.datatype, DataTypes::Object(ObjectExpressions::Object(vec![
					Field{ name: "batch".to_string(), datatype: DataTypes::Number(NumberExpressions::Range(NumberPrimitive::Integer(1), NumberPrimitive::Integer(10))) },
					Field{ name: "commited".to_string(), datatype: DataTypes::Number(NumberExpressions::Range(NumberPrimitive::Integer(1), NumberPrimitive::Integer(10))) },
				])));
			}
		}
	}
}
