use std::{thread};



#[allow(dead_code)]
pub fn downloadvideo(url : &String) {
    let url = url.clone();
    thread::spawn(move ||{
        println!("c'est url {url}");
    });
}

pub fn get_image(url : &String, name : &String) {
    let url = url.clone();
    let name = name.clone();
    thread::spawn(move ||{
        println!("c'est url {url}, et le nom {name}");
        if let Some(id) = extract_param(&url, "v"){
            println!("Extracted ID: {}", id);
        } else {
            println!("No ID found in the URL");
        }
    });
}

fn extract_param(url: &str, key: &str) -> Option<String> {
    let key_eq = format!("{}=", key);
    let start = url.find(&key_eq)? + key_eq.len();
    let end = url[start..].find('&').map(|i| start + i).unwrap_or(url.len());
    Some(url[start..end].to_string())
}