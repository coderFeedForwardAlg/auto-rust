use std::collections::HashMap;
use crate::schema::Col;

#[derive(Debug)]
pub struct Row {
    pub name: String,
    pub cols: Vec<Col>,
}

// Helper function to insert multiple PostgreSQL types for a single Rust type
pub fn insert_multiple(map: &mut HashMap<String, String>, rust_type: &str, postgres_types: &[&str]) {
    for pg_type in postgres_types {
        map.insert(pg_type.to_string(), rust_type.to_string());
    }
}

pub fn create_type_map() -> HashMap<String, String> {
    let mut type_map = HashMap::new();

    insert_multiple(&mut type_map, "bool", &["BOOL"]);
    insert_multiple(&mut type_map, "i8", &["CHAR"]);
    insert_multiple(&mut type_map, "i16", &["SMALLINT", "SMALLSERIAL", "INT2"]);
    insert_multiple(&mut type_map, "i32", &["INT", "SERIAL", "INT4", "INTEGER"]);
    insert_multiple(&mut type_map, "i64", &["BIGINT", "BIGSERIAL", "INT8"]);
    insert_multiple(&mut type_map, "f32", &["REAL", "FLOAT4"]);
    insert_multiple(&mut type_map, "f64", &["DOUBLE PRECISION", "FLOAT8"]);
    insert_multiple(&mut type_map, "String", &["VARCHAR", "CHAR(N)", "TEXT", "NAME", "CITEXT"]);
    insert_multiple(&mut type_map, "Vec<u8>", &["BYTEA"]);
    insert_multiple(&mut type_map, "()", &["VOID"]);
    // insert_multiple(&mut type_map, "PgInterval", &["INTERVAL"]);
    // insert_multiple(&mut type_map, "PgMoney", &["MONEY"]);
    // insert_multiple(&mut type_map, "PgLTree", &["LTREE"]);
    // insert_multiple(&mut type_map, "PgLQuery", &["LQUERY"]);
    // insert_multiple(&mut type_map, "PgCiText", &["CITEXT1"]);
    // insert_multiple(&mut type_map, "PgCube", &["CUBE"]);
    // insert_multiple(&mut type_map, "PgPoint", &["POINT"]);
    // insert_multiple(&mut type_map, "PgLine", &["LINE"]);
    // insert_multiple(&mut type_map, "PgLSeg", &["LSEG"]);
    // insert_multiple(&mut type_map, "PgBox", &["BOX"]);
    // insert_multiple(&mut type_map, "PgPath", &["PATH"]);
    // insert_multiple(&mut type_map, "PgPolygon", &["POLYGON"]);
    // insert_multiple(&mut type_map, "PgCircle", &["CIRCLE"]);
    // insert_multiple(&mut type_map, "PgHstore", &["HSTORE"]);

    // Add the new pairs
    // type_map.insert("NUMERIC".to_string(), "bigdecimal::Decimal".to_string());
    type_map.insert("TIMESTAMPTZ".to_string(), "chrono::DateTime<Utc>".to_string());
    type_map.insert("TIMESTAMP".to_string(), "chrono::NaiveDateTime".to_string());
    type_map.insert("DATE".to_string(), "chrono::NaiveDate".to_string());
    type_map.insert("TIME".to_string(), "chrono::NaiveTime".to_string());
    type_map.insert("TIMETZ".to_string(), "PgTimeTz".to_string());
    type_map.insert("UUID".to_string(), "uuid::Uuid".to_string());
    insert_multiple(&mut type_map, "ipnetwork::IpNetwork", &["INET", "CIDR"]);
    insert_multiple(&mut type_map, "std::net::IpAddr", &["INET", "CIDR"]);
    insert_multiple(&mut type_map, "ipnet::IpNet", &["INET", "CIDR"]);
    insert_multiple(&mut type_map, "mac_address::MacAddress", &["MACADDR"]);
    insert_multiple(&mut type_map, "bit_vec::BitVec", &["BIT", "VARBIT"]);
    // insert_multiple(&mut type_map, "Json<T>", &["JSON", "JSONB"]); //  *******  TODO:fix ********* //
    insert_multiple(&mut type_map, "serde_json::Value", &["JSON", "JSONB"]);
    insert_multiple(&mut type_map, "&serde_json::value::RawValue", &["JSON", "JSONB"]);

    // Handle PgRange<T> types
    type_map.insert("INT8RANGE".to_string(), "PgRange<i64>".to_string());
    type_map.insert("INT4RANGE".to_string(), "PgRange<i32>".to_string());
    type_map.insert("TSRANGE".to_string(), "PgRange<PgTimestamp>".to_string()); // Assuming you have a PgTimestamp type
    type_map.insert("TSTZRANGE".to_string(), "PgRange<PgTimestampTz>".to_string()); // Assuming you have a PgTimestampTz type
    type_map.insert("DATERANGE".to_string(), "PgRange<PgDate>".to_string()); // Assuming you have a PgDate type
    type_map.insert("NUMRANGE".to_string(), "PgRange<PgNumeric>".to_string()); // Assuming you have a PgNumeric type


    type_map
}

