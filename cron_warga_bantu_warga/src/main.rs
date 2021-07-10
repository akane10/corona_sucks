use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::from_value;
use serde_json::Value;
use std::{collections::HashMap, fs::File};
use tokio::time;

type Hjson = HashMap<String, Value>;

const URL: &str = "https://sheets.googleapis.com/v4/spreadsheets/1RIcSiQqPCw-6H55QIYwblIQDPpFQmDNC73ukFa05J7c?includeGridData=true";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataSheets {
    title: String,
    row_data: Vec<Vec<String>>,
}

pub async fn fetch_data() -> Result<Vec<DataSheets>, Box<dyn std::error::Error>> {
    let key: String = dotenv::var("API_KEY").expect("Missing api key");
    let url = format!("{}&key={}", URL, key);

    fn get_value<T: DeserializeOwned>(json: &Value, fallback: T) -> T {
        from_value(json.clone()).unwrap_or(fallback)
    }

    let resp = reqwest::get(url).await?.json::<Hjson>().await?;
    let sheets_json = resp.get("sheets").unwrap();
    let sheets: Vec<Value> = get_value(&sheets_json, Vec::new());

    let data: Vec<DataSheets> = sheets[1..]
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

    Ok(data)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let mut interval = time::interval(time::Duration::from_secs(1 * 3600));
    loop {
        interval.tick().await;
        println!("{}", "fetching data...");
        let data = fetch_data().await?;

        println!("{}", "writing file...");
        let file = File::create("../public/data.json")?;
        serde_json::to_writer_pretty(file, &data)?;
        println!("{}", "done");
    }
}
