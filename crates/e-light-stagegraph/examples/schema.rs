use e_light_stagegraph::Stage;


fn main() {
    let schema = schemars::schema_for!(Stage);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
