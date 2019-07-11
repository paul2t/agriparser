#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_assignments)]
#![allow(unused_variables)]
#![allow(unused_macros)]

#[macro_use]
extern crate json;
extern crate byteorder;
extern crate roxmltree;

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
use reqwest::header;
use reqwest::header::HeaderMap;


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
	axereal();
	println!("");
	soufflet();
	println!("");
	scribblemaps();
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

fn json_str(obj: &json::JsonValue) -> &str {
	// if obj.is_null() {
		// ""
	// } else {
		// println!("{:?}", obj.as_str());
		match obj.as_str() {
			Some(s) => s,
			None => "",
		}
	// }
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

fn format(data: Vec<Vec<String>>, keys: &[&str]) -> String {
	let sep = "\t";

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

	let data_utf16: Vec<u16> = data.encode_utf16().collect();
    // data_utf16.push(0);

    let mut result: Vec<u8> = Vec::new();
    for it in data_utf16 {
    	let _ = result.write_u16::<LittleEndian>(it);
    }

	trynone!(f.write(&result[..]));

	return Some(());
}

fn axereal() -> Option<()> {
	let name = "axereal";
	println!("=> {}", name);

	let data = download_json("https://www.axereal.com/geojsoncarte.json", &name)?;

    if data["features"].is_null() { println!("No data found in the json file"); return None; }
    if data["features"].len() <= 0 { println!("No elements found in the features"); return None; }

	let custom_keys = [ "lat", "lon", "type" ];
	let prop_keys = [ "name", "description", "Type_site", "Societe", "Adresse 1", "Adresse 2", "CP", "Ville", "Pays", "Tel", "Tid", "Tid_parent", "Type_parent" ];
	let mut keys : Vec<&str> = vec![];
	keys.extend_from_slice(&custom_keys);
	keys.extend_from_slice(&prop_keys);

	let mut site_list: Vec<Vec<String>> = vec![];

	for it in data["features"].members() {
		let mut values: Vec<String> = vec![];
		json_parse_array(&it["geometry"]["coordinates"], 2, &mut values);
		json_parse_names(&it["properties"], &prop_keys, &mut values);
		site_list.push(values);
	}

	output(&format(site_list, &keys), &name)?;

    println!("Done.");
	return Some(());
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
	println!("=> {}", name);

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

	let mut site_list: Vec<Vec<String>> = vec![];

    for it in data["overlays"].members() {
		let mut values: Vec<String> = vec![];
    	if it["description"].is_null() { continue; }

    	json_parse_array(&it["points"][0], 2, &mut values);
	    values.push(get_type(&it["styleId"], &legend));
		json_parse_names(&it, &prop_keys, &mut values);
		site_list.push(values);
    }

    let description_index = keys.iter().position(|&r| r == "description").unwrap();
    let title_index = keys.iter().position(|&r| r == "title").unwrap();

    site_list.sort_by(|a, b| {
    	match a[title_index].cmp(&b[title_index]) { // title
	    	Ordering::Equal => a[description_index].cmp(&b[description_index]), // description
	    	other => other,
	    }
    });

	output(&format(site_list, &keys), &name)?;

	println!("Done.");
	return Some(());
}


fn json_write_to_file(path: &str, data: &json::JsonValue) -> Option<()> {
	println!("Writing json to {}", path);
	trynone!(write!(&mut BufWriter::new(&trynone!(File::create(&path))), "{:#}", data));
	return Some(());
}

fn write_to_file(path: &str, content: &str) -> Option<()> {
	println!("Writing to {}", path);
	trynone!(write!(&mut BufWriter::new(&trynone!(File::create(&path))), "{}", content));
	return Some(());
}


fn soufflet() -> Option<()> {
	let name = "soufflet";
	println!("=> {}", name);

	let url = "https://www.soufflet.com/fr/nos-implantations";
	let content = download(url)?;
	write_to_file(&format!("{}.html", name), &content);

	println!("Parsing result");
	let settings_key = "drupal-settings-json";
	let settings_index = match content.find(&settings_key) { Some(n) => n + settings_key.len(), None => { println!("ERROR: Could not find '{}' in {}.html", settings_key, name); return None; }, };
	let mut content = &content[settings_index..];
	while !content.starts_with('{') { content = &content[1..]; }

	let end_index = match content.find("</script>") { Some(n) => n, None => { println!("ERROR: Could not find '</script>' after {} in {}.html", settings_key, name); return None; }, };
	content = &content[..end_index];
	let data = trynone!(json::parse(&content));
	json_write_to_file(&format!("{}.json", name), &data);

	println!("Listing sites");
	let mut ids : Vec<&str> = Vec::new();
	let mut positions : HashMap<&str, &json::JsonValue> = HashMap::new();
	for it in data["leaflet"]["leaflet-map"]["features"].members() {
		let s = json_str(&it["id"]);
		if !s.is_empty() {
			ids.push(s);
			positions.insert(s, &it);
			// println!("id => '{}'", s);
		}
	}
	println!("Found {} sites", ids.len());
	assert!(ids.len() == positions.len());
	// println!("{:?}", positions);

	let view_args = ids.join("+");

	let disable_download = false;
	let text = if !disable_download {

		let url = "https://www.soufflet.com/fr/views/ajax?_wrapper_format=drupal_ajax";

		let mut headers = HeaderMap::new();
		headers.insert(header::ACCEPT, "application/json, text/javascript, */*; q=0.01".parse().unwrap());
		headers.insert(header::ORIGIN, "https://www.soufflet.com".parse().unwrap());
		headers.insert("X-Requested-With", "XMLHttpRequest".parse().unwrap());
		headers.insert(header::DNT, "1".parse().unwrap());
		headers.insert(header::CONTENT_TYPE, "application/x-www-form-urlencoded; charset=UTF-8".parse().unwrap());

		let params = [
			("ajax_page_state[libraries]", "blazy/load,captcha/base,core/html5shiv,facets/drupal.facets.checkbox-widget,facets/drupal.facets.views-ajax,gdpr_compliance/popup,google_analytics/google_analytics,gs_azure_app_insights/gs_azure_app_insights.tracker,gs_theme/global-scripts,gs_theme/global-styling,gs_theme/newsletter-scripts,leaflet/leaflet-drupal,leaflet_more_maps/leaflet-more-maps,recaptcha/google.recaptcha_fr,recaptcha/recaptcha,system/base,views/views.ajax,views/views.module"),
			("ajax_page_state[theme]", "gs_theme"),
			("ajax_page_state[theme_token]", ""),
			("_drupal_ajax", "1"),
			("pager_element", "0"),
			("view_args", &view_args),
			("view_base_path", "our-locations"),
			("view_display_id", "block_list_result"),
			("view_dom_id", "1cbfb4bd57c4e48be38803d3d613b621fa944d0a7809ded947643bba3bc08408"),
			("view_name", "locations_map"),
			("view_path", "/fr/nos-implantations"),
		];


		println!("Downloading {}", url);
		let client = reqwest::Client::new();
		let mut resp = trynone!(client.post(url).form(&params).headers(headers).send());
		trynone!(resp.text())
	} else {
		let path = format!("{}_xml.json", name);
		println!("reading from file {}", path);
		trynone!(fs::read_to_string(&path))
	};

	println!("Parsing result");
	let data = trynone!(json::parse(&text));

	if name.len() > 0 && !disable_download {
		json_write_to_file(&format!("{}_xml.json", name), &data);
	}

	println!("Extracting infos from json");
	let xmls = &data[2]["data"].as_str()?;
	// println!("{}", xmls.len());
	let xmld = xmls.replace(" class=\"blocImplantation\"\"", " class=\"blocImplantation\"").replace("<br>", "<br/>");
	// write_to_file(&format!("{}.xml", name), &xmld);

	let keys = [ "lat", "lon", "name", "address", "postcode", "city", "admin_area", "country", "phone", "email", "ID", "href", ];
	let addr_keys = ["address-line1", "postal-code", "locality", "administrative-area", "country" ];

	let doc = trynone!(roxmltree::Document::parse(&xmld));
	// write_to_file(&format!("{}_formatted.xml", name), &format!("{:?}", &doc));

	let mut site_list : Vec<Vec<String>> = Vec::new();
	for node in doc.root_element().children() {
		if !node.is_element() { continue; }
		if node.tag_name().name() != "div" { continue; }

		let country = xml_select_child_text(&node, &[("p", &[("class", Some("nomPays"))])]).trim();
		// println!(">>>>>>>>>>>>>>> country: {} <<<<<<<<<<<<<<<<<<<", country);

		
		for node_group in node.children() {
			if !node_group.is_element() { continue; }
			if node_group.tag_name().name() != "div" { continue; }
			if node_group.attribute("class") != Some("listeImplantations") { continue; }


			for mut node_site in node_group.children() {
				if !node_site.is_element() { continue; }
				if node_site.tag_name().name() != "div" { continue; }
				node_site = match xml_select_child(&node_site, &[("div", &[])]) { Some(n) => n, None => continue, };

				let mut values : Vec<String> = vec![];

				let id = node_site.attribute("data-entity-id").unwrap_or("");
				let text = xml_select_child_text(&node_site, &[("div", &[("class", Some("nomImplantation"))]), ("a", &[]), ("span", &[])]).trim();
				// if text.len() > 0 {println!("{}", text); }
				let href = xml_select_child_attribute(&node_site, &[("div", &[("class", Some("nomImplantation"))]), ("a", &[])], "href").trim();
				// if href.len() > 0 {println!("{}", href); }

				let lat = "";
				let lon = "";

				fn format_text(s: &str) -> String {
					s	.replace("\t", "    ") // To avoid inserting a separator
						.replace("–", "-") // Character not supported by Excel
						.trim().to_string()
				}

				// println!("{}", id);
				let mut lat = String::new();
				let mut lon = String::new();
				if let Some(pos) = positions.get(&id) {
					// println!("- {}", json_str(&pos["lat"]));
					lat = pos["lat"].to_string();
					lon = pos["lon"].to_string();
					// println!("=> '{}' '{}' <=", lat, lon);
				}
				
				values.push(lat.to_string());
				values.push(lon.to_string());
				values.push(format_text(text));

				if let Some(node_address) = xml_select_child(&node_site, &[("span", &[("class", Some("adresseImplantation"))]), ("div", &[]), ("div", &[]), ("p", &[("class", Some("address"))])]) {
					for k in addr_keys.iter() {
						values.push(format_text(xml_select_child_text(&node_address, &[("span", &[("class", Some(k))])]).trim()));
					}
				} else {
					for k in addr_keys.iter() {
						values.push(String::new());
					}
				}


				let phone = xml_select_child_text(&node_site, &[("div", &[("class", Some("coordonneesImplantation"))]), ("div", &[]), ("div", &[]), ("div", &[]), ("a", &[("class", Some("phone-link"))])]);
				let email = xml_select_child_text(&node_site, &[("div", &[("class", Some("coordonneesImplantation"))]), ("div", &[]), ("div", &[]), ("div", &[]), ("a", &[("class", None), ("title", None)])]);
				// println!("phone: {} / email: {}", phone, email);
				values.push(phone.to_string());
				values.push(email.to_string());

				values.push(id.to_string());
				values.push(href.to_string());
				
				site_list.push(values);
			}

		}
	}

	output(&format(site_list, &keys), &name)?;

	println!("Done.");
	return Some(());
}



fn xml_select_child<'a, 'b>(node: &'a roxmltree::Node<'b, 'b>, selector: &[(&str, &[(&str, Option<&str>)])]) -> Option<roxmltree::Node<'b, 'b>> {
	if selector.len() <= 0 {
		return Some(*node);
	}

	let param = selector[0];
	for child in node.children() {
		if child.is_element() && child.tag_name().name() == param.0 {
			let mut found = true;
			for (att, val) in param.1 {
				if child.attribute(*att) != *val {
					found = false;
				}
			}
			if found {
				let result = xml_select_child(&child, &selector[1..]);
				if result != None {
					return result;
				}
			}
		}
	}

	return None;
}

fn xml_select_child_text<'a>(node: &'a roxmltree::Node, selector: &[(&str, &[(&str, Option<&str>)])]) -> &'a str {
	if let Some(selected) = xml_select_child(node, selector) {
		selected.text().unwrap_or("")
	} else {
		""
	}
}

fn xml_select_child_attribute<'a>(node: &'a roxmltree::Node, selector: &[(&str, &[(&str, Option<&str>)])], name: &'a str) -> &'a str {
	if let Some(selected) = xml_select_child(node, selector) {
		selected.attribute(name).unwrap_or("")
	} else {
		""
	}
}
