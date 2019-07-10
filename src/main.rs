#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_assignments)]
#![allow(unused_variables)]

#[macro_use]
extern crate json;


use std::collections::HashMap;
use std::cmp::Ordering;
use std::{
	fs,
    fs::File,
    io::{BufWriter, Write},
};


struct ScribbleSite {
	id : String,
	title : String,
	desc : String,
	t : String,
	lat : String,
	lon : String,
}

fn main() {
	let address = String::from("https://www.scribblemaps.com/api/maps/NATUP/smjson");
	println!("Downloading from {}", address);
	let resp = reqwest::get(&address).unwrap().text().unwrap();

	println!("Parsing result");
	let data = json::parse(&resp).unwrap();

    let path = String::from("scribblemaps.json");
	println!("Writing json to {}", path);
    write!(&mut BufWriter::new(&File::create(&path).unwrap()), "{:#}", data).unwrap();

	println!("Extracting infos from json");
    let mut site_list : Vec<ScribbleSite> = vec![];
    for it in data["overlays"].members() {
    	let mut desc = value_or_empty(&it["description"]);
    	if desc.is_empty() { continue; }
    	desc = desc.replace("<div>", "");
    	desc = desc.replace("</div>", "");
    	let site = ScribbleSite {
	    	id: it["id"].to_string().trim().to_string(),
    		desc: desc.trim().to_string(),
	    	title: value_or_empty(&it["title"]).trim().to_string(),
	    	t: it["type"].to_string().trim().to_string(),
	    	lat: it["points"][0][0].to_string().trim().to_string(),
	    	lon: it["points"][0][1].to_string().trim().to_string(),
	    };
	    site_list.push(site);
    }

    site_list.sort_by(|a, b| {
    	match a.title.cmp(&b.title) {
	    	Ordering::Equal => a.desc.cmp(&b.desc),
	    	other => other,
	    }
    });

    {
		println!("Generating output");
		let mut output = String::new();
		let sep = String::from(" ; ");

		output.push_str("id");
		output.push_str(&sep);

		output.push_str("type");
		output.push_str(&sep);

		output.push_str("latitude");
		output.push_str(&sep);
		output.push_str("longitude");
		output.push_str(&sep);

		output.push_str("description");
		output.push_str(&sep);

		output.push_str("title");
		output.push_str(&sep);

		output.push_str("\n");

	    for it in site_list {
			output.push_str(&it.id);
			output.push_str(&sep);

			output.push_str(&it.t);
			output.push_str(&sep);

			output.push_str(&it.lat);
			output.push_str(&sep);
			output.push_str(&it.lon);
			output.push_str(&sep);

			output.push_str(&it.desc);
			output.push_str(&sep);

			output.push_str(&it.title);
			output.push_str(&sep);

			output.push_str("\n");
	    }

	    let opath = String::from("scribblemaps.csv");
		println!("Writing output to {}", opath);
		let mut f = fs::File::create(&opath).expect("Unable to create file");
		f.write_all(output.as_bytes()).expect("Unable to write data");
	}
}


fn value_or_empty(obj: &json::JsonValue) -> String {
	if obj.is_null() {
		String::new()
	} else {
		obj.to_string()
	}
}


/*

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
		String::new()
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

fn soufflet() {
	let obj = json::parse(&fs::read_to_string("data\\data.json").unwrap()).unwrap();
	let positions = json::parse(&fs::read_to_string("data\\positions.json").unwrap()).unwrap();
	let hpos = map_id_positions(&positions);
	// println!("{:#}", obj[482]["div"]);
	
	let site_list = parse_list(&obj, &hpos);
	println!("count: {}", site_list.len());

	let mut f = fs::File::create("data\\data.csv").expect("Unable to create file");
	f.write_all(format(&site_list).as_bytes()).expect("Unable to write data");
}
*/
