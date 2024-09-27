use std::collections::HashMap;

use routes::{ingest_routes, Route};
use schemas::{Schema, ingest_schemas};
use server_nano::Server;
use values::{build_value, Context, StringExpressions};

mod fake;
mod schemas;
mod routes;
mod values;

fn ingest_data(source: &serde_json::Value) -> (HashMap<String, Route>, HashMap<String, Schema>) {
	let routes = ingest_routes(&source);
	let schemas = ingest_schemas(&source);

	(routes, schemas)
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

pub fn parse_template(template: &str) -> Vec<StringExpressions> {
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
