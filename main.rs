extern crate reqwest;
extern crate rss;

use reqwest::blocking::Client;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::process::Command;

fn main() {
    // URL of the RSS feed
    let url = "link to RSS from 'my feed' on habr";

    // Create a Reqwest client
    let client = Client::new();

    // Fetch the RSS feed
    let response = client.get(url).send();

    // Handle the response
    let response = match response {
        Ok(res) => res,
        Err(err) => {
            eprintln!("Failed to fetch RSS feed: {}", err);
            return;
        }
    };

    // Parse the RSS feed
    let channel = match response.text() {
        Ok(text) => {
            let reader = BufReader::new(text.as_bytes());
            match rss::Channel::read_from(reader) {
                Ok(channel) => channel,
                Err(err) => {
                    eprintln!("Failed to parse RSS feed: {}", err);
                    return;
                }
            }
        }
        Err(err) => {
            eprintln!("Failed to read response body: {}", err);
            return;
        }
    };

    // Open a file to write the results
    let mut file = match File::create("rss_links") {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Failed to create file: {}", err);
            return;
        }
    };

    // Write the first 5 news titles and links to the file
    for (index, item) in channel.items().iter().enumerate() {
        if index >= 5 {
            break;
        }
        if let Some(title) = item.title() {
            if let Some(link) = item.link() {
                let line = format!("{}\n{}\n------------------------\n", title, link);
                if let Err(err) = file.write_all(line.as_bytes()) {
                    eprintln!("Failed to write to file: {}", err);
                    return;
                }
            } else {
                let line = format!("{}\nN/A\n------------------------\n", title);
                if let Err(err) = file.write_all(line.as_bytes()) {
                    eprintln!("Failed to write to file: {}", err);
                    return;
                }
            }
        } else {
            let line = "Untitled\n".to_string();
            if let Err(err) = file.write_all(line.as_bytes()) {
                eprintln!("Failed to write to file: {}", err);
                return;
            }
        }
    }

    // Send the content of the file via curl
    let rss_content = match fs::read_to_string("rss_links") {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Failed to read file: {}", err);
            return;
        }
    };

    // Set the parameters for the POST request
    let bot_token = "BOT TOKEN";
    let group_id = "CHAT ID";
    let curl_command = format!("curl -X POST -s -F 'text={}' -F 'chat_id={}' 'https://api.telegram.org/bot{}/sendMessage'",
                               rss_content, group_id, bot_token);

    // Execute the curl command
    let output = Command::new("bash")
        .arg("-c")
        .arg(curl_command)
        .output();
}
