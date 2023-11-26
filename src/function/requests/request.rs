use chrono::Utc;
use reqwest::Client;
use serde_json::Value;

use crate::constant::DAYS;
use crate::function::sqls::general::cache::{get_database_cache, set_database_cache};

/// the number of day before the cache is too old and need to be renewed.
/// Makes a requests to AniList.
///
/// This function takes a JSON object and a boolean flag `always_update` as arguments.
///
/// If `always_update` is set to true, it will directly make a new requests regardless of the cache.
/// Otherwise, if `always_update` is false, it will first check if a cached response exists for the
/// given JSON object. If a cached response exists, it will be returned; if not, a new requests will
/// be made.
///
/// The result of this function is a JSON response in the form of a String.
///
/// The function is asynchronous. It may need to wait for I/O operations (network and disk) to complete.
///
/// # Arguments
///
/// * `json` - A JSON object that represents the requests to be sent.
/// * `always_update` - A boolean flag. If it's true, a new requests will be made regardless of the cache.
///
/// # Examples
///
/// ```
/// let json = serde_json::json!({ "query": "...", "variables": { "id": 12345 }});
/// let always_update = false;
/// let response = make_request_anilist(json, always_update).await;
///
/// println!("{}", response);
/// ```
///
/// # Returns
///
/// * A JSON response in the form of a String.
///
pub async fn make_request_anilist(json: Value, always_update: bool) -> String {
    if always_update {
        do_request(json, always_update).await
    } else {
        get_cache(json.clone()).await
    }
}

/// Asynchronously retrieves cached requests data from a SQLite database.
///
/// This function takes a JSON object (`json`) as input and queries a SQLite database (`./cache.db`)
/// to try and retrieve any cached requests data that matches the input.
///
/// If the database does not contain any matching requests data, the function executes
/// an unspecified requests operation (`do_request()`) with the given `json` object.
/// Furthermore, if any cached data hasn't been updated within a specified time period
/// (calculated as the number of `DAYS` times the number of seconds in a day),
/// `do_request()` will be executed as well.
///
/// # Arguments
///
/// * `json` - A Serde JSON object representing the requests data to be queried in the SQLite database.
///
/// # Returns
///
/// * A `String` value representing the response related to the provided requests data (i.e., `json`).
/// This `String` is either directly fetched from the database or is the result of a fresh requests.
///
/// # Errors
///
/// This function will panic (`unwrap()`) if there is any issue executing or fetching from the SQL statement.
/// It will also panic if `get_pool()` or `do_request()` encounter errors.
///
/// # Examples
///
/// ```rust
/// let json = serde_json::json!({"some": "data"});
/// let result = get_cache(json);
/// ```
///
/// # Note
///
/// This function is `async`, meaning it returns a `Future`. It must be `await`ed in an asynchronous context.
async fn get_cache(json: Value) -> String {
    let (json_resp, response, last_updated): (Option<String>, Option<String>, Option<i64>) =
        get_database_cache(json.clone()).await;

    if json_resp.is_none() || response.is_none() || last_updated.is_none() {
        do_request(json.clone(), false).await
    } else {
        let updated_at = last_updated.unwrap();
        let duration_since_updated = Utc::now().timestamp() - updated_at;
        if duration_since_updated < (DAYS * 24 * 60 * 60) {
            response.unwrap()
        } else {
            do_request(json.clone(), false).await
        }
    }
}

/// Asynchronous function to add a JSON requests and its corresponding response to a cache database.
///
/// # Arguments
///
/// * `json` - A Value object representing the JSON requests.
/// * `resp` - A String object containing the response for the JSON requests.
///
/// # Returns
///
/// * A boolean, currently always true, indicating whether the operation was successful.
///
/// # Database interactions
///
/// * Connects to a SQLite database located at "./cache.db" on the local filesystem.
/// * Executes an SQL query to insert the JSON requests, the response, and a timestamp of the last modification into a table named "request_cache".
/// * The parameters provided to the SQL query are the JSON requests, the response, and the current timestamp.
/// * In case the table already contains a record with the same JSON requests, it will be replaced due to the "REPLACE" keyword.
///
/// # Panics
///
/// * If the operation to execute the SQL query fails, the program will panic due to the use of `unwrap()`.
///
/// # Example
///
/// ```rust
/// let json_request: Json = get_json_request_represented_as_value();
/// let response: String = get_the_corresponding_response_as_string();
/// add_cache(json_request, response).await;
/// ```
///
/// # Note
///
/// This function always currently returns true, so the boolean return value is currently not indicative of whether the operation was successful. This might be subject for future improvements, such as implementing error handling, to prevent possible panics.
async fn add_cache(json: Value, resp: String) -> bool {
    set_database_cache(json, resp).await;

    true
}

/// This function executes a POST requests to a specific URL (https://graphql.anilist.co/).
///
/// # Arguments
///
/// * `json: Value` - The JSON content that acts as the body for the POST requests.
/// This is cloned into the body of the requests.
///
/// * `always_update: bool` - A boolean flag determining whether the response should be added to the cache.
/// If `always_update` is false, the response from the server will be added to the cache.
///
/// # Return
///
/// This function returns a `String`, the content of the response obtained from the server.
///
/// This function is async, so it'll return a Future that should be awaited for.
///
/// # Error Handling
///
/// Note that in case of any error occuring while executing the requests or retrieving the response,
/// the function will panic due to the use of `unwrap()`.
///
/// # Usage
///
/// ```
/// let resp = do_request(json_data, true).await;
/// println!("{}", resp);
/// ```
///
/// # Features
///
/// * Uses POST method to send the requests.
/// * Adds common headers such as "Content-Type" and "Accept".
/// * Caches the response if the `always_update` flag is set to false.
///
/// # Improvements
///
/// For better error handling, consider replacing `unwrap()` calls with proper error handling code.
///
/// # Async
///
/// This function is `async` and therefore needs to be awaited for when called.
async fn do_request(json: Value, always_update: bool) -> String {
    let client = Client::new();
    let res = client
        .post("https://graphql.anilist.co/")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(json.clone().to_string())
        .send()
        .await
        .unwrap()
        .text()
        .await;
    let resp = res.unwrap();
    if !always_update {
        add_cache(json.clone(), resp.clone()).await;
    }
    resp
}
