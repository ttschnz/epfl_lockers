use json::JsonValue;
use reqwest::header;
use serde_derive::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockerGroup {
    pub name: String,
    pub coordinates: (f64, f64),
    pub level: i32,
}
impl LockerGroup {
    const URL: &str = "https://campus.epfl.ch/deploy/backend_proxy/14620/lockers/js-map";

    pub fn request(auth_pcsessid: &str) -> Result<Vec<Self>, Box<dyn std::error::Error>> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Content-Type",
            "application/vnd.apache.thrift.json; charset=UTF-8"
                .parse()
                .unwrap(),
        );
        headers.insert("X-PC-APP-IDENTIFIER", "org.pocketcampus".parse().unwrap());
        headers.insert("X-PC-APP-VERSION", "3".parse().unwrap());
        headers.insert("X-PC-AUTH-PCSESSID", auth_pcsessid.parse().unwrap());
        headers.insert("X-PC-LANG-CODE", "en".parse().unwrap());
        headers.insert("X-PC-PUSHNOTIF-OS", "WEB".parse().unwrap());

        let client = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();
        let res = client.post(Self::URL)
             .headers(headers)
             .body("[1,\"getMapLayerItems2\",1,0,{\"1\":{\"rec\":{\"1\":{\"set\":[\"str\",1,\"available_locker_groups\"]},\"2\":{\"tf\":0}}}}]")
             .send()?
             .text()?;

        Self::parse(&res)
    }

    pub fn parse(json_text: &str) -> Result<Vec<Self>, Box<dyn std::error::Error>> {
        let root = json::parse(json_text)?;
        let mut locker_groups = Vec::new();
        if root.is_array() {
            let set = &root[4]["0"]["rec"]["2"]["set"];

            let count_present = set[1]
                .as_usize()
                .ok_or("count present could not be determined")?;
            for index in 0..count_present {
                locker_groups.push(Self::from_json_value(&set[index + 2])?);
            }
        }
        Ok(locker_groups)
    }
    fn from_json_value(set: &JsonValue) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            name: String::from(set["1"]["str"].as_str().ok_or("name unknown")?),
            coordinates: (
                set["2"]["dbl"].as_f64().ok_or("lat unknown")?,
                set["3"]["dbl"].as_f64().ok_or("long unknown")?,
            ),
            level: set["4"]["i32"].as_i32().ok_or("level unknown")?,
        })
    }
}
