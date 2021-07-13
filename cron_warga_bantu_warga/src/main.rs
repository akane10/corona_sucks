use reqwest::header::HeaderName;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{from_value, json};
use serde_json::Value;
use std::{collections::HashMap, fs::File, path::Path};
use tokio::time;
use std::time::Instant;

type Hjson = HashMap<String, Value>;

const URL: &str = "https://sheets.googleapis.com/v4/spreadsheets/1RIcSiQqPCw-6H55QIYwblIQDPpFQmDNC73ukFa05J7c:getByDataFilter";
const URL_REFRESH: &str = "https://oauth2.googleapis.com/token";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataSheets {
    title: String,
    row_data: Vec<Vec<String>>,
}

fn get_value<T: DeserializeOwned>(json: &Value, fallback: T) -> T {
    from_value(json.clone()).unwrap_or(fallback)
}

async fn refresh_token(refresh_token: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client_id: &str = &dotenv::var("CLIENT_ID").expect("Missing client id");
    let client_secret: &str = &dotenv::var("CLIENT_SECRET").expect("Missing client secret");
    let url = format!("{}?client_id={}&client_secret={}&refresh_token={}&grant_type=refresh_token", URL_REFRESH, client_id, client_secret, refresh_token);

    let hdr = HeaderName::from_static("content-length");
    let client = reqwest::Client::new();
    let resp = client.post(url).header(hdr, "0").send().await?.json::<Hjson>().await?;

    let token = resp.get("access_token").unwrap().to_string();
    Ok(token)
}

pub async fn fetch_data(sheet_id: u64, access_token: &str) -> Result<Option<DataSheets>, Box<dyn std::error::Error>> {
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
    let resp = client.post(URL).bearer_auth(access_token).json(&req_body).send().await?.json::<Hjson>().await?;
    let elapsed = now.elapsed();
    println!("finished {:#?}", elapsed);

    let sheets_json = resp.get("sheets").unwrap();
    let sheets: Vec<Value> = get_value(&sheets_json, Vec::new());

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
}

async fn get_sheets(access_token: &str) -> Result<Vec<(u64, String)>, Box<dyn std::error::Error>> {
    let hdr = HeaderName::from_static("content-length");
    let client = reqwest::Client::new();
    let resp = client.post(URL).header(hdr, "0").bearer_auth(access_token).send().await?.json::<Hjson>().await?;

    let sheets_json = resp.get("sheets").unwrap();
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let r_token: &str = &dotenv::var("REFRESH_TOKEN").expect("Missing refresh token");

    let mut interval = time::interval(time::Duration::from_secs(100));
    loop {
        println!("start");
        interval.tick().await;
        let access_token = refresh_token(r_token).await?;
        let sheet_ids = get_sheets(&access_token).await?;

        for (id, title) in sheet_ids {
            let data = fetch_data(id, &access_token).await?;

            println!("{}", "writing file...");
            let now = Instant::now();
            let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("public/data").to_str().unwrap().to_string();
            let filename = format!("{}/{}.json", p, title.replace(" ", "").to_lowercase());
            let file = File::create(filename)?;
            serde_json::to_writer_pretty(file, &data)?;
            let elapsed = now.elapsed();
            println!("finished {:#?}", elapsed);
            println!("{}", "done");
        }
    }
}
