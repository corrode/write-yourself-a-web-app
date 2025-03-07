use axum::{extract::Query, routing::get, Router};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    // Initialize the router
    let app = Router::new()
        .route("/", get(hello_world))
        .route("/weather", get(weather));

    println!("Server running on http://0.0.0.0:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Handler function for the root path
async fn hello_world() -> &'static str {
    "Hello, World!"
}

// Handler function for the weather path
async fn weather(Query(params): Query<WeatherQuery>) -> Result<String, String> {
    let lat_long = fetch_lat_long(&params.city)
        .await
        .map_err(|e| e.to_string())?;
    let weather = fetch_weather(lat_long).await.map_err(|e| e.to_string())?;
    Ok(format!("Weather for {}: {:?}", params.city, weather))
}

#[derive(Deserialize)]
struct WeatherQuery {
    city: String,
}

#[derive(Deserialize, Debug)]
struct GeoResponse {
    results: Vec<LatLong>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct LatLong {
    latitude: f64,
    longitude: f64,
}

#[derive(Deserialize, Debug)]
struct WeatherResponse {
    latitude: f64,
    longitude: f64,
    timezone: String,
    hourly: Hourly,
}

#[derive(Deserialize, Debug)]
struct Hourly {
    time: Vec<String>,
    temperature_2m: Vec<f64>,
}

async fn fetch_lat_long(city: &str) -> Result<LatLong, Box<dyn std::error::Error>> {
    let url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
        city
    );
    let response = reqwest::get(&url).await?.json::<GeoResponse>().await?;
    response
        .results
        .get(0)
        .cloned()
        .ok_or_else(|| "No results found".into())
}

async fn fetch_weather(lat_long: LatLong) -> Result<WeatherResponse, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m",
        lat_long.latitude, lat_long.longitude
    );
    let response = reqwest::get(&url).await?.json::<WeatherResponse>().await?;
    Ok(response)
}
