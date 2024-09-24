use std::{collections::HashMap, hash::{Hash, Hasher}};

use server_nano::Server;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Field {
	name: String,
	datatype: DataTypes,
}

#[derive(Debug, Clone)]
struct Schema {
	name: String,
	fields: Vec<Field>,
}

#[derive(Debug, Clone)]
struct Route {
	name: String,
	response: DataTypes,
}

fn ingest_routes(value: &serde_json::Value) -> HashMap<String, Route> {
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

struct FakeField {
	name: &'static str,
	values: [&'static str; 12],
}

const FIELDS: [FakeField; 12] = [
	FakeField { name: "first_name", values: FAKE_FIRST_NAMES },
	FakeField { name: "last_name", values: FAKE_LAST_NAMES },
	FakeField { name: "email", values: ["a@email.com", "b@email.com", "c@email.com", "d@email.com", "e@email.com", "f@email.com", "g@email.com", "h@email.com", "i@email.com", "j@email.com", "k@email.com", "l@email.com",], },
	FakeField { name: "phone", values: ["1234567890", "2345678901", "3456789012", "4567890123", "5678901234", "6789012345", "7890123456", "8901234567", "9012345678", "0123456789", "1234567890", "2345678901"] },
	FakeField { name: "address", values: ["123 Fake St", "456 Fake St", "789 Fake St", "012 Fake St", "345 Fake St", "678 Fake St", "901 Fake St", "234 Fake St", "567 Fake St", "890 Fake St", "123 Fake St", "456 Fake St"] },
	FakeField { name: "city", values: ["New York", "Los Angeles", "Chicago", "Houston", "Phoenix", "Philadelphia", "San Antonio", "San Diego", "Dallas", "San Jose", "Austin", "Jacksonville"] },
	FakeField { name: "state", values: ["NY", "CA", "IL", "TX", "AZ", "PA", "TX", "CA", "TX", "CA", "TX", "FL"] },
	FakeField { name: "zip", values: ["12345", "23456", "34567", "45678", "56789", "67890", "78901", "89012", "90123", "01234", "12345", "23456"] },
	FakeField { name: "country", values: ["USA", "USA", "USA", "USA", "USA", "USA", "USA", "USA", "USA", "USA", "USA", "USA"] },
	FakeField { name: "company", values: ["Apple", "Google", "Microsoft", "Amazon", "Facebook", "Twitter", "Uber", "Lyft", "Airbnb", "Netflix", "Spotify", "Slack"] },
	FakeField { name: "job", values: ["Software Engineer", "Product Manager", "Designer", "Data Scientist", "Sales", "Marketing", "Customer Support", "HR", "Finance", "Operations", "Legal", "Security"] },
	FakeField { name: "age", values: ["20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "30", "31"] },
];

const FAKE_FIRST_NAMES: [&str; 12] = [
	"Facundo",
	"Lucca",
	"Maximo",
	"Juan",
	"Pedro",
	"Maria",
	"Jose",
	"Lucia",
	"Carlos",
	"Julieta",
	"Martin",
	"Agustina",
];

const FAKE_LAST_NAMES: [&str; 12] = [
	"Villa",
	"Salerno",
	"Alvarez",
	"Martinez",
	"Perez",
	"Sanchez",
	"Romero",
	"Suarez",
	"Vazquez",
	"Rojas",
	"Acosta",
	"Blanco",
];

const ADJECTIVES: [&str; 12] = [
	"crazy",
	"rambuctious",
	"happy",
	"sazzy",
	"creepy",
	"sexy",
	"cool",
	"tubular",
	"radical",
	"intriguing",
	"boring",
	"lame",
];

const ROLES: [&str; 12] = [
	"superadmin",
	"admin",
	"editor",
	"author",
	"contributor",
	"subscriber",
	"customer",
	"guest",
	"visitor",
	"banned",
	"pending",
	"deleted",
];

fn get_fake_name(seed: usize) -> &'static str {
	FAKE_FIRST_NAMES[seed % FAKE_FIRST_NAMES.len()]
}

fn get_fake_last_name(seed: usize) -> &'static str {
	FAKE_LAST_NAMES[seed % FAKE_LAST_NAMES.len()]
}

fn get_fake_role(seed: usize) -> &'static str {
	ROLES[seed % ROLES.len()]
}

fn get_fake_full_name(seed: usize) -> String {
	if seed % 2 == 0 {
		format!("{} {}", get_fake_name(seed), get_fake_last_name(seed))
	} else if seed % 3 == 0 {
		format!("{} {} {} {}", get_fake_name(seed), get_fake_last_name(seed + 1), get_fake_last_name(seed), get_fake_last_name(seed + 2))
	} else {
		format!("{} {} {}", get_fake_name(seed), get_fake_last_name(seed + 1), get_fake_last_name(seed))
	}
}

fn get_fake_uuidv4(seed: usize) -> String {
	const HEX_CHARS: [char; 16] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'];

	fn hex_char(i: usize) -> char {
		HEX_CHARS[i % 16]
	}

	format!("{}{}{}{}{}{}{}{}-{}{}{}{}-4{}{}{}-{}{}{}{}{}{}{}{}{}{}{}{}", hex_char(seed + 0), hex_char(seed + 18), hex_char(seed + 3), hex_char(seed + 99), hex_char(seed + 2), hex_char(seed + 18), hex_char(seed + 6), hex_char(seed + 7), hex_char(seed + 19), hex_char(seed + 9), hex_char(seed + 36), hex_char(seed + 23), hex_char(seed + 12), hex_char(seed + 11), hex_char(seed + 14), hex_char(seed + 15), hex_char(seed + 13), hex_char(seed + 17), hex_char(seed + 12), hex_char(seed + 9), hex_char(seed + 20), hex_char(seed + 21), hex_char(seed + 22), hex_char(seed + 5), hex_char(seed + 24), hex_char(seed + 25), hex_char(seed + 16))
}

struct Context {
	id: Option<String>,
	seed: usize,
	size: usize,
}

fn build_value(schemas: &HashMap<String, Schema>, datatype: &DataTypes, ctx: &Context) -> serde_json::Value {
	let hashed_key = if let Some(id) = ctx.id.as_ref() {
		let mut hasher = std::collections::hash_map::DefaultHasher::new();
		id.hash(&mut hasher);
		(hasher.finish() as usize).wrapping_add(ctx.seed)
	} else {
		ctx.seed
	};

	let val = match datatype {
		DataTypes::String(expressions) => {
			let mut res_string = String::new();

			for expression in expressions {
				match expression {
					StringExpressions::Literal(s) => {
						res_string.push_str(&s);
					},
					StringExpressions::Range(min, max) => {
						let val = min + (hashed_key as i64).rem_euclid(max - min);
						res_string.push_str(&val.to_string());
					},
					StringExpressions::Variable(s) => {
						match s.as_str() {
							"FULL_NAME" => {
								res_string.push_str(get_fake_full_name(hashed_key).as_str());
							},
							"FIELD.name" => {
								let name = FIELDS[hashed_key % FIELDS.len()].name;
								res_string.push_str(name);
							},
							"FIELD.value" => {
								let value = FIELDS[hashed_key % FIELDS.len()].values[hashed_key % FIELDS[hashed_key % FIELDS.len()].values.len()];
								res_string.push_str(value);
							},
							"this.id" => {
								res_string.push_str(ctx.id.as_ref().map_or("", |f| f.as_str()));
							},
							"this.id::UUID" | "this.id::UUIDv4" => {
								res_string.push_str(get_fake_uuidv4(hashed_key).as_str());
							},
							"ADJECTIVE" => {
								res_string.push_str(ADJECTIVES[hashed_key % ADJECTIVES.len()]);
							},
							"ROLE" => {
								res_string.push_str(get_fake_role(hashed_key));
							},
							_ => {
								res_string.push_str(s.as_str());
							}
						}
					},
					StringExpressions::Date(date) => {
						use chrono::prelude::*;

						let now = Utc::now();

						let date = match date {
							Dates::Future => {
								now + chrono::Duration::days(36 + (hashed_key as i64).rem_euclid(120 - 36))
							},
							Dates::Soon => {
								now + chrono::Duration::days(1 + (hashed_key as i64).rem_euclid(36 - 1))
							},
							Dates::Now => { now },
							Dates::Recent => {
								now - chrono::Duration::days(1 + (hashed_key as i64).rem_euclid(36 - 1))
							},
							Dates::Past => {
								now - chrono::Duration::days(36 + (hashed_key as i64).rem_euclid(120 - 36))
							},
						};

						res_string.push_str(&date.to_rfc3339());
					}
				}
			}

			serde_json::Value::String(res_string)
		},
		DataTypes::Array(expression) => {
			let mut arr = Vec::new();

			for i in 0..16 {
				let id = format!("{}", i);

				match expression {
					ObjectExpressions::Schema(schema_name) => {
						let schema = schemas.get(schema_name).expect("Schema not found");
						arr.push(serde_json::Value::Object(build_object(schemas, (&schema.fields), &Context{ id: Some(id), seed: hashed_key, size: ctx.size })));
					},
					ObjectExpressions::Object(fields) => {
						arr.push(serde_json::Value::Object(build_object(schemas, &fields, &Context{ id: Some(id), seed: hashed_key, size: ctx.size })));
					},
				}
			}

			serde_json::Value::Array(arr)
		},
		DataTypes::Number(expression) => {
			let mut val = 0;

			match expression {
				NumberExpressions::Literal(v) => {
					val = *v;
				},
				NumberExpressions::Range(min, max) => {
					val = min + (hashed_key as i64).rem_euclid(max - min);
				},
				NumberExpressions::Variable(s) => {
					match s.as_str() {
						"this.id" => {
							val = hashed_key as i64;
						},
						_ => {
							val = s.parse::<i64>().unwrap();
						}
					}
				}
			}

			serde_json::Value::Number(serde_json::Number::from(val))
		}
		DataTypes::Object(expression) => {
			let obj = match expression {
				ObjectExpressions::Object(fields) => {
					build_object(schemas, &fields, ctx)
				},
				ObjectExpressions::Schema(schema_name) => {
					let schema = schemas.get(schema_name).expect("Schema not found");
					build_object(schemas, &schema.fields, ctx)
				},
			};

			serde_json::Value::Object(obj)
		},
		DataTypes::Null => serde_json::Value::Null,
	};

	val
}

fn build_object(schemas: &HashMap<String, Schema>, (fields): &(Vec<Field>), ctx: &Context) -> serde_json::value::Map<String, serde_json::Value> {
	let hashed_key = if let Some(id) = ctx.id.as_ref() {
		let mut hasher = std::collections::hash_map::DefaultHasher::new();
		id.hash(&mut hasher);
		(hasher.finish() as usize).wrapping_add(ctx.seed)
	} else {
		ctx.seed
	};

	let mut obj = serde_json::value::Map::new();

	for field in fields {
		obj.insert(field.name.clone(), build_value(schemas, &field.datatype, &Context{ id: ctx.id.to_owned(), seed: hashed_key, size: ctx.size }));
	}

	obj
}

fn ingest_data(source: &serde_json::Value) -> (HashMap<String, Route>, HashMap<String, Schema>) {
	let routes = ingest_routes(&source);
	let schemas = ingest_schemas(&source);

	(routes, schemas)
}

fn ingest_schema(source: &serde_json::Map<String, serde_json::Value>) -> Vec<Field> {
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
		}
	}

	fields
}

fn ingest_schemas(source: &serde_json::Value) -> HashMap<String, Schema> {
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

fn main() {
	let data_path = std::env::args().nth(1).expect("No data file path was provided");

	let data = std::fs::read_to_string(data_path).expect("Failed to read data file");

    let source = serde_json5::from_str::<serde_json::Value>(&data).unwrap();

    let (routes, schemas) = ingest_data(&source);

	let scale = std::env::args().nth(2).map(|scale| scale.parse::<usize>().unwrap()).unwrap_or(16);
	let seed = std::env::args().nth(3).map(|seed| seed.parse::<usize>().unwrap()).unwrap_or(0);

	let mut app = Server::new();

	// Put routes that contain colons at the end
	let mut routes = routes.into_iter().collect::<Vec<_>>();
	routes.sort_by(|(a, _), (b, _)| {
        let a_has_colon = a.split('/').last().unwrap_or("").contains(':');
        let b_has_colon = b.split('/').last().unwrap_or("").contains(':');

        if a_has_colon == b_has_colon {
            std::cmp::Ordering::Equal
        } else if a_has_colon {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Less
        }
    });

	for (route_name, route) in routes {
		let schemas = schemas.clone();

		app.get(&route_name, move |req, res| {
			let id = req.parameter("id").map(|id| id.to_string());

			let rsp = build_value(&schemas, &route.response, &Context{ id, seed, size: scale });

			res.json(&rsp)
		});
	}

	app.listen("0.0.0.0:80").unwrap();
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum Dates {
	Future,
	Soon,
	Now,
	Recent,
	Past,
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum StringExpressions {
	Literal(String),
	Variable(String),
	Range(i64, i64),
	Date(Dates),
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum NumberExpressions {
	Literal(i64),
	Range(i64, i64),
	Variable(String),
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum ObjectExpressions {
	Object(Vec<Field>),
	Schema(String),
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum DataTypes {
	String(Vec<StringExpressions>),
	Number(NumberExpressions),
	Object(ObjectExpressions),
	Array(ObjectExpressions),
	Null,
}

fn scan_until<'a>(s: &'a str, m: &'a str) -> (&'a str, &'a str) {
	let mut i = 0;
	while i < s.len() {
		if s[i..].starts_with(m) {
			return (&s[..i], &s[i..]);
		}
		i += 1;
	}
	(&s, "")
}

fn parse_template(template: &str) -> Vec<StringExpressions> {
	let mut result = Vec::new();
	let mut start = template;

	while !start.is_empty() {
		if start.starts_with("${") {
			let (content, rest) = scan_until(&start[2..], "}");
			let variable = content;
			if variable.contains("..") {
				let range = variable.split("..").collect::<Vec<&str>>();
				let min = range[0].parse::<i64>().unwrap();
				let max = range[1].parse::<i64>().unwrap();
				result.push(StringExpressions::Range(min, max));
			} else {
				result.push(StringExpressions::Variable(variable.to_string()));
			}

			start = &rest[1..];
		} else {
			let (chunk, rest) = scan_until(start, "${");
			result.push(StringExpressions::Literal(chunk.to_string()));
			start = rest;
		}
	}

	result
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test() {
		const STRING: &str = r#"
		{
			schemas: {
				Person: {
					id: "${this.id::UUID}",
					name: "${faker.person.fullName}",
					risk: "${1..100}",
				},
				Campaign: {
					id: "${this.id::UUID}",
					name: "My ${ADJECTIVE} campaign",
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
			routes: {
				"/people": {
					response: { schema: "Person[]", },
					routes: {
						"/:id": {
							response: { schema: "Person", },
						},
					},
				},
				"/campaigns": {
					response: { schema: "Campaign[]", },
					routes: {
						"/:id": {
							response: { schema: "Campaign", },
							routes: {
								"/certificants": {
									response: { schema: "Person[]", },
								},
							}
						},
					},
				},
			}
		}
		"#;
	}

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

	#[test]
	fn parse_range_generator() {
		{
			let template = "${1..100}";
			let result = parse_template(template);
			assert_eq!(result, vec![StringExpressions::Range(1, 100)]);
		}

		{
			let template = "My age is ${1..100}";
			let result = parse_template(template);
			assert_eq!(result, vec![StringExpressions::Literal("My age is ".to_string()), StringExpressions::Range(1, 100)]);
		}

		{
			let template = "${1..100} years old";
			let result = parse_template(template);
			assert_eq!(result, vec![StringExpressions::Range(1, 100), StringExpressions::Literal(" years old".to_string())]);
		}

		{
			let template = "${0..50}${50..100}";
			let result = parse_template(template);
			assert_eq!(result, vec![StringExpressions::Range(0, 50), StringExpressions::Range(50, 100)]);
		}

		{
			let template = "${0..50} ${50..100}";
			let result = parse_template(template);
			assert_eq!(result, vec![StringExpressions::Range(0, 50), StringExpressions::Literal(" ".to_string()), StringExpressions::Range(50, 100)]);
		}
	}

	#[test]
	fn parse_variable_generator() {
		{
			let template = "${this.id::UUID}";
			let result = parse_template(template);
			assert_eq!(result, vec![StringExpressions::Variable("this.id::UUID".to_string())]);
		}

		{
			let template = "My name is ${faker.person.fullName}";
			let result = parse_template(template);
			assert_eq!(result, vec![StringExpressions::Literal("My name is ".to_string()), StringExpressions::Variable("faker.person.fullName".to_string())]);
		}

		{
			let template = "${faker.person.fullName} years old";
			let result = parse_template(template);
			assert_eq!(result, vec![StringExpressions::Variable("faker.person.fullName".to_string()), StringExpressions::Literal(" years old".to_string())]);
		}

		{
			let template = "${faker.person.fullName} ${faker.person.fullName}";
			let result = parse_template(template);
			assert_eq!(result, vec![StringExpressions::Variable("faker.person.fullName".to_string()), StringExpressions::Literal(" ".to_string()), StringExpressions::Variable("faker.person.fullName".to_string())]);
		}
	}

	#[test]
	fn parse_mixed_generators() {
		{
			let template = "My name is ${faker.person.fullName} and I am ${1..100} years old";
			let result = parse_template(template);
			assert_eq!(result, vec![
				StringExpressions::Literal("My name is ".to_string()),
				StringExpressions::Variable("faker.person.fullName".to_string()),
				StringExpressions::Literal(" and I am ".to_string()),
				StringExpressions::Range(1, 100),
				StringExpressions::Literal(" years old".to_string()),
			]);
		}

		{
			let template = "${1..100} ${faker.person.fullName} ${1..100}";
			let result = parse_template(template);
			assert_eq!(result, vec![
				StringExpressions::Range(1, 100),
				StringExpressions::Literal(" ".to_string()),
				StringExpressions::Variable("faker.person.fullName".to_string()),
				StringExpressions::Literal(" ".to_string()),
				StringExpressions::Range(1, 100),
			]);
		}
	}

	#[test]
	fn parse_literal() {
		{
			let template = "My name is John";
			let result = parse_template(template);
			assert_eq!(result, vec![StringExpressions::Literal("My name is John".to_string())]);
		}
	}
}
