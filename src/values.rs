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
		DataTypes::Boolean(value) => {
			serde_json::Value::Bool(*value)
		},
		DataTypes::String(expression) => {
			let mut res_string = String::new();

			build_string(expression, &mut res_string, hashed_key, ctx);

			serde_json::Value::String(res_string)
		},
		DataTypes::Enum(values) => {
			let val = hashed_key % values.len();
			serde_json::Value::String(values[val].clone())
		},
		DataTypes::Array(expression) => {
			match expression {
				ArrayExpressions::Literal(elements) => {
					let arr = elements.iter().map(|expr| {
						build_value(schemas, expr, ctx)
					}).collect();

					serde_json::Value::Array(arr)
				},
				ArrayExpressions::Generated(expression) => {
					let arr = (0..ctx.size).map(|i| {
						build_value(schemas, expression, &Context{ id: None, seed: hashed_key.wrapping_add(i), size: ctx.size })
					}).collect();

					serde_json::Value::Array(arr)
				}
			}
		},
		DataTypes::Number(expression) => {
			build_number(expression, hashed_key)
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

fn build_number(expression: &NumberExpressions, hashed_key: usize) -> serde_json::Value {
    let val = match expression {
	    NumberExpressions::Literal(v) => {
		    match v {
			    NumberPrimitive::Integer(v) => serde_json::Number::from(*v),
			    NumberPrimitive::NonInteger(v) => serde_json::Number::from_f64(*v).unwrap(),
		    }
	    },
	    NumberExpressions::Range(min, max) => {
		    match min {
			    NumberPrimitive::Integer(min) => {
				    match max {
					    NumberPrimitive::Integer(max) => {
						    let val = min + (hashed_key as i64).rem_euclid(max - min);
						    serde_json::Number::from(val)
					    },
					    NumberPrimitive::NonInteger(max) => {
						    let val = *min as f64 + (hashed_key as f64).rem_euclid(*max - *min as f64);
						    serde_json::Number::from_f64(val).unwrap()
					    }
				    }
			    },
			    NumberPrimitive::NonInteger(min) => {
				    match max {
					    NumberPrimitive::Integer(max) => {
						    let val = min + (hashed_key as f64).rem_euclid(*max as f64 - min);
						    serde_json::Number::from_f64(val).unwrap()
					    },
					    NumberPrimitive::NonInteger(max) => {
						    let val = min + (hashed_key as f64).rem_euclid(max - min);
						    serde_json::Number::from_f64(val).unwrap()
					    }
				    }
			    }
		    }
	    },
	    NumberExpressions::Variable(s) => {
		    match s.as_str() {
			    "this.id" => {
				    serde_json::Number::from(hashed_key as i64)
			    },
			    _ => {
				    serde_json::Number::from(s.parse::<i64>().unwrap())
			    }
		    }
	    }
    };

    serde_json::Value::Number(val)
}

fn build_string(expression: &StringExpressions, res_string: &mut String, hashed_key: usize, ctx: &Context) {
    match expression {
	    StringExpressions::Literal(s) => {
		    res_string.push_str(&s);
	    },
	    StringExpressions::Range(min, max) => {
		    let val = min + (hashed_key as i64).rem_euclid(max - min);
		    res_string.push_str(&val.to_string());
	    },
		StringExpressions::Template(expressions) => {
			for expression in expressions {
				build_string(expression, res_string, hashed_key, ctx);
			}
		}
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
	Template(Vec<StringExpressions>),
	Range(i64, i64),
	Date(Dates),
}

#[derive(Debug, PartialEq, Clone)]
pub enum NumberPrimitive {
	NonInteger(f64),
	Integer(i64),
}

#[derive(Debug, PartialEq, Clone)]
pub enum NumberExpressions {
	Literal(NumberPrimitive),
	Range(NumberPrimitive, NumberPrimitive),
	Variable(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ObjectExpressions {
	Object(Vec<Field>),
	Schema(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ArrayExpressions {
	Literal(Vec<DataTypes>),
	Generated(Box<DataTypes>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum DataTypes {
	String(StringExpressions),
	Number(NumberExpressions),
	Object(ObjectExpressions),
	Array(ArrayExpressions),
	Enum(Vec<String>),
	Boolean(bool),
	Null,
}

// TODO: test

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_build_value() {
		let mut schemas = HashMap::new();

		schemas.insert("test".to_string(), Schema{ name: "test".to_string(), fields: vec![] });

		let ctx = Context{ id: None, seed: 0, size: 0 };

		let value = build_value(&schemas, &DataTypes::String(StringExpressions::Literal("test".to_string())), &ctx);
		assert_eq!(value, serde_json::Value::String("test".to_string()));

		let value = build_value(&schemas, &DataTypes::Number(NumberExpressions::Literal(NumberPrimitive::Integer(42))), &ctx);
		assert_eq!(value, serde_json::Value::Number(serde_json::Number::from(42)));

		let value = build_value(&schemas, &DataTypes::Number(NumberExpressions::Literal(NumberPrimitive::NonInteger(42.0))), &ctx);
		assert_eq!(value, serde_json::Value::Number(serde_json::Number::from_f64(42.0).unwrap()));

		let value = build_value(&schemas, &DataTypes::Object(ObjectExpressions::Schema("test".to_string())), &ctx);
		assert_eq!(value, serde_json::Value::Object(serde_json::value::Map::new()));

		let value = build_value(&schemas, &DataTypes::Array(ArrayExpressions::Literal(vec![DataTypes::String(StringExpressions::Literal("test".to_string()))])), &ctx);
		assert_eq!(value, serde_json::Value::Array(vec![serde_json::Value::String("test".to_string())]));

		let value = build_value(&schemas, &DataTypes::Enum(vec!["test".to_string()]), &ctx);
		assert_eq!(value, serde_json::Value::String("test".to_string()));

		let value = build_value(&schemas, &DataTypes::Boolean(true), &ctx);
		assert_eq!(value, serde_json::Value::Bool(true));

		let value = build_value(&schemas, &DataTypes::Null, &ctx);
		assert_eq!(value, serde_json::Value::Null);
	}
}
