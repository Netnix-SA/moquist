use std::collections::HashMap;

use crate::{schemas::ingest_schema, values::{DataTypes, ObjectExpressions},};

#[derive(Debug, Clone)]
pub struct Route {
	name: String,
	pub response: DataTypes,
}

pub fn ingest_routes(value: &serde_json::Value) -> HashMap<String, Route> {
	fn ingest_routes_internal(value: &serde_json::Value, routes: &mut HashMap<String, Route>, parent: String) {
		if let Some(jroutes) = value.get("routes") {
			for (route_name, route) in jroutes.as_object().unwrap() {
				let response = match route.get("response") {
					Some(serde_json::Value::String(response)) => {
						if response.contains("[]") {
							let response = response.replace("[]", "");
							DataTypes::Array(ObjectExpressions::Schema(response))
						} else {
							DataTypes::Object(ObjectExpressions::Schema(response.clone()))
						}
					},
					Some(serde_json::Value::Object(response)) => {
						match response.get("schema") {
							Some(serde_json::Value::String(response)) => {
								if response.contains("[]") {
									let response = response.replace("[]", "");
									DataTypes::Array(ObjectExpressions::Schema(response))
								} else {
									DataTypes::Object(ObjectExpressions::Schema(response.clone()))
								}
							}
							Some(serde_json::Value::Object(schema)) => {
								if let Some(serde_json::Value::Object(items)) = schema.get("items") {
									if let Some(serde_json::Value::Object(schema)) = items.get("schema") {
										DataTypes::Array(ObjectExpressions::Object(ingest_schema(schema)))
									} else {
										DataTypes::Null
									}
								} else {
									DataTypes::Object(ObjectExpressions::Object(ingest_schema(schema)))
								}
							}
							_ => DataTypes::Null,
						}
					},
					_ => DataTypes::Null,
				};

				let r = format!("{}{}", parent, route_name);

				routes.insert(r.clone(), Route {
					name: r.clone(),
					response,
				});

				ingest_routes_internal(route, routes, r);
			}
		}
	}

	let mut routes = HashMap::new();

	ingest_routes_internal(value, &mut routes, "".to_string());

	routes
}

#[cfg(test)]
mod tests {
	use crate::{schemas::Field, values::StringExpressions};

	use super::*;

	#[test]
	fn test_ingest_routes() {
		const STRING: &str = r#"
		{
			schemas: {
				Person: {
					id: "${this.id::UUID}",
					name: "${faker.person.fullName}",
					risk: "${1..100}",
				},
			},
			routes: {
				"/people": {
					response: { schema: "Person[]", },
					routes: {
						"/:id": {
							response: { schema: "Person", },
						},
						"/positions": {
							response: { schema: { items: { schema: { fields: { name: { template: "Teller" } } } } } },
						},
					},
				},
			}
		}
		"#;

		let values: serde_json::Value = serde_json5::from_str(STRING).unwrap();

		let routes = ingest_routes(&values);

		{
			let people = routes.get("/people").unwrap();
			assert_eq!(people.response, DataTypes::Array(ObjectExpressions::Schema("Person".to_string())));

			{
				let people_id = routes.get("/people/:id").unwrap();
				assert_eq!(people_id.response, DataTypes::Object(ObjectExpressions::Schema("Person".to_string())));
			}

			{
				let people_positions = routes.get("/people/positions").unwrap();
				assert_eq!(people_positions.response, DataTypes::Array(ObjectExpressions::Object(vec![Field {
					name: "name".to_string(),
					datatype: DataTypes::String(vec![StringExpressions::Literal("Teller".to_string())]),
				}])));
			}
		}
	}
}
