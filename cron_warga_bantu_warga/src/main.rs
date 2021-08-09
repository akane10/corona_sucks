use chrono::prelude::*;
use data_encoding::HEXUPPER;
use error::Error;
use google_sheet::get_sheet_data;
use google_sheet::get_sheets;
use google_sheet::refresh_token;
use ring::digest::Context;
use ring::digest::Digest;
use ring::digest::SHA256;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use serde_json::from_value;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::path::Path;
use std::time::Instant;
use tokio::time;

mod error;
mod google_sheet;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataSheets {
    sheet_id: u64,
    title: String,
    row_data: Vec<Vec<String>>,
    updated_at: DateTime<Utc>,
}

fn sha256_digest<R: Read>(mut reader: R) -> Result<Digest, Error> {
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    Ok(context.finish())
}

pub fn get_value<T: DeserializeOwned>(json: &Value, fallback: T) -> T {
    from_value(json.clone()).unwrap_or(fallback)
}

fn parse_data(sheet: &Value) -> Result<DataSheets, Error> {
    let title: String = from_value(sheet["properties"]["title"].clone())?;
    let sheet_id: u64 = from_value(sheet["properties"]["sheetId"].clone())?;
    let row_data_json: Vec<Value> = from_value(sheet["data"][0]["rowData"].clone())?;

    let row_data: Vec<Vec<String>> = row_data_json
        .into_iter()
        .map(|x| get_value(&x["values"], Vec::new()))
        .map(|x: Vec<Value>| {
            let formatted_value: Vec<String> = x
                .into_iter()
                .map(|value| get_value(&value["formattedValue"], String::from("")))
                .collect();
            formatted_value
        })
        .filter(|val| !val.into_iter().all(|x| x.is_empty()))
        .collect();

    let data = DataSheets {
        sheet_id,
        title,
        row_data,
        updated_at: Utc::now(),
    };
    Ok(data)
}

fn compare_hash(
    data: &Vec<Value>,
    data_json: Option<&Value>,
) -> Result<(bool, bool, String), Error> {
    let sheets_str: String = serde_json::to_string(&data)?;

    let reader = BufReader::new(sheets_str.as_bytes());
    let digest = sha256_digest(reader)?;
    let sha: String = HEXUPPER.encode(digest.as_ref());

    let (is_not_match, is_new) = match data_json {
        Some(ref val) => {
            if sha.ne(&val["hash"]) {
                (true, false)
            } else {
                (false, false)
            }
        }
        _ => (true, true),
    };

    Ok((is_not_match, is_new, sha))
}

fn get_data(data: Vec<Value>, sheet_id: u64, title: &str) -> Result<Option<DataSheets>, Error> {
    let p = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("public/data/data.json");
    let file = File::open(&p);
    let mut s: HashMap<String, Value> = match file {
        Ok(f) => {
            let reader = BufReader::new(f);
            serde_json::from_reader(reader).unwrap_or(HashMap::new())
        }
        Err(e) => {
            println!("{}", e);
            HashMap::new()
        }
    };

    let mut data_json = s.get_mut(&sheet_id.to_string());

    let (is_not_match, is_new, sha) = compare_hash(&data, data_json.as_deref())?;

    if is_not_match {
        println!("{}", "parsing data...");
        let now = Instant::now();
        let data = parse_data(&data[0])?;
        let elapsed = now.elapsed();
        println!("finished {:#?}", elapsed);

        let total_row = data.row_data.len();

        if is_new {
            println!("INSERT NEW {} - {}", title, sha);
            let val = json!({ "title": title,  "total_row": total_row, "hash": Value::String(sha.clone()) });
            s.insert(sheet_id.to_string(), val);
            let file = File::create(p)?;
            serde_json::to_writer(file, &s)?;
        } else {
            match data_json {
                Some(ref mut val) => {
                    println!("NEW HASH {} - {}", title, sha);
                    val["hash"] = Value::String(sha);
                    val["total_row"] = json!(total_row);
                    let file = File::create(p)?;
                    serde_json::to_writer(file, &s)?;
                }
                _ => println!("should never happens"),
            }
        }
        Ok(Some(data))
    } else {
        println!("SKIP {}: {}", title, sha);
        Ok(None)
    }
}

async fn run(access_token: &str) -> Result<(), Error> {
    let sheet_ids: HashMap<u64, String> = get_sheets(&access_token).await?;
    let p = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("public/data");

    for (id, title) in sheet_ids.iter() {
        if let Ok(val) = get_sheet_data(*id, &access_token).await {
            if let Some(data) = get_data(val, *id, title)? {
                println!("{}", "writing file...");
                let now = Instant::now();
                let filename = format!("{}/{}.json", p.to_str().unwrap(), id);
                let filename_lastest = format!("{}/last_updated.json", p.to_str().unwrap());
                let file = File::create(filename)?;
                let file_lastest = File::create(filename_lastest)?;
                serde_json::to_writer(file, &data)?;

                let last = json!({ "sheet_id": data.sheet_id, "title": data.title, "updated_at": data.updated_at });
                serde_json::to_writer(file_lastest, &last)?;

                let elapsed = now.elapsed();
                println!("finished {:#?}", elapsed);
            } else {
                println!("no data");
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();

    let mut interval = time::interval(time::Duration::from_secs(30));
    loop {
        println!("start...");
        interval.tick().await;
        match refresh_token().await {
            Ok(ref access_token) => {
                match run(access_token).await {
                    Err(e) => println!("{}", e),
                    _ => (),
                }
            }
            Err(e) => println!("{}", e),
        };
    }
}
