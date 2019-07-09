#[allow(dead_code)]
#[allow(unused_imports)]

#[macro_use]
extern crate json;


use std::fs;
use std::io::Write;
use std::collections::HashMap;


struct Site {
	id : String,
	name : String,
	phone : String,
	email : String,
	href : String,
	lat : String,
	lon : String,
	address: Vec<String>,
}

fn parse_address(obj: &json::JsonValue) -> Vec<String> {
	let mut address : Vec<String> = vec![];
	for i in 0..obj.len() {
		address.push(obj[i].to_string());
	}
	return address;
}

fn value_or_empty(obj: &json::JsonValue) -> String {
	if obj.is_null() {
		String::from("")
	} else {
		obj.to_string()
	}
}

fn get_phone(obj: &json::JsonValue) -> String {
	if !obj["div"][1].is_array() {
		value_or_empty(&obj["div"][1]["div"]["div"]["div"]["a"]["#text"])
	} else {
		value_or_empty(&obj["div"][1][0]["div"]["div"]["a"]["#text"])
	}
}

fn get_email(obj: &json::JsonValue) -> String {
	if obj["div"][1].is_array() {
		value_or_empty(&obj["div"][1][1]["div"]["div"]["a"]["#text"])
	} else {
		String::from("")
	}
}

fn handle(site_list: &mut Vec<Site>, positions : &HashMap<String, &json::JsonValue>, obj : &json::JsonValue) {
	let id = value_or_empty(&obj["@data-entity-id"]);
	let pos = positions[&id];
	let site = Site {
		id: 	id,
		name: 	value_or_empty(&obj["div"][0]["a"]["span"]),
		phone: 	get_phone(&obj),
		email: 	get_email(&obj),
		lat: 	value_or_empty(&pos["lat"]),
		lon: 	value_or_empty(&pos["lon"]),
		href: 	value_or_empty(&obj["div"][0]["a"]["@href"]),
		address: parse_address(&obj["span"]["div"]["div"]["p"]["span"]),
	};
	// println!("{}: {}", site_list.len(), site.name);
	site_list.push(site);
}

fn parse_list(obj : &json::JsonValue, positions : &HashMap<String, &json::JsonValue>) -> Vec<Site> {
	let mut site_list: Vec<Site> = vec![];
	for x in 0..obj.len()
	{
		if obj[x].is_array() {
			for y in 0..obj[x].len() {
				handle(&mut site_list, &positions, &obj[x][y]);
			}
		}
		else {
			handle(&mut site_list, &positions, &obj[x]["div"]);
		}
	}
	return site_list;
}

fn format(site_list: &Vec<Site>) -> String {
	let mut data = String::new();
	let sep = String::from(" ; ");

	data.push_str("ID");
	data.push_str(&sep);

	data.push_str("latitude");
	data.push_str(&sep);
	data.push_str("longitude");
	data.push_str(&sep);

	data.push_str("name");
	data.push_str(&sep);

	data.push_str("phone");
	data.push_str(&sep);

	data.push_str("email");
	data.push_str(&sep);

	for _i in 0..5 {
		data.push_str("address");
		data.push_str(&sep);
	}

	data.push_str("href");
	data.push_str(&sep);

	data.push('\n');

	for it in site_list {
		data.push_str(&it.id);
		data.push_str(&sep);

		data.push_str(&it.lat);
		data.push_str(&sep);
		data.push_str(&it.lon);
		data.push_str(&sep);

		data.push_str(&it.name);
		data.push_str(&sep);

		data.push_str(&it.phone);
		data.push_str(&sep);

		data.push_str(&it.email);
		data.push_str(&sep);

		for i in 0..5 {
			if it.address.len() > i {
				data.push_str(&it.address[i]);
			}
			data.push_str(&sep);
		}

		data.push_str(&it.href);
		data.push_str(&sep);

		data.push('\n');
	}
	return data;
}

fn map_id_positions(positions: &json::JsonValue) -> HashMap<String, &json::JsonValue> {
	let mut result : HashMap<String, &json::JsonValue> = HashMap::new();
	let list = &positions["leaflet"]["leaflet-map"]["features"];
	for i in 0..list.len() {
		result.insert(list[i]["id"].to_string(), &list[i]);
	}
	return result;
}

fn main() {
	let obj = json::parse(&fs::read_to_string("data\\data.json").unwrap()).unwrap();
	let positions = json::parse(&fs::read_to_string("data\\positions.json").unwrap()).unwrap();
	let hpos = map_id_positions(&positions);
	// println!("{:#}", obj[482]["div"]);
	
	let site_list = parse_list(&obj, &hpos);
	println!("count: {}", site_list.len());

	let mut f = fs::File::create("data\\data.csv").expect("Unable to create file");
	f.write_all(format(&site_list).as_bytes()).expect("Unable to write data");
}
