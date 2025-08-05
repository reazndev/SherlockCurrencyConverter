use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, vec};
use surf;
use tokio;

#[derive(Debug, Serialize, Deserialize)]
struct FrankfurterResponse {
    base: String,
    date: String,
    rates: HashMap<String, f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SherlockPipeResponse {
    title: String,
    content: String,
    next_content: String,
    actions: Vec<ApplicationAction>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationAction {
    name: Option<String>,
    exec: Option<String>,
    icon: Option<String>,
    method: String,
    exit: bool,
}

impl ApplicationAction {
    fn from_conversion(amount: f64, from: &str, to: &str, result: f64, rate: f64) -> Self {
        let result_text = format!("{:.2} {}", result, to.to_uppercase());
        let detailed_info = format!(
            "{:.2} {} = {:.2} {}\nExchange Rate: 1 {} = {:.6} {}",
            amount,
            from.to_uppercase(),
            result,
            to.to_uppercase(),
            from.to_uppercase(),
            rate,
            to.to_uppercase()
        );

        Self {
            name: Some(result_text),
            exec: Some(detailed_info),
            icon: Some(String::from("preferences-system")),
            method: String::from("copy"),
            exit: true,
        }
    }
}

fn parse_currency_input(input: &str) -> Result<(f64, String, String), String> {
    // Remove "cc" prefix and clean the input
    let cleaned = input.trim();

    // Pattern 1: "100 usd in chf" or "100 usd chf"
    let re1 = Regex::new(r"^(\d+(?:\.\d+)?)\s+([a-zA-Z]{3,4})(?:\s+in)?\s+([a-zA-Z]{3,4})$").unwrap();

    if let Some(caps) = re1.captures(cleaned) {
        let amount: f64 = caps[1].parse().map_err(|_| "Invalid amount")?;
        let from_currency = caps[2].to_uppercase();
        let to_currency = caps[3].to_uppercase();
        return Ok((amount, from_currency, to_currency));
    }

    Err("Invalid format. Use: cc [amount] [from_currency] [to_currency] or cc [amount] [from_currency] in [to_currency]".to_string())
}

fn format_conversion_content(amount: f64, from: &str, to: &str, result: f64, rate: f64, date: &str) -> String {
    format!(
        r#"<span font_desc="monospace">
─── <b><i>Currency Conversion</i></b> ───

<b>{:.2} {}</b> = <b>{:.2} {}</b>

Exchange Rate: 1 {} = {:.6} {}
Inverse Rate: 1 {} = {:.6} {}

Date: {}
────────────
</span>"#,
        amount,
        from.to_uppercase(),
        result,
        to.to_uppercase(),
        from.to_uppercase(),
        rate,
        to.to_uppercase(),
        to.to_uppercase(),
        1.0 / rate,
        from.to_uppercase(),
        date
    )
}

async fn perform_conversion(amount: f64, from: &str, to: &str) -> Result<(f64, f64, String), Box<dyn std::error::Error>> {
    // If converting from the same currency, return 1:1
    if from.eq_ignore_ascii_case(to) {
        return Ok((amount, 1.0, "Today".to_string()));
    }

    // Use Frankfurter API to get exchange rate
    let url = format!(
        "https://api.frankfurter.dev/v1/latest?base={}&symbols={}",
        from.to_uppercase(),
        to.to_uppercase()
    );

    let mut response = surf::get(&url).await?;
    let status = response.status();

    if !status.is_success() {
        return Err(format!("HTTP Error: {}", status).into());
    }

    let body_text = response.body_string().await?;
    let frankfurter_response: FrankfurterResponse = serde_json::from_str(&body_text)?;

    // Get the exchange rate for the target currency
    if let Some(&rate) = frankfurter_response.rates.get(&to.to_uppercase()) {
        let result = amount * rate;
        Ok((result, rate, frankfurter_response.date))
    } else {
        Err(format!("Currency '{}' not supported or not found", to.to_uppercase()).into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Error: No conversion parameters provided. Usage: sherlock-currency [amount] [from_currency] [to_currency]");
        std::process::exit(1);
    }

    // Join all arguments except the program name
    let input = args[1..].join(" ");

    let (amount, from_currency, to_currency) = match parse_currency_input(&input) {
        Ok(parsed) => parsed,
        Err(error_msg) => {
            eprintln!("Parse Error: {}", error_msg);
            let sherlock_error_response = SherlockPipeResponse {
                title: "Invalid Input Format".to_string(),
                content: format!(
                    r#"<span font_desc="monospace">
─── <b><i>Usage Examples</i></b> ───

• cc 100 usd chf
• cc 50 eur in gbp
• cc 1000 jpy usd
• cc 25.5 cad aud

Supported: 30+ major currencies including:
USD, EUR, GBP, JPY, CHF, CAD, AUD, etc.

Note: Cryptocurrencies not supported by this API
────────────
</span>"#
                ),
                next_content: String::new(),
                actions: vec![],
            };
            println!("{}", serde_json::to_string(&sherlock_error_response).unwrap());
            return Ok(());
        }
    };

    match perform_conversion(amount, &from_currency, &to_currency).await {
        Ok((result, rate, date)) => {
            let content = format_conversion_content(
                amount,
                &from_currency,
                &to_currency,
                result,
                rate,
                &date,
            );

            let action = ApplicationAction::from_conversion(
                amount,
                &from_currency,
                &to_currency,
                result,
                rate,
            );

            let sherlock_response = SherlockPipeResponse {
                title: format!("{:.2} {} → {:.2} {}",
                               amount,
                               from_currency.to_uppercase(),
                               result,
                               to_currency.to_uppercase()
                ),
                content: content.clone(),
                next_content: content,
                actions: vec![action],
            };
            println!("{}", serde_json::to_string(&sherlock_response).unwrap());
        }
        Err(e) => {
            eprintln!("Conversion failed: {}", e);

            let error_content = if e.to_string().contains("Currency") && e.to_string().contains("not supported") {
                format!(
                    r#"<span font_desc="monospace">
─── <b><i>Currency Not Supported</i></b> ───

'{}' or '{}' is not supported by Frankfurter API.

Supported currencies include:
• Major: USD, EUR, GBP, JPY, CHF, CAD, AUD
• European: SEK, NOK, DKK, PLN, CZK, HUF
• Asian: CNY, HKD, SGD, KRW, INR, THB
• Others: BRL, MXN, ZAR, TRY, RUB

Note: Cryptocurrencies are not supported
────────────
</span>"#,
                    from_currency.to_uppercase(),
                    to_currency.to_uppercase()
                )
            } else if e.to_string().contains("HTTP Error") {
                format!(
                    r#"<span font_desc="monospace">
─── <b><i>Network Error</i></b> ───

Failed to connect to Frankfurter API.
Please check your internet connection and try again.

Error: {}
────────────
</span>"#,
                    e
                )
            } else {
                format!(
                    r#"<span font_desc="monospace">
─── <b><i>Conversion Error</i></b> ───

An error occurred during conversion:
{}

Please verify currency codes and try again.
────────────
</span>"#,
                    e
                )
            };

            let sherlock_error_response = SherlockPipeResponse {
                title: "Conversion Failed".to_string(),
                content: error_content,
                next_content: String::new(),
                actions: vec![],
            };
            println!("{}", serde_json::to_string(&sherlock_error_response).unwrap());
        }
    }

    Ok(())
}