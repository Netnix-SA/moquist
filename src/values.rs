use std::{collections::HashMap, hash::{Hash, Hasher}};

use crate::{fake, schemas::{Field, Schema,}};

pub struct Context {
	pub id: Option<String>,
	pub seed: usize,
	pub size: usize,
}

pub fn build_value(schemas: &HashMap<String, Schema>, datatype: &DataTypes, ctx: &Context) -> serde_json::Value {
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
								res_string.push_str(fake::get_fake_full_name(hashed_key).as_str());
							},
							"FIELD.name" => {
								let name = fake::get_fake_field_name(hashed_key);
								res_string.push_str(name);
							},
							"FIELD.value" => {
								let value = fake::get_fake_field_value(hashed_key);
								res_string.push_str(value);
							},
							"this.id" => {
								res_string.push_str(ctx.id.as_ref().map_or("", |f| f.as_str()));
							},
							"this.id::UUID" | "this.id::UUIDv4" => {
								res_string.push_str(&fake::get_fake_uuidv4(hashed_key));
							},
							"ADJECTIVE" => {
								res_string.push_str(fake::get_fake_adjective(hashed_key));
							},
							"ROLE" => {
								res_string.push_str(fake::get_fake_role_name(hashed_key));
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
		DataTypes::Enum(values) => {
			let val = hashed_key % values.len();
			serde_json::Value::String(values[val].clone())
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

pub fn build_object(schemas: &HashMap<String, Schema>, (fields): &(Vec<Field>), ctx: &Context) -> serde_json::value::Map<String, serde_json::Value> {
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Dates {
	Future,
	Soon,
	Now,
	Recent,
	Past,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StringExpressions {
	Literal(String),
	Variable(String),
	Range(i64, i64),
	Date(Dates),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NumberExpressions {
	Literal(i64),
	Range(i64, i64),
	Variable(String),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ObjectExpressions {
	Object(Vec<Field>),
	Schema(String),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DataTypes {
	String(Vec<StringExpressions>),
	Number(NumberExpressions),
	Object(ObjectExpressions),
	Array(ObjectExpressions),
	Enum(Vec<String>),
	Null,
}

// TODO: test
