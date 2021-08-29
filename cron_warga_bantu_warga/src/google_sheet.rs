use crate::error::Error;
use crate::get_value;
use reqwest::header::HeaderName;
use serde_json::from_value;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;

const URL: &str = "https://sheets.googleapis.com/v4/spreadsheets/1RIcSiQqPCw-6H55QIYwblIQDPpFQmDNC73ukFa05J7c:getByDataFilter";
const URL_REFRESH: &str = "https://oauth2.googleapis.com/token";

type Hjson = HashMap<String, Value>;

pub async fn refresh_token() -> Result<String, Error> {
    let refresh_token: &str = &dotenv::var("REFRESH_TOKEN").expect("Missing refresh token");
    let client_id: &str = &dotenv::var("CLIENT_ID").expect("Missing client id");
    let client_secret: &str = &dotenv::var("CLIENT_SECRET").expect("Missing client secret");
    let url = format!(
        "{}?client_id={}&client_secret={}&refresh_token={}&grant_type=refresh_token",
        URL_REFRESH, client_id, client_secret, refresh_token
    );

    let hdr = HeaderName::from_static("content-length");
    let client = reqwest::Client::new();
    let resp = client
        .post(url)
        .header(hdr, "0")
        .send()
        .await?
        .json::<Hjson>()
        .await?;

    if let Some(token) = resp.get("access_token") {
        Ok(token.to_string())
    } else {
        println!("{:#?}", resp);
        Err(Error::Others("failed to get access_token".to_string()))
    }
}

pub async fn get_sheets(access_token: &str) -> Result<HashMap<u64, String>, Error> {
    let hdr = HeaderName::from_static("content-length");
    let client = reqwest::Client::new();
    let resp = client
        .post(URL)
        .header(hdr, "0")
        .bearer_auth(access_token)
        .send()
        .await?
        .error_for_status()?
        .json::<Hjson>()
        .await?;

    if let Some(sheets_json) = resp.get("sheets") {
        let sheets: Vec<Value> = get_value(&sheets_json, Vec::new());
        let mut data = HashMap::new();

        for sheet in sheets[1..].iter() {
            let id = from_value(sheet["properties"]["sheetId"].clone());
            let title = from_value(sheet["properties"]["title"].clone());

            match (id, title) {
                (Ok(id), Ok(title)) => {
                    data.insert(id, title);
                }
                _ => (),
            }
        }

        Ok(data)
    } else {
        println!("{:#?}", resp);
        Err(Error::Others("failed to get sheets".to_string()))
    }
}

pub async fn get_sheet_data(sheet_id: u64, access_token: &str) -> Result<Vec<Value>, Error> {
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
    let resp = client
        .post(URL)
        .bearer_auth(access_token)
        .json(&req_body)
        .send()
        .await?
        .json::<Hjson>()
        .await?;
    let elapsed = now.elapsed();
    println!("finished {:#?}", elapsed);

    if let Some(val) = resp.get("sheets") {
        let sheets: Vec<Value> = from_value(val.clone())?;
        Ok(sheets)
    } else {
        println!("{:#?}", resp);
        Err(Error::Others("failed to get sheets".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_all() {
        let access_token = refresh_token().await.unwrap();
        get_sheets(&access_token).await.unwrap();
        let res = get_sheet_data(0, &access_token).await;
        assert!(res.is_ok());
    }
}
