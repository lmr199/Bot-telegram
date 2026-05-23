use std::env;
use std::thread;
use std::time::Duration;

use dotenv::dotenv;
use serde::Deserialize;

#[derive(Deserialize)]
struct Response {
    result: Vec<Update>,
}

#[derive(Deserialize)]
struct Update {
    update_id: i64,
    message: Option<Message>,
}

#[derive(Deserialize)]
struct Message {
    text: Option<String>,
    chat: Chat,
}

#[derive(Deserialize)]
struct Chat {
    id: i64,
}

fn get_updates(token: &str, offset: i64) -> Option<Vec<Update>> {

    let url = format!(
        "https://api.telegram.org/bot{}/getUpdates?offset={}",
        token,
        offset
    );

    let response = reqwest::blocking::get(&url).ok()?;

    let data: Response = response.json().ok()?;

    Some(data.result)
}

fn send_message(token: &str, chat_id: i64, text: &str) {

    let url = format!(
        "https://api.telegram.org/bot{}/sendMessage",
        token
    );

    let client = reqwest::blocking::Client::new();

    let _ = client
        .post(&url)
        .form(&[
            ("chat_id", chat_id.to_string()),
            ("text", text.to_string())
        ])
        .send();
}

fn validate_cpf(cpf: &str) -> bool {

    let digits: Vec<u32> = cpf
        .chars()
        .filter(|c| c.is_ascii_digit())
        .map(|c| c.to_digit(10).unwrap())
        .collect();

    if digits.len() != 11 || digits.iter().all(|&d| d == digits[0]) {
        return false;
    }

    let total: u32 = digits[..9]
        .iter()
        .enumerate()
        .map(|(i, &d)| d * (10 - i as u32))
        .sum();

    let mut remainder = (total * 10) % 11;

    if remainder == 10 {
        remainder = 0;
    }

    if remainder != digits[9] {
        return false;
    }

    let total: u32 = digits[..10]
        .iter()
        .enumerate()
        .map(|(i, &d)| d * (11 - i as u32))
        .sum();

    let mut remainder = (total * 10) % 11;

    if remainder == 10 {
        remainder = 0;
    }

    remainder == digits[10]
}

fn main() {

    dotenv().ok();

    let token = env::var("TELEGRAM_TOKEN")
        .expect("TELEGRAM_TOKEN não encontrado");

    let mut last_update_id = 0;

    // Ignora mensagens antigas
    if let Some(updates) = get_updates(&token, 0) {

        if let Some(last) = updates.last() {
            last_update_id = last.update_id;
        }
    }

    loop {

        if let Some(updates) = get_updates(&token, last_update_id + 1) {

            for update in updates {

                last_update_id = update.update_id;

                if let Some(message) = update.message {

                    if let Some(text) = message.text {

                        let reply = if validate_cpf(&text) {
                            "✅ Boa chefe, você existe!!"
                        } else {
                            "❌ Sai daí, robô!!"
                        };

                        send_message(
                            &token,
                            message.chat.id,
                            reply
                        );
                    }
                }
            }
        }

        thread::sleep(Duration::from_secs(2));
    }
}

