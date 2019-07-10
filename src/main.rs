#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_assignments)]
#![allow(unused_variables)]
#![allow(unused_macros)]

#[macro_use]
extern crate json;
extern crate byteorder;

use std::{
	fs,
    fs::File,
    io,
    io::{BufWriter, Write},
    cmp::Ordering,
    collections::HashMap,
};

use reqwest::StatusCode;
use byteorder::{WriteBytesExt, LittleEndian};


macro_rules! trynone {
    ($e:expr) => (match $e {
        Ok(val) => val,
        Err(err) => {println!("FAILED!"); return None; },
    });
}
macro_rules! tryretv {
    ($e:expr, $v:expr) => (match $e {
        Ok(val) => val,
        Err(err) => {println!("FAILED!"); return $v; },
    });
}

fn main() {
	scribblemaps();
	println!("");
	axereal();
}


fn download(url: &str) -> Option<String> {
	println!("Downloading from {}", url);
	let mut req = trynone!(reqwest::get(url));
	if req.status() != StatusCode::OK { println!("FAILED: {}", req.status()); return None; };
	let resp = trynone!(req.text());
	if resp.is_empty() { println!("FAILED: Data is empty"); return None; }
	return Some(resp);
}

fn download_json(url: &str, name: &str) -> Option<json::JsonValue> {
	let resp = download(url)?;
	println!("Parsing result");
	let data = trynone!(json::parse(&resp));

	if name.len() > 0 {
		let mut path = String::from(name);
		path.push_str(".json");
		println!("Writing json to {}", path);
	    trynone!(write!(&mut BufWriter::new(&trynone!(File::create(&path))), "{:#}", data));
	}

	return Some(data);
}



fn json_parse_names(data: &json::JsonValue, keys: &[&str], out: &mut Vec<String>) {
	for key in keys.iter() {
		out.push(json_get_value(&data[*key]));
	}
}

fn json_parse_array(data: &json::JsonValue, len: i32, out: &mut Vec<String>) {
	for key in 0..len {
		out.push(json_get_value(&data[key as usize]));
	}
}


fn json_get_value(obj: &json::JsonValue) -> String {
	if obj.is_null() {
		String::new()
	} else {
		obj.to_string().replace("<div>", "")
						.replace("</div>", "")
						.replace("&#039;", "\'")
						.replace("&quot;", "\"")
						.replace("&#x27;", "\'")
						.replace("&lt;", "<")
						.replace("&gt;", ">")
						.replace("–", "-")
						.replace("&amp;", "&")
						.trim().to_string()
	}
}

fn format(data: Vec<Vec<String>>, keys: Vec<&str>) -> String {
	let sep = ";";

	println!("Generating output");
	let mut result = String::new();

	result.push_str("sep=");
	result.push_str(sep);
	result.push_str("\n");

    for it in keys.iter() {
		result.push_str(it);
		result.push_str(sep);
    }

	result.push_str("\n");

    for val in data {
    	for it in val {
			result.push_str(&it);
			result.push_str(sep);
		}

		result.push_str("\n");
    }

    return result;
}

fn output(data: &str, name: &str) -> Option<()> {
	let mut path = String::from(name);
	path.push_str(".csv");
	println!("Writing output to {}", path);
	let mut f = trynone!(fs::File::create(path));

	let mut data_utf16: Vec<u16> = data.encode_utf16().collect();
    data_utf16.push(0);

    let mut result: Vec<u8> = Vec::new();
    for it in data_utf16 {
    	let _ = result.write_u16::<LittleEndian>(it);
    }

	trynone!(f.write(&result[..]));

	return Some(());
}

fn axereal() -> Option<()> {
	let name = "axereal";
	println!("{}", name);

	let data = download_json("https://www.axereal.com/geojsoncarte.json", &name)?;

    if data["features"].is_null() { println!("No data found in the json file"); return None; }
    if data["features"].len() <= 0 { println!("No elements found in the features"); return None; }

	let custom_keys = [ "lat", "lon", "type" ];
	let prop_keys = [ "name", "description", "Type_site", "Societe", "Adresse 1", "Adresse 2", "CP", "Ville", "Pays", "Tel", "Tid", "Tid_parent", "Type_parent" ];
	let mut keys : Vec<&str> = vec![];
	keys.extend_from_slice(&custom_keys);
	keys.extend_from_slice(&prop_keys);

	let mut valist: Vec<Vec<String>> = vec![];

	for it in data["features"].members() {
		let mut values: Vec<String> = vec![];
		json_parse_array(&it["geometry"]["coordinates"], 2, &mut values);
		json_parse_names(&it["properties"], &prop_keys, &mut values);
		valist.push(values);
	}

	output(&format(valist, keys), &name)?;

    println!("Done.");
    Some(())
}


fn get_type(obj : &json::JsonValue, legend : &HashMap<String, String>) -> String {
	let val = json_get_value(&obj);
	if !val.is_empty() {
		legend.get(&val).unwrap_or(&val).to_string()
	} else {
		String::new()
	}
}

fn scribblemaps() -> Option<()> {
	let name = "scribblemaps";
	println!("{}", name);

	let data = download_json("https://www.scribblemaps.com/api/maps/NATUP/smjson", &name)?;

	println!("Extracting infos from json");

    let mut legend : HashMap<String, String> = HashMap::new();
    for it in data["legend"].members() {
    	let id = json_get_value(&it["styleId"]);
    	if id.is_empty() { continue; }
    	legend.insert(id, json_get_value(&it["name"]));
    }

	let custom_keys = [ "lat", "lon", "type" ];
	let prop_keys = [ "description", "title", "id" ];
	let mut keys : Vec<&str> = vec![];
	keys.extend_from_slice(&custom_keys);
	keys.extend_from_slice(&prop_keys);

	let mut valist: Vec<Vec<String>> = vec![];

    for it in data["overlays"].members() {
		let mut values: Vec<String> = vec![];
    	if it["description"].is_null() { continue; }

    	json_parse_array(&it["points"][0], 2, &mut values);
	    values.push(get_type(&it["styleId"], &legend));
		json_parse_names(&it, &prop_keys, &mut values);
		valist.push(values);
    }

    let description_index = keys.iter().position(|&r| r == "description").unwrap();
    let title_index = keys.iter().position(|&r| r == "title").unwrap();

    valist.sort_by(|a, b| {
    	match a[title_index].cmp(&b[title_index]) { // title
	    	Ordering::Equal => a[description_index].cmp(&b[description_index]), // description
	    	other => other,
	    }
    });

	output(&format(valist, keys), &name)?;

	println!("Done.");
	Some(())
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
		json_get_value(&obj["div"][1]["div"]["div"]["div"]["a"]["#text"])
	} else {
		json_get_value(&obj["div"][1][0]["div"]["div"]["a"]["#text"])
	}
}

fn get_email(obj: &json::JsonValue) -> String {
	if obj["div"][1].is_array() {
		json_get_value(&obj["div"][1][1]["div"]["div"]["a"]["#text"])
	} else {
		String::new()
	}
}

fn handle(site_list: &mut Vec<Site>, positions : &HashMap<String, &json::JsonValue>, obj : &json::JsonValue) {
	let id = json_get_value(&obj["@data-entity-id"]);
	let pos = positions.get(&id);
	let site = Site {
		id: 	id,
		name: 	json_get_value(&obj["div"][0]["a"]["span"]),
		phone: 	get_phone(&obj),
		email: 	get_email(&obj),
		lat: 	json_get_value(&pos["lat"]),
		lon: 	json_get_value(&pos["lon"]),
		href: 	json_get_value(&obj["div"][0]["a"]["@href"]),
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
