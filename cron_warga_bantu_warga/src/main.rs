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

mod error;

const URL: &str = "https://sheets.googleapis.com/v4/spreadsheets/1RIcSiQqPCw-6H55QIYwblIQDPpFQmDNC73ukFa05J7c:getByDataFilter";
const URL_REFRESH: &str = "https://oauth2.googleapis.com/token";

type Hjson = HashMap<String, Value>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataSheets {
    title: String,
    row_data: Vec<Vec<String>>,
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

pub async fn fetch_data(sheet_id: u64, access_token: &str) -> Result<Option<DataSheets>, Error> {
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

    let state_path = concat!(env!("CARGO_MANIFEST_DIR"), "/state.json");
    let file = File::open(state_path);
    let mut s: HashMap<String, String> = match file {
        Ok(f) => {
            let reader = BufReader::new(f);
            serde_json::from_reader(reader).unwrap_or(HashMap::new())
        }
        Err(e) => {
            println!("{}", e);
            let map = HashMap::new();
            map
        }
    };

    let is = match s.get(&sheet_id.to_string()) {
        Some(val) => val.ne(&sha),
        _ => false
    };
    if is {
        println!("insert {}: {}", sheet_id, sha);
        s.insert(sheet_id.to_string(), sha);
        let file = File::create(state_path)?;
        serde_json::to_writer_pretty(file, &s)?;

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

                DataSheets { title, row_data }
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
        println!("skip {}: {}", sheet_id, sha);
        Ok(None)
    }

}

async fn get_sheets(access_token: &str) -> Result<Vec<(u64, String)>, Error> {
    let hdr = HeaderName::from_static("content-length");
    let client = reqwest::Client::new();
    let resp = client.post(URL).header(hdr, "0").bearer_auth(access_token).send().await?.json::<Hjson>().await?;

    if let Some(sheets_json) = resp.get("sheets") {
        let sheets: Vec<Value> = get_value(&sheets_json, Vec::new());

        let data: Vec<(u64, String)> = sheets[1..]
            .into_iter()
            .map(|sheet| {
                 let id = from_value(sheet["properties"]["sheetId"].clone()).unwrap();
                 let title = from_value(sheet["properties"]["title"].clone()).unwrap();
                 (id, title)
            })
            .collect();
        Ok(data)
    } else {
        Err(Error::Others("failed to get sheets".to_string()))
    }
}

async fn run(r_token: &str) -> Result<(), Error> {
    let access_token = refresh_token(r_token).await?;
    let sheet_ids = get_sheets(&access_token).await?;

    for (id, title) in sheet_ids {
        if let Some(data) = fetch_data(id, &access_token).await? {
            println!("{}", "writing file...");
            let now = Instant::now();
            let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("public/data").to_str().unwrap().to_string();
            let filename = format!("{}/{}.json", p, title.replace(" ", "").to_lowercase());
            let file = File::create(filename)?;
            serde_json::to_writer_pretty(file, &data)?;
            let elapsed = now.elapsed();
            println!("finished {:#?}", elapsed);
            println!("{}", "done");
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
        println!("start");
        interval.tick().await;
        match run(r_token).await {
            Err(err) => println!("{}", err),
            _ => ()
        }
    }
}
