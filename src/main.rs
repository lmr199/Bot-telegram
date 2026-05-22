use std::env;
use std::thread;
use std::time::Duration;
use serde::Deserialize;
use dotenv::dotenv;

#[derive(Deserialize)]
struct Response {
    result: Vec<Update>,
}

#[derive(Deserialize)]
struct Update {
    message: Message,
}

#[derive(Deserialize)]
struct Message {
    date: u64,
    text: Option<String>,
    chat: Chat,
}

#[derive(Deserialize)]
struct Chat {
    id: i64,
}

fn get_updates(token: &str) -> Option<(i64, String, u64)> {
    let url = format!(
      "https://api.telegram.org/bot{}/getUpdates"
        token
    );
    let response = reqwest::blocking::get(&url).ok()?;
    let data: Response = response.json().ok()?;
    let last = data.result.into_iter().next()?;
    let text = last.message.text?;
    Some((last.message.chat.id, text, last.message.date))
}

fn send_message(token: &str, chat_id: i64, text: &str) {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let client = reqwest::blocking::Client::new();
    let _ = client
        .post(&url)
        .form(&[("chat_id", chat_id.to_string()), ("text", text.to_string())])
        .send();
}

fn validate_cpf(cpf: &str) -> bool {
    let digits: Vec<u32> = cpf.chars()
        .filter(|c| c.is_ascii_digit())
        .map(|c| c.to_digit(10).unwrap())
        .collect();

    if digits.len() != 11 || digits.iter().all(|&d| d == digits[0]) {
        return false;
    }

    let total: u32 = digits[..9].iter().enumerate()
        .map(|(i, &d)| d * (10 - i as u32))
        .sum();
    let mut remainder = (total * 10) % 11;
    if remainder == 10 { remainder = 0; }
    if remainder != digits[9] { return false; }

    let total: u32 = digits[..10].iter().enumerate()
        .map(|(i, &d)| d * (11 - i as u32))
        .sum();
    let mut remainder = (total * 10) % 11;
    if remainder == 10 { remainder = 0; }

    remainder == digits[10]
}

fn main() {
    dotenv().ok();
    let token = env::var("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN não encontrado");

    let mut last_date: u64 = 0;

    // Descarta mensagens antigas antes de iniciar
    if let Some((_, _, date)) = get_updates(&token) {
        last_date = date;
    }

    loop {
        if let Some((chat_id, text, date)) = get_updates(&token) {
            if date > last_date {
                last_date = date;
                let reply = if validate_cpf(&text) {
                    "✅ Boa chefe, você existe!!"
                } else {
                    "❌ Sai daí, robô!!"
                };
                send_message(&token, chat_id, reply);
            }
        }
        thread::sleep(Duration::from_secs(1));
    }
}
