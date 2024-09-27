# Moquist
> Create simple REST API mocks with ease.

Moquist simplifies the task of creating API mocks by letting you define reusable pieces, like schemas or routes.

## Sample
```json5
{
	schemas: {
		Person: {
			name: { template: "${FULL_NAME}" },
			age: { range: { min: 18, max: 99 } },
		},
	},
	routes: {
		'/people': {
			response: { schema: "Person[]" },
			routes: {
				'/:id': {
					response: { schema: "Person" },
				}
			}
		},
	},
}
```

As you can see, you can define a schema for a `Person` and then use it in the `/people` route and its sub-routes.

## Installation
You can pull the latest images from `ghcr.io/netnix-sa/moquist`.

## Usage
Moquist runs by default on port 80.
The Dockerfile entrypoint is the `moquist` command, so you can pass any arguments to it.

### Arguments
1. The path to the configuration file.
2. The scale factor for the array sizes. (optional)

Moquist uses [`json5`](https://json5.org/) for its configuration files, so you can use comments and other niceties.

## Schemas
Schemas are the building blocks of your API. They define the structure of your data.

To start defining them create the top-level `schemas` key in your configuration file.

```json5
{
	schemas: {
		// ...
	},
}
```

Each key inside the `schemas` object is a schema definition, and said schemas will be named after the key.

To define a schema, you will then define the `fields` key inside of your schema definition.

```json5
{
	schemas: {
		Person: {
			fields: {
				// ...
			},
		},
	},
}
```

Inside the `fields` object, you will then define the fields of your schema.

```json5
{
	schemas: {
		Person: {
			fields: {
				name: { template: "${FULL_NAME}" },
				age: { range: { min: 18, max: 99 } },
				// ...
			},
		},
	},
}
```

### Fields

Fields can have different types of definitions, like `template` or `range`.

Any non-object fields will be treated as literals of that same type.

```json5
name: "John Doe", // Literal, will always be "John Doe"
age: 25, // Literal, will always be 25
```

#### Template

The `template` field is a string that can contain variables that will be replaced by their values.
This field will always produce a string.


```json5
presentation: { template: "My name is ${FULL_NAME}" },
```

##### Templates

###### Variables

- `FULL_NAME`: A persons full name.
- `FIRST_NAME`: A persons first name.
- `LAST_NAME`: A persons last name.
- `PHONE_NUMBER`: A random phone number.
- `EMAIL`: A random email address.
- `ADDRESS`: A random address.
- `ADJECTIVE`: A random adjective.
- `NOUN`: A random noun.
- `VERB`: A random verb.

###### Ranges

- `a..b`: A random number between `a` and `b`.

###### Special values

- `this.id`: The current object's id. Usually taken from the route parameter or the array index. Of type string.

###### Casts

Some values can be cast to other formats by using the `::` operator.

- `this.id::UUIDv4`: The current object's id as a UUIDv4.

##### Range

The `range` field is an object that defines a range of (numeric) values.
This field will always produce a number.

```json5
age: { range: { min: 18, max: 99 } },
```

Or you can use an array.

```json5
age: { range: [18, 99] }, // Equivalent to the previous example
```

The min and max values are inclusive.

##### Enum

The `enum` field is an array of values that will be randomly selected.
This field will always produce a value from the array.

```json5
prefixes: { enum: ["Mr.", "Mrs.", "Ms.", "Dr."] },
```

Or

```json5
prefixes: { values: ["Mr.", "Mrs.", "Ms.", "Dr."] },
```

##### Nested objects

You can nest objects by defining object properties with a `fields` key.

```json5
Person: {
	fields: {
		telephones: {
			fields: {
				home: { template: "${PHONE_NUMBER}" },
				work: { template: "${PHONE_NUMBER}" },
			},
		},
	},
},
```

Or, you can reference other schemas by using the `schema` field.

```json5
Telephones: {
	fields: {
		home: { template: "${PHONE_NUMBER}" },
		work: { template: "${PHONE_NUMBER}" },
	},
},
Person: {
	fields: {
		telephones: { schema: "Telephones" },
	},
},
```

##### Arrays

You can define arrays by using the `items` field.

```json5
// ...
Person: {
	fields: {
		contacts: { items: { schema: "Contact", } },
	},
},
```

## Routes

Routes are the paths that your API will respond to.

To start defining them create the top-level `routes` key in your configuration file.

```json5
{
	routes: {
		// ...
	},
}
```

Each key inside the `routes` object is a route definition, and said routes will be named after the key.

To define a route, you will then define the `response` key inside of your route definition.

```json5
{
	routes: {
		'/people': {
			response: { schema: "Person[]" },
			// ...
		},
	},
}
```

The response expects an `schema` property that works the same way as the schema definitions. You can even define 'inline' types.

#### Sub-routes

You can define sub-routes by defining a `routes` key inside of your route definition.

```json5
{
	routes: {
		'/people': {
			response: { schema: "Person[]" },
			routes: {
				'/:id': {
					response: { schema: "Person" },
				}
			}
		},
	},
}
```
