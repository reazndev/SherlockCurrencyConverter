# Sherlock Currency Converter

A currency converter extension for the Sherlock application launcher that supports traditional currencies.

## Features

- Convert between 30+ major world currencies
- Real-time exchange rates from Frankfurter API

## Installation


1. **Clone**
   ```bash
   git clone https://github.com/reazndev/SherlockCurrencyConverter.gi
   cd SherlockCurrencyConverter
   ```
   
2. **Build the binary:**
   ```bash
   cargo build --release
   ```

3. **Copy the binary to Sherlock scripts directory:**
   ```bash
   cp target/release/sherlock-currency ~/.config/sherlock/scripts/
   chmod +x ~/.config/sherlock/scripts/sherlock-currency
   ```

4. **Add the configuration to your Sherlock fallback.json:**
   Add the following entry to your `~/.config/sherlock/fallback.json` file:
   ```json
   {
       "name": "Currency Converter",
       "alias": "cc",
       "type": "bulk_text",
       "async": true,
       "args": {
           "icon": "preferences-system",
           "exec": "~/.config/sherlock/scripts/sherlock-currency",
           "exec-args": "{keyword}"
       },
       "priority": 0,
       "shortcut": false
   }
   ```

## Usage

Use the alias `cc` followed by your conversion parameters:

note: you can change the alias in the fallback.json to your liking.

### Syntax Options:
`cc [amount] [from_currency] [to_currency]`

`cc [amount] [from_currency] in [to_currency]`

### Examples:
```
cc 100 usd chf
cc 50 eur in gbp
cc 1000 jpy usd
cc 25.5 cad aud
cc 1 chf usd
cc 42.50 gbp jpy
```

### Supported Currencies:
- **Major Currencies:** USD, EUR, GBP, JPY, CHF, CAD, AUD, NZD
- **European:** SEK, NOK, DKK, PLN, CZK, HUF, BGN, HRK, RON
- **Asian:** CNY, HKD, SGD, KRW, INR, THB, MYR, PHP, IDR
- **Others:** BRL, MXN, ZAR, TRY, RUB, ILS
- **All ISO currency codes** supported by Frankfurter API

Note: Cryptocurrencies are not supported by the Frankfurter API
