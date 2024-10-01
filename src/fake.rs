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

pub fn get_fake_full_name(seed: usize) -> String {
	if seed % 2 == 0 {
		format!("{} {}", get_fake_name(seed), get_fake_last_name(seed))
	} else if seed % 3 == 0 {
		format!("{} {} {} {}", get_fake_name(seed), get_fake_last_name(seed + 1), get_fake_last_name(seed), get_fake_last_name(seed + 2))
	} else {
		format!("{} {} {}", get_fake_name(seed), get_fake_last_name(seed + 1), get_fake_last_name(seed))
	}
}

fn get_fake_field(seed: usize) -> &'static FakeField {
	&FIELDS[seed % FIELDS.len()]
}

pub fn get_fake_field_name(seed: usize) -> &'static str {
	get_fake_field(seed).name
}

pub fn get_fake_field_value(seed: usize) -> &'static str {
	get_fake_field(seed).values[seed % get_fake_field(seed).values.len()]
}

pub fn get_fake_adjective(seed: usize) -> &'static str {
	ADJECTIVES[seed % ADJECTIVES.len()]
}

pub fn get_fake_role_name(seed: usize) -> &'static str {
	get_fake_role(seed)
}

pub fn get_fake_uuidv4(seed: usize) -> String {
	const HEX_CHARS: [char; 16] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'];

	fn hex_char(i: usize) -> char {
		HEX_CHARS[i % 16]
	}

	format!("{}{}{}{}{}{}{}{}-{}{}{}{}-4{}{}{}-{}{}{}{}{}{}{}{}{}{}{}{}", hex_char(seed + 0), hex_char(seed + 18), hex_char(seed + 3), hex_char(seed + 99), hex_char(seed + 2), hex_char(seed + 18), hex_char(seed + 6), hex_char(seed + 7), hex_char(seed + 19), hex_char(seed + 9), hex_char(seed + 36), hex_char(seed + 23), hex_char(seed + 12), hex_char(seed + 11), hex_char(seed + 14), hex_char(seed + 15), hex_char(seed + 13), hex_char(seed + 17), hex_char(seed + 12), hex_char(seed + 9), hex_char(seed + 20), hex_char(seed + 21), hex_char(seed + 22), hex_char(seed + 5), hex_char(seed + 24), hex_char(seed + 25), hex_char(seed + 16))
}
