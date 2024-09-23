use std::{collections::HashMap, hash::{Hash, Hasher}};

use server_nano::Server;

#[derive(Debug, Clone)]
struct Field {
	name: String,
	ty: String,
	builder: Vec<TemplatePieces>,
}

#[derive(Debug, Clone)]
struct Schema {
	name: String,
	fields: Vec<Field>,
}

#[derive(Debug, Clone)]
struct Route {
	name: String,
	response: String,
}

fn ingest_routes(value: &serde_json::Value, routes: &mut HashMap<String, Route>, parent: String) {
	if let Some(jroutes) = value.get("routes") {
		for (route_name, route) in jroutes.as_object().unwrap() {
			let response = route.get("response").unwrap().as_object().unwrap().get("schema").unwrap().as_str().unwrap().to_string();

			let r = format!("{}{}", parent, route_name);

			routes.insert(r.clone(), Route {
				name: r.clone(),
				response,
			});

			ingest_routes(route, routes, r);
		}
	}
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

fn get_fake_name(seed: usize) -> &'static str {
	FAKE_FIRST_NAMES[seed % FAKE_FIRST_NAMES.len()]
}

fn get_fake_last_name(seed: usize) -> &'static str {
	FAKE_LAST_NAMES[seed % FAKE_LAST_NAMES.len()]
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

fn build_object(schemas: &HashMap<String, Schema>, schema: &Schema, id: &str, seed: usize) -> serde_json::Value {
	let mut obj = serde_json::value::Map::new();

	let hashed_key: usize = {
		let mut hasher = std::collections::hash_map::DefaultHasher::new();
		id.hash(&mut hasher);
		(hasher.finish() as usize).wrapping_add(seed)
	};

	schema.fields.iter().for_each(|field| {
		let val = match field.ty.as_str() {
			"String" => {
				let mut res_string = String::new();
				for tp in &field.builder {
					match tp {
						TemplatePieces::String(s) => {
							res_string.push_str(&s);
						},
						TemplatePieces::Range(min, max) => {
							let val = min + (hashed_key as i64 % (max - min));
							res_string.push_str(&val.to_string());
						},
						TemplatePieces::Variable(s) => {
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
									res_string.push_str(id);
								},
								"this.id::UUID" | "this.id::UUIDv4" => {
									res_string.push_str(get_fake_uuidv4(hashed_key).as_str());
								},
								"ADJECTIVE" => {
									res_string.push_str(ADJECTIVES[hashed_key % ADJECTIVES.len()]);
								},
								_ => {
									res_string.push_str(s.as_str());
								}
							}
						}
						_ => {
							unreachable!();
						}
					}
				}
				serde_json::Value::String(res_string)
			},
			"Array" => {
				let mut arr = Vec::new();

				for tp in &field.builder {
					match tp {
						TemplatePieces::Schema(schema_name) => {
							let schema = schemas.get(schema_name).expect("Schema not found");
							arr.push(build_object(schemas, schema, id, hashed_key));
						},
						_ => {
							unreachable!();
						}
					}
				}

				serde_json::Value::Array(arr)
			},
			_ => {
				unreachable!();
			}
		};

		obj.insert(field.name.clone(), val);
	});

	serde_json::Value::Object(obj)
}

fn ingest_data(source: &serde_json::Value) -> (HashMap<String, Route>, HashMap<String, Schema>) {
	let mut routes = HashMap::<String, Route>::new();

	ingest_routes(&source, &mut routes, "".to_string());
	let schemas = ingest_schemas(&source);

	(routes, schemas)
}

fn ingest_schemas(source: &serde_json::Value) -> HashMap<String, Schema> {
	let mut schemas = HashMap::new();

	if let Some(serde_json::Value::Object(jschemas)) = source.get("schemas") {
    	for (name, schema) in jschemas {
     		let jfields = schema.get("fields").unwrap().as_object().unwrap();
			let mut fields = Vec::new();
			for (field_name, field) in jfields {
				if let Some(serde_json::Value::String(template)) = field.get("template") {
					fields.push(Field {
						name: field_name.to_string(),
						ty: "String".to_string(),
						builder: parse_template(template.as_str()),
					});
				}

				if let Some(serde_json::Value::Object(items)) = field.get("items") {
					if let Some(serde_json::Value::String(schema)) = items.get("schema") {
						fields.push(Field {
							name: field_name.to_string(),
							ty: "Array".to_string(),
							builder: vec![TemplatePieces::Schema(schema.to_string())],
						});
					}
				}
			}

			schemas.insert(name.to_string(), Schema {
				name: name.to_string(),
				fields,
			});
     	}
    }

    schemas
}

fn main() {
	let data_path = std::env::args().nth(1).expect("No data file provided");

	let data = std::fs::read_to_string(data_path).unwrap();

    let source = serde_json5::from_str::<serde_json::Value>(&data).unwrap();

    let (routes, schemas) = ingest_data(&source);

	let scale = 16;
	let seed: usize = 0;

	let mut app = Server::new();

	for (route_name, route) in routes {
		let datatype_name = route.response.split("[").collect::<Vec<&str>>()[0].to_string();
		let is_array = route.response.ends_with("[]");

		let schemas = schemas.clone();
		let datatype_name = datatype_name.clone();

		app.get(&route_name, move |req, res| {
			let key = req.parameter("id").unwrap_or("-1").to_string();

			let schema = schemas.get(&datatype_name).unwrap();

			let rsp = if is_array {
				let mut arr = Vec::new();
				for i in 0..scale {
					arr.push(build_object(&schemas, &schema, &format!("{}", i), seed));
				}
				serde_json::Value::Array(arr)
			} else {
				build_object(&schemas, &schema, &key, seed)
			};

			res.json(&rsp)
		});
	}

	app.listen("0.0.0.0:80").unwrap();
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum TemplatePieces {
	String(String),
	Variable(String),
	Range(i64, i64),
	Schema(String),
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

fn parse_template(template: &str) -> Vec<TemplatePieces> {
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
				result.push(TemplatePieces::Range(min, max));
			} else {
				result.push(TemplatePieces::Variable(variable.to_string()));
			}

			start = &rest[1..];
		} else {
			let (chunk, rest) = scan_until(start, "${");
			result.push(TemplatePieces::String(chunk.to_string()));
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
	fn parse_range_generator() {
		{
			let template = "${1..100}";
			let result = parse_template(template);
			assert_eq!(result, vec![TemplatePieces::Range(1, 100)]);
		}

		{
			let template = "My age is ${1..100}";
			let result = parse_template(template);
			assert_eq!(result, vec![TemplatePieces::String("My age is ".to_string()), TemplatePieces::Range(1, 100)]);
		}

		{
			let template = "${1..100} years old";
			let result = parse_template(template);
			assert_eq!(result, vec![TemplatePieces::Range(1, 100), TemplatePieces::String(" years old".to_string())]);
		}

		{
			let template = "${0..50}${50..100}";
			let result = parse_template(template);
			assert_eq!(result, vec![TemplatePieces::Range(0, 50), TemplatePieces::Range(50, 100)]);
		}

		{
			let template = "${0..50} ${50..100}";
			let result = parse_template(template);
			assert_eq!(result, vec![TemplatePieces::Range(0, 50), TemplatePieces::String(" ".to_string()), TemplatePieces::Range(50, 100)]);
		}
	}

	#[test]
	fn parse_variable_generator() {
		{
			let template = "${this.id::UUID}";
			let result = parse_template(template);
			assert_eq!(result, vec![TemplatePieces::Variable("this.id::UUID".to_string())]);
		}

		{
			let template = "My name is ${faker.person.fullName}";
			let result = parse_template(template);
			assert_eq!(result, vec![TemplatePieces::String("My name is ".to_string()), TemplatePieces::Variable("faker.person.fullName".to_string())]);
		}

		{
			let template = "${faker.person.fullName} years old";
			let result = parse_template(template);
			assert_eq!(result, vec![TemplatePieces::Variable("faker.person.fullName".to_string()), TemplatePieces::String(" years old".to_string())]);
		}

		{
			let template = "${faker.person.fullName} ${faker.person.fullName}";
			let result = parse_template(template);
			assert_eq!(result, vec![TemplatePieces::Variable("faker.person.fullName".to_string()), TemplatePieces::String(" ".to_string()), TemplatePieces::Variable("faker.person.fullName".to_string())]);
		}
	}

	#[test]
	fn parse_mixed_generators() {
		{
			let template = "My name is ${faker.person.fullName} and I am ${1..100} years old";
			let result = parse_template(template);
			assert_eq!(result, vec![
				TemplatePieces::String("My name is ".to_string()),
				TemplatePieces::Variable("faker.person.fullName".to_string()),
				TemplatePieces::String(" and I am ".to_string()),
				TemplatePieces::Range(1, 100),
				TemplatePieces::String(" years old".to_string()),
			]);
		}

		{
			let template = "${1..100} ${faker.person.fullName} ${1..100}";
			let result = parse_template(template);
			assert_eq!(result, vec![
				TemplatePieces::Range(1, 100),
				TemplatePieces::String(" ".to_string()),
				TemplatePieces::Variable("faker.person.fullName".to_string()),
				TemplatePieces::String(" ".to_string()),
				TemplatePieces::Range(1, 100),
			]);
		}
	}

	#[test]
	fn parse_literal() {
		{
			let template = "My name is John";
			let result = parse_template(template);
			assert_eq!(result, vec![TemplatePieces::String("My name is John".to_string())]);
		}
	}
}
