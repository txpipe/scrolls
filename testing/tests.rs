use redis::{Client, Commands};
use postgres::{Client as PgClient, NoTls, Row};
use std::collections::HashMap;
use std::fs;
use serde::Deserialize;
use serde::Serialize;
use serde_json;

const REDIS_CONNECTION_STRING: &str = "redis://localhost:6379/";
const PG_CONNECTION_STRING: &str = "postgres://postgres:Sexonthebeachisthebest@localhost:5432/cexplorer";

const START_HASH: &str = "6cec73b3959a07dc4138bd3dc89e1e17544622f9e4171c55b222552a5c3e791e";
const END_HASH: &str = "dcb5733291c907ae6cc9e9b59f93856ec49d60f724adc4f154c6929b57ce060a";

struct Test<'a> {
    name: &'a str
}

#[derive(Deserialize, Serialize)]
struct KeyValue {
    key: String,
    value: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define the tests to run
    let tests = vec![
        Test {
            name: "balance_by_address",
        },
        Test {
            name: "txcount_by_address",
        },
        // Test {
        //     name: "assets_by_address",
        // },
        // Test {
        //     name: "balance_by_stake_key",
        // },
        // Test {
        //     name: "txcount_by_stake_key",
        // },
        // Test {
        //     name: "assets_by_stake_key",
        // },
        // Test {
        //     name: "tx_count_by_asset",
        // },
        // Test {
        //     name: "supply_by_asset",
        // },
        // Test {
        //     name: "addresses_by_asset",
        // },
        // Test {
        //     name: "stake_keys_by_asset",
        // },
    ];

    // Connect to Redis
    let redis_client = Client::open(REDIS_CONNECTION_STRING)?;
    let mut redis_connection = redis_client.get_connection()?;

    // Connect to PostgreSQL
    let mut pg_client = PgClient::connect(PG_CONNECTION_STRING, NoTls)?;

    // Run each test
    for test in tests {
        println!("Running test: {}", test.name);

        // Load SQL query from file
        let mut query = fs::read_to_string(format!("sql/{}.sql", test.name))?;

        query = query.replace("{{ start_hash }}", START_HASH);
        query = query.replace("{{ end_hash }}", END_HASH);

        // Fetch results from PostgreSQL using the SQL query
        let rows: Vec<Row> = pg_client.query(&query, &[])?;

        // Fetch a list of keys from Redis using the wildcard selection
        let keys: Vec<String> = redis_connection.keys(format!("{}.*", test.name))?;

        if rows[0].get::<usize, String>(1).starts_with('[') && rows[0].get::<usize, String>(1).ends_with(']') {
            compare_hash_map(rows, keys, &mut redis_connection)?;
        } else {
            compare_string(rows, keys, &mut redis_connection)?;
        }

        println!("Test passed")
    }

    Ok(())
}

fn compare_string(rows: Vec<Row>, keys: Vec<String>, redis_connection: &mut redis::Connection) -> Result<(), Box<dyn std::error::Error>> {
    // Convert rows to a map
    let mut postgres_values_map: HashMap<String, String> = HashMap::new();
    for row in rows {
        let key: String = row.get(0);
        let value: String = row.get(1);
        postgres_values_map.insert(key, value);
    }

    // Fetch the values of the keys from Redis and store them in a HashMap
    let mut redis_values_map: HashMap<String, String> = HashMap::new();
    for key in keys {
        let value: String = redis_connection.get(&key)?;
        redis_values_map.insert(key, value);
    }

    if postgres_values_map.len() != redis_values_map.len() {
        return Err("The number of keys in both maps do not match".to_string().into());
    }
    
    // Check that postgres_values_map and redis_values_map are identical
    for (key, value) in &postgres_values_map {
        // Check that the key exists in the Redis key set
        if !redis_values_map.contains_key(key) {
            return Err(format!("Key {} not found in Redis key set", key).into());
        }
        // Check that the values are the same
        if redis_values_map.get(key).unwrap() != value {
            return Err(format!("Values for key {} do not match", key).into());
        }
    }

    Ok(())
}

fn compare_hash_map(rows: Vec<Row>, keys: Vec<String>, redis_connection: &mut redis::Connection) -> Result<(), Box<dyn std::error::Error>> {
    let mut postgres_values_map: HashMap<String, HashMap<String, String>> = HashMap::new();
    for row in rows {
        let key: String = row.get(0);
        let value: String = row.get(1);

        let key_values: Vec<KeyValue> = serde_json::from_str(&value)?;

        let mut sub_map = HashMap::new();
        for kv in key_values {
            sub_map.insert(kv.key, kv.value);
        }
        postgres_values_map.insert(key, sub_map);
    }

    let mut redis_values_map: HashMap<String, HashMap<String, String>> = HashMap::new();
    for key in keys {
        let redis_sorted_set: Vec<(String, String)> = redis_connection.zrange_withscores(&key, 0, -1)?;

        let mut sub_map = HashMap::new();
        for (k, v) in redis_sorted_set {
            sub_map.insert(k, v);
        }
        redis_values_map.insert(key, sub_map);
    }

    if postgres_values_map.len() != redis_values_map.len() {
        return Err("The number of keys in both maps do not match".to_string().into());
    }

    for (key, postgres_sub_map) in &postgres_values_map {
        match postgres_values_map.get(key) {
            Some(redis_sub_map) => {
                if postgres_sub_map.len() != redis_sub_map.len() {
                    return Err(format!(
                        "The number of keys in sub-maps for key {} do not match",
                        key
                    ).into());
                }

                for (sub_key, redis_value) in postgres_sub_map {
                    match redis_sub_map.get(sub_key) {
                        Some(postgres_value) => {
                            if redis_value != postgres_value {
                                return Err(format!(
                                    "Values for key {} and sub-key {} do not match",
                                    key, sub_key
                                ).into());
                            }
                        }
                        None => {
                            return Err(format!(
                                "Sub-key {} not found in Redis map for key {}",
                                sub_key, key
                            ).into());
                        }
                    }
                }
            }
            None => {
                return Err(format!("Key {} not found in Redis map", key).into());
            }
        }
    }

    Ok(())
}