use serde::{Deserialize, Serialize};
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

    let resp = reqwest::get(url).await?.json::<Hjson>().await?;
    let sheets_json = resp.get("sheets").unwrap();
    let sheets: Vec<Value> = serde_json::from_value(sheets_json.clone()).unwrap_or(Vec::new());

    let data: Vec<DataSheets> = sheets[1..]
        .into_iter()
        .map(|sheet| {
            let row_data_json: Vec<Value> =
                serde_json::from_value(sheet["data"][0]["rowData"].clone()).unwrap_or(Vec::new());
            let title: String = serde_json::from_value(sheet["properties"]["title"].clone())
                .unwrap_or(String::from(""));

            (title, row_data_json)
        })
        .map(|(title, row_data_json)| {
            let row_data: Vec<Vec<String>> = row_data_json
                .into_iter()
                .map(|x| serde_json::from_value(x["values"].clone()).unwrap_or(Vec::new()))
                .map(|x: Vec<Value>| {
                    let formatted_value: Vec<String> = x
                        .into_iter()
                        .map(|value| {
                            let formatted_value: String =
                                serde_json::from_value(value["formattedValue"].clone())
                                    .unwrap_or(String::from(""));
                            formatted_value
                        })
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
