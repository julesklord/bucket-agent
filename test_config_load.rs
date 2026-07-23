use std::collections::HashMap;
use toml::Value;

fn main() {
    let config_str = std::fs::read_to_string("/home/julesklord/.bucket/config.toml").unwrap();
    let config: Value = config_str.parse().unwrap();
    
    println!("Full config: {:#?}", config);
    
    // Check if model section exists
    if let Some(model_section) = config.get("model") {
        println!("\nmodel section found: {:#?}", model_section);
        if let Value::Table(table) = model_section {
            for (key, value) in table {
                println!("  model key: {}", key);
                println!("  model value: {:#?}", value);
            }
        }
    } else {
        println!("\nmodel section NOT found!");
    }
}
