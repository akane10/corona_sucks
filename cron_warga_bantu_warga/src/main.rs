use reqwest::header::HeaderName;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{from_value, json};
use serde_json::Value;
use std::{collections::HashMap, fs::File, path::Path};
use tokio::time;
use std::time::Instant;
use data_encoding::HEXUPPER;
use ring::digest::{Context, Digest, SHA256};
use std::io::{BufReader, Read};
use error::Error;
use chrono::prelude::*;

mod error;

const URL: &str = "https://sheets.googleapis.com/v4/spreadsheets/1RIcSiQqPCw-6H55QIYwblIQDPpFQmDNC73ukFa05J7c:getByDataFilter";
const URL_REFRESH: &str = "https://oauth2.googleapis.com/token";

type Hjson = HashMap<String, Value>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataSheets {
    sheet_id: u64,
    title: String,
    row_data: Vec<Vec<String>>,
    updated_at: DateTime<Utc>
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

fn get_value<T: DeserializeOwned>(json: &Value, fallback: T) -> T {
    from_value(json.clone()).unwrap_or(fallback)
}

async fn refresh_token(refresh_token: &str) -> Result<String, Error> {
    let client_id: &str = &dotenv::var("CLIENT_ID").expect("Missing client id");
    let client_secret: &str = &dotenv::var("CLIENT_SECRET").expect("Missing client secret");
    let url = format!("{}?client_id={}&client_secret={}&refresh_token={}&grant_type=refresh_token", URL_REFRESH, client_id, client_secret, refresh_token);

    let hdr = HeaderName::from_static("content-length");
    let client = reqwest::Client::new();
    let resp = client.post(url).header(hdr, "0").send().await?.json::<Hjson>().await?;

    if let Some(token) = resp.get("access_token") {
        Ok(token.to_string())
    } else {
        Err(Error::Others("failed to get access_token".to_string()))
    }
}

pub async fn fetch_data(sheet_id: u64, title: &str, access_token: &str) -> Result<Option<DataSheets>, Error> {
    let req_body = json!({
      "includeGridData": true,
      "dataFilters": [
        {
          "gridRange": {
            "sheetId": sheet_id
          }
        }
      ]
    });

    println!("{}", "fetching data...");
    let now = Instant::now();
    let client = reqwest::Client::new();
    let resp = client.post(URL).bearer_auth(access_token).json(&req_body).send().await?.text().await?;
    let elapsed = now.elapsed();
    println!("finished {:#?}", elapsed);

    let reader = BufReader::new(resp.as_bytes());
    let digest = sha256_digest(reader)?;
    let sha: String = HEXUPPER.encode(digest.as_ref());

    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("public/data/data.json");
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

    let is = match s.get_mut(&sheet_id.to_string()) {
        Some(val) => {
            if sha.ne(&val["hash"]) {
                println!("NEW HASH {} - {}", title, sha);
                val["hash"] = Value::String(sha.clone());
                let file = File::create(p)?;
                serde_json::to_writer_pretty(file, &s)?;
                true
            } else {
                false
            }
        },
        _ => {
            println!("INSERT NEW {} - {}", title, sha);
            let val = json!({ "title": title, "hash": Value::String(sha.clone()) });
            s.insert(sheet_id.to_string(), val.clone());
            let file = File::create(p)?;
            serde_json::to_writer_pretty(file, &s)?;
            true
        }
    };
    if is {
        let sheets_json: Value = serde_json::from_str(&resp)?;
        let sheets: Vec<Value> = get_value(&sheets_json["sheets"], Vec::new());

        println!("{}", "parsing data...");
        let now = Instant::now();
        let data: Vec<DataSheets> = sheets
            .into_iter()
            .map(|sheet| {
                let row_data_json: Vec<Value> = get_value(&sheet["data"][0]["rowData"], Vec::new());
                let title: String = get_value(&sheet["properties"]["title"], String::from(""));

                (title, row_data_json)
            })
            .map(|(title, row_data_json)| {
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

                DataSheets { sheet_id, title, row_data, updated_at: Utc::now() }
            })
            .collect();
        let elapsed = now.elapsed();
        println!("finished {:#?}", elapsed);

        if data.len() > 0 {
            Ok(Some(data[0].clone()))
        } else {
            Ok(None)
        }
    } else {
        println!("SKIP {}: {}", title, sha);
        Ok(None)
    }

}

async fn get_sheets(access_token: &str) -> Result<HashMap<u64, String>, Error> {
    let hdr = HeaderName::from_static("content-length");
    let client = reqwest::Client::new();
    let resp = client.post(URL).header(hdr, "0").bearer_auth(access_token).send().await?.json::<Hjson>().await?;

    if let Some(sheets_json) = resp.get("sheets") {
        let sheets: Vec<Value> = get_value(&sheets_json, Vec::new());
        let mut data = HashMap::new();

        for sheet in sheets[1..].iter() {
            let id = from_value(sheet["properties"]["sheetId"].clone());
            let title = from_value(sheet["properties"]["title"].clone());

            match (id, title) {
                (Ok(id), Ok(title)) => {
                data.insert(id, title);
                },
                _ => ()
            }

        }

        Ok(data)
    } else {
        Err(Error::Others("failed to get sheets".to_string()))
    }
}

async fn run(r_token: &str) -> Result<(), Error> {
    let access_token = refresh_token(r_token).await?;
    let sheet_ids: HashMap<u64, String> = get_sheets(&access_token).await?;
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("public/data");

    for (id, title) in sheet_ids.iter() {
        if let Some(data) = fetch_data(*id, title, &access_token).await? {
            println!("{}", "writing file...");
            let now = Instant::now();
            let filename = format!("{}/{}.json", p.to_str().unwrap(), id);
            let filename_lastest = format!("{}/last_updated.json", p.to_str().unwrap());
            let file = File::create(filename)?;
            let file_lastest = File::create(filename_lastest)?;
            serde_json::to_writer_pretty(file, &data)?;

            let last = json!({ "sheet_id": data.sheet_id, "title": data.title, "updated_at": data.updated_at });
            serde_json::to_writer_pretty(file_lastest, &last)?;

            let elapsed = now.elapsed();
            println!("finished {:#?}", elapsed);
        } else {
            println!("no data");
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();
    let r_token: &str = &dotenv::var("REFRESH_TOKEN").expect("Missing refresh token");

    let mut interval = time::interval(time::Duration::from_secs(150));
    loop {
        println!("start...");
        interval.tick().await;
        match run(r_token).await {
            Err(err) => println!("{}", err),
            _ => ()
        }
    }
}
