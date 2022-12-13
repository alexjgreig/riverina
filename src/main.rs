// TODO: Look at the ideas found in research articles for statistical arbitrage, pairs trading,
// cointegration.
// Attempting to exploit the inefficency's of the market for profit, i.e the prices oscillating
// around a mean / objective price enables reversions to the mean to be targeted.
//TODO: Look into sharpe ration and see the draw down, e.g. in the backtest
//TODO: Do cointegration tests when the market is closed and update the cointegrating pairs list.
//Continuously store past prices.
//TODO: Add error handling for networking errors, there is bound to be errors in the process of
//routing packets therefore must be able to reinstantiate itself and run again.

mod algorithm;
mod forex;
mod message_constructer;
mod message_parser;
mod network;
use chrono::{Datelike, Timelike, Utc, Weekday};
use forex::CurrencyPair;
use message_constructer::MessageConstructer;
use network::TlsClient;
use rpassword;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args[1].trim() == "live" {
        // live();
    } else if args[1].trim() == "backtest" {
        backtest();
    } else if args[1].trim() == "data" {
        data();
    } else {
        panic!("Please input a valid command to initiate program");
    }
}
/*
fn live() {
    let args: Vec<String> = env::args().collect();
    let host: &str = "h35.p.ctrader.com";
    let price_port: u16 = 5211;
    let trade_port: u16 = 5212;
    let username: String = "3528709".to_string();
    let password: String =
        rpassword::read_password_from_tty(Some("Authentication Password: ")).unwrap();
    let sender_comp_id: String = "pepperstone.3528709".to_string();
    let target_comp_id: String = "cServer".to_string();

    let constructer: MessageConstructer =
        MessageConstructer::new(username, password, sender_comp_id, target_comp_id);

    //Seconds || Minutes || Hours || Days
    let b_regression_size = 60 * 60 * 24 * 1;
    let mut pair: CurrencyPair = CurrencyPair::new("EUR/USD", b_regression_size);

    // Establish a connection with server
    let mut tls_client_price = TlsClient::new(host, price_port);
    let mut tls_client_trade = TlsClient::new(host, trade_port);

    tls_client_price.logon(&constructer, "QUOTE");
    tls_client_trade.logon(&constructer, "TRADE");

    let prices = tls_client_price
        .market_data_request_establishment(&constructer, "EUR/USD", 1)
        .unwrap();

    pair.bid_price = prices[0];
    pair.offer_price = prices[1];

    let mut counter = 0;
    // Main Loop
    let mut instant = Instant::now();
    let mut capital: f64 = args[2].trim().parse::<f64>().unwrap();
    let mut connected = true;
    let mut position_id: String = String::new();

    loop {
        if (Utc::now().weekday() == Weekday::Fri
            && Utc::now().hour() == 21
            && Utc::now().minute() >= 55)
            || (Utc::now().weekday() == Weekday::Fri && Utc::now().hour() > 21)
            || (Utc::now().weekday() == Weekday::Sat)
            || (Utc::now().weekday() == Weekday::Sun && Utc::now().hour() < 22)
        {
            if connected == true {
                tls_client_price.logout(&constructer, "QUOTE");
                tls_client_trade.logout(&constructer, "TRADE");
                connected = false;
            }
            println!("\n\n\n\n\nMarket Closed, waiting for open\n\n\n\n\n");
            thread::sleep(Duration::from_secs(60));
        } else {
            if connected == false {
                tls_client_price = TlsClient::new(host, price_port);
                tls_client_trade = TlsClient::new(host, trade_port);
                tls_client_price.logon(&constructer, "QUOTE");
                tls_client_trade.logon(&constructer, "TRADE");
                let prices = tls_client_price
                    .market_data_request_establishment(&constructer, "EUR/USD", 1)
                    .unwrap();
                pair.bid_price = prices[0];
                pair.offer_price = prices[1];
                connected = true;
            }
            if instant.elapsed() >= Duration::from_millis(1000) {
                instant = Instant::now();
                let prices = match &mut tls_client_price.market_data_update() {
                    Err(e) => {
                        if *e == "test_request".to_owned() {
                            tls_client_price.heartbeat(&constructer, "QUOTE");
                            tls_client_trade.heartbeat(&constructer, "TRADE");
                            [pair.bid_price, pair.offer_price]
                        } else if *e == "timed_out".to_owned() {
                            [pair.bid_price, pair.offer_price]
                        } else if *e == "heartbeat".to_owned() {
                            [pair.bid_price, pair.offer_price]
                        } else if *e == "connection_aborted".to_owned() {
                            thread::sleep(Duration::from_secs(60));
                            tls_client_price = TlsClient::new(host, price_port);
                            tls_client_trade = TlsClient::new(host, trade_port);
                            tls_client_price.logon(&constructer, "QUOTE");
                            tls_client_trade.logon(&constructer, "TRADE");
                            tls_client_price
                                .market_data_request_establishment(&constructer, "EUR/USD", 1)
                                .unwrap();
                            [pair.bid_price, pair.offer_price]
                        } else if *e == "0_bytes_read".to_owned() {
                            [pair.bid_price, pair.offer_price]
                        } else {
                            panic!("{}", e);
                        }
                    }
                    Ok(x) => [x[0], x[1]],
                };
                pair.bid_price = prices[0];
                pair.offer_price = prices[1];

                let leverage = 10.0;
                let order_quantity = ((capital / 100.0).floor() * 100.0 * leverage) as u64;
                // Buying and Selling Based on Signals Given by Algorithm
                // 1: Buy, When currently not in a position
                // 2: Sell, when currently not in a postition
                // 3: Double Buy, to exit a short position and enter a long position
                // 4: Double sell, to exit a long position and enter a short position
                let decision: u8 = algorithm::linear_regression(&mut pair);
                if decision == 1 {
                    match tls_client_trade.single_order(&constructer, 1, 1, order_quantity, None) {
                        Err(e) => panic!("{}", e),
                        Ok(id) => position_id = id,
                    }
                    if pair.owned == false {
                        capital += (pair.sell_price - pair.buy_price) * 1.0 / pair.buy_price
                            * order_quantity as f64
                            - order_quantity as f64 * 0.000035 * 2.0;
                    }
                } else if decision == 2 {
                    match tls_client_trade.single_order(&constructer, 1, 2, order_quantity, None) {
                        Err(e) => panic!("{}", e),
                        Ok(id) => position_id = id,
                    }
                    if pair.owned == false {
                        capital += (pair.sell_price - pair.buy_price) * 1.0 / pair.sell_price
                            * order_quantity as f64
                            - order_quantity as f64 * 0.000035 * 2.0;
                    }
                } else if decision == 3 {
                    match tls_client_trade.single_order(
                        &constructer,
                        1,
                        1,
                        order_quantity,
                        Some(position_id),
                    ) {
                        Err(e) => panic!("{}", e),
                        _ => (),
                    }
                    capital += (pair.sell_price - pair.buy_price) * 1.0 / pair.buy_price
                        * order_quantity as f64
                        - order_quantity as f64 * 0.000035 * 2.0;
                    let order_quantity = ((capital / 100.0).floor() * 100.0 * leverage) as u64;
                    match tls_client_trade.single_order(&constructer, 1, 1, order_quantity, None) {
                        Err(e) => panic!("{}", e),
                        Ok(id) => position_id = id,
                    }
                } else if decision == 4 {
                    match tls_client_trade.single_order(
                        &constructer,
                        1,
                        2,
                        order_quantity,
                        Some(position_id),
                    ) {
                        Err(e) => panic!("{}", e),
                        _ => (),
                    }
                    capital += (pair.sell_price - pair.buy_price) * 1.0 / pair.sell_price
                        * order_quantity as f64
                        - order_quantity as f64 * 0.000035 * 2.0;
                    let order_quantity = ((capital / 100.0).floor() * 100.0 * leverage) as u64;
                    match tls_client_trade.single_order(&constructer, 1, 2, order_quantity, None) {
                        Err(e) => panic!("{}", e),
                        Ok(id) => position_id = id,
                    }
                }
                println!("{}\n{}", capital, &pair);
                counter += 1;
                if counter == 15 {
                    if tls_client_price.heartbeat(&constructer, "QUOTE")
                        == "connection_aborted".to_owned()
                    {
                        thread::sleep(Duration::from_secs(60));
                        tls_client_price = TlsClient::new(host, price_port);
                        tls_client_trade = TlsClient::new(host, trade_port);
                        tls_client_price.logon(&constructer, "QUOTE");
                        tls_client_trade.logon(&constructer, "TRADE");
                        tls_client_price
                            .market_data_request_establishment(&constructer, "EUR/USD", 1)
                            .unwrap();
                    }
                    if tls_client_trade.heartbeat(&constructer, "TRADE")
                        == "connection_aborted".to_owned()
                    {
                        thread::sleep(Duration::from_secs(60));
                        tls_client_price = TlsClient::new(host, price_port);
                        tls_client_trade = TlsClient::new(host, trade_port);
                        tls_client_price.logon(&constructer, "QUOTE");
                        tls_client_trade.logon(&constructer, "TRADE");
                        tls_client_price
                            .market_data_request_establishment(&constructer, "EUR/USD", 1)
                            .unwrap();
                    }
                    counter = 0;
                }
            }
        }
    }
}
*/
fn backtest() {
    // Read the values of csv into bid and offer prices. Format for the analysis function.
    // Time, Open, High, Low, Close, Volume
    let regression_size = 60 * 60 * 6;
    let mut pair: CurrencyPair = CurrencyPair::new("EUR/USD", 1);
    let spread: f64 = 0.00001;
    let mut capital: f64 = 1000.0;

    let mut file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open("src/backtest.txt")
        .unwrap();

    let reader = BufReader::new(File::open("hist/eurusd_s.txt").expect("CANNOT OPEN FILE"));

    let leverage = 10.0;
    let mut wins_counter = 0;
    let mut loss_counter = 0;
    let mut linec = 0;

    for line in reader.lines() {
        linec += 1;
        let prices: Vec<String> = line
            .unwrap()
            .trim()
            .split(" ")
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        pair.bid_price = prices[0].parse::<f64>().unwrap();
        pair.offer_price = prices[1].parse::<f64>().unwrap();

        let decision = algorithm::pair_linear_regression(&mut pair, regression_size);
        let order_quantity = (((capital - 0.000001) / 100.0).floor() * 100.0 * leverage) as u64;
        if decision == 1 {
            if pair.owned == false {
                capital += (pair.sell_price - pair.buy_price) * 1.0 / pair.buy_price
                    * order_quantity as f64
                    - order_quantity as f64 * 0.000035 * 2.0;
                if (pair.sell_price - pair.buy_price) * 1.0 / pair.buy_price * order_quantity as f64
                    - order_quantity as f64 * 0.000035 * 2.0
                    > 0.00
                {
                    wins_counter += 1;
                } else {
                    loss_counter += 1;
                }

                file.write(
                    format!(
                        "\n started at {} ended at {} in sell direction\n capital: {}",
                        pair.sell_price, pair.buy_price, capital
                    )
                    .as_bytes(),
                )
                .expect("unable to write file");
            }
        } else if decision == 2 {
            if pair.owned == false {
                capital += (pair.sell_price - pair.buy_price) * 1.0 / pair.sell_price
                    * order_quantity as f64
                    - order_quantity as f64 * 0.000035 * 2.0;
                if (pair.sell_price - pair.buy_price) * 1.0 / pair.buy_price * order_quantity as f64
                    - order_quantity as f64 * 0.000035 * 2.0
                    > 0.00
                {
                    wins_counter += 1;
                } else {
                    loss_counter += 1;
                }
                file.write(
                    format!(
                        "\n started at {} ended at {} in buy direction\n capital: {}",
                        pair.buy_price, pair.sell_price, capital
                    )
                    .as_bytes(),
                )
                .expect("Unable to write file");
            }
        } else if decision == 3 {
            capital += (pair.sell_price - pair.buy_price) * 1.0 / pair.buy_price
                * order_quantity as f64
                - order_quantity as f64 * 0.000035 * 2.0;
            if (pair.sell_price - pair.buy_price) * 1.0 / pair.buy_price * order_quantity as f64
                - order_quantity as f64 * 0.000035 * 2.0
                > 0.00
            {
                wins_counter += 1;
            } else {
                loss_counter += 1;
            }
            file.write(
                format!(
                    "\n started at {} ended at {} in sell direction\n capital: {}",
                    pair.sell_price, pair.buy_price, capital
                )
                .as_bytes(),
            )
            .expect("unable to write file");
        } else if decision == 4 {
            capital += (pair.sell_price - pair.buy_price) * 1.0 / pair.sell_price
                * order_quantity as f64
                - order_quantity as f64 * 0.000035 * 2.0;
            if (pair.sell_price - pair.buy_price) * 1.0 / pair.buy_price * order_quantity as f64
                - order_quantity as f64 * 0.000035 * 2.0
                > 0.00
            {
                wins_counter += 1;
            } else {
                loss_counter += 1;
            }
            file.write(
                format!(
                    "\n started at {} ended at {} in buy direction\n capital: {}",
                    pair.buy_price, pair.sell_price, capital
                )
                .as_bytes(),
            )
            .expect("Unable to write file");
        }
        if capital < 100.0 {
            println!("{}", linec);
            break;
        }
    }
    println!("WINS: {}, LOSS: {}", wins_counter, loss_counter);
}

fn data() {
    let host: &str = "h35.p.ctrader.com";
    let price_port: u16 = 5211;
    let username: String = "3528709".to_string();
    let password: String =
        rpassword::read_password_from_tty(Some("Authentication Password: ")).unwrap();
    let sender_comp_id: String = "pepperstone.3528709".to_string();
    let target_comp_id: String = "cServer".to_string();

    let constructer: MessageConstructer =
        MessageConstructer::new(username, password, sender_comp_id, target_comp_id);

    let mut pairs: Vec<CurrencyPair> = Vec::new();

    //MAJORS
    pairs.push(CurrencyPair::new("EUR/USD", 1));
    pairs.push(CurrencyPair::new("GBP/USD", 2));
    pairs.push(CurrencyPair::new("USD/JPY", 4));
    pairs.push(CurrencyPair::new("AUD/USD", 5));
    pairs.push(CurrencyPair::new("USD/CHF", 6));
    pairs.push(CurrencyPair::new("USD/CAD", 8));
    //MINORS
    pairs.push(CurrencyPair::new("AUD/CAD", 18));
    pairs.push(CurrencyPair::new("AUD/CHF", 23));
    pairs.push(CurrencyPair::new("AUD/NZD", 20));
    pairs.push(CurrencyPair::new("AUD/SGD", 43));
    pairs.push(CurrencyPair::new("EUR/AUD", 14));
    pairs.push(CurrencyPair::new("EUR/CHF", 10));
    pairs.push(CurrencyPair::new("EUR/GBP", 9));
    pairs.push(CurrencyPair::new("GBP/AUD", 16));
    pairs.push(CurrencyPair::new("GBP/CHF", 40));
    pairs.push(CurrencyPair::new("NZD/USD", 12));
    //CROSSES
    pairs.push(CurrencyPair::new("EUR/JPY", 3));
    pairs.push(CurrencyPair::new("GBP/JPY", 7));
    pairs.push(CurrencyPair::new("AUD/JPY", 11));
    pairs.push(CurrencyPair::new("CHF/JPY", 13));
    pairs.push(CurrencyPair::new("CAD/JPY", 15));
    pairs.push(CurrencyPair::new("EUR/CAD", 17));
    pairs.push(CurrencyPair::new("GBP/CAD", 19));
    pairs.push(CurrencyPair::new("NZD/JPY", 21));
    pairs.push(CurrencyPair::new("GBP/NZD", 25));
    pairs.push(CurrencyPair::new("EUR/NZD", 26));
    pairs.push(CurrencyPair::new("CAD/CHF", 27));
    pairs.push(CurrencyPair::new("NZD/CAD", 30));
    pairs.push(CurrencyPair::new("NZD/CHF", 39));
    pairs.push(CurrencyPair::new("SGD/JPY", 58));
    //EXOTICS - They instruments close from the market randomly it seems

    pairs.push(CurrencyPair::new("USD/NOK", 22));
    pairs.push(CurrencyPair::new("USD/MXN", 24));
    pairs.push(CurrencyPair::new("USD/SGD", 28));
    pairs.push(CurrencyPair::new("USD/SEK", 29));
    pairs.push(CurrencyPair::new("EUR/SEK", 31));
    pairs.push(CurrencyPair::new("GBP/SGD", 32));
    pairs.push(CurrencyPair::new("EUR/NOK", 33));
    pairs.push(CurrencyPair::new("EUR/HUF", 34));
    pairs.push(CurrencyPair::new("USD/PLN", 35));
    pairs.push(CurrencyPair::new("EUR/CZK", 45));
    pairs.push(CurrencyPair::new("EUR/PLN", 48));
    pairs.push(CurrencyPair::new("EUR/SGD", 49));
    pairs.push(CurrencyPair::new("EUR/ZAR", 51));
    pairs.push(CurrencyPair::new("GBP/SEK", 53));
    pairs.push(CurrencyPair::new("NOK/JPY", 55));
    pairs.push(CurrencyPair::new("NOK/SEK", 56));
    pairs.push(CurrencyPair::new("SEK/JPY", 57));
    pairs.push(CurrencyPair::new("USD/CZK", 59));
    pairs.push(CurrencyPair::new("USD/HKD", 60));
    pairs.push(CurrencyPair::new("USD/ZAR", 62));
    pairs.push(CurrencyPair::new("USD/HUF", 68));
    pairs.push(CurrencyPair::new("USD/RUB", 70));
    pairs.push(CurrencyPair::new("USD/CNH", 71));
    pairs.push(CurrencyPair::new("ZAR/JPY", 10027));

    let mut tls_client_price = TlsClient::new(host, price_port);

    println!("{}", tls_client_price.logon(&constructer, "QUOTE"));

    for pair in pairs.iter_mut() {
        tls_client_price
            .market_data_request_establishment(&constructer, pair.name, pair.id)
            .unwrap();
    }

    let mut counter = 0;
    // Main Loop
    let mut instant = Instant::now();
    let mut connected = true;

    let mut file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open("hist_s.txt")
        .unwrap();

    let mut prev_partial = String::new();

    loop {
        if (Utc::now().weekday() == Weekday::Fri
            && Utc::now().hour() == 21
            && Utc::now().minute() >= 55)
            || (Utc::now().weekday() == Weekday::Fri && Utc::now().hour() > 21)
            || (Utc::now().weekday() == Weekday::Sat)
            || (Utc::now().weekday() == Weekday::Sun && Utc::now().hour() < 22)
        {
            if connected == true {
                tls_client_price.logout(&constructer, "QUOTE");
                connected = false;
            }
            println!("\n\n\n\n\nMarket Closed, waiting for open\n\n\n\n\n");
            thread::sleep(Duration::from_secs(60));
        } else {
            if connected == false {
                tls_client_price = TlsClient::new(host, price_port);
                tls_client_price.logon(&constructer, "QUOTE");
                for pair in pairs.iter_mut() {
                    tls_client_price
                        .market_data_request_establishment(&constructer, pair.name, pair.id)
                        .unwrap();
                }
                connected = true;
                println!("Market Started, Connected to Exchange");
            }
            if instant.elapsed() < Duration::from_millis(1000) {
                match &mut tls_client_price.market_data_update(prev_partial.clone()) {
                    Err(e) => {
                        if *e == "test_request".to_owned() {
                            tls_client_price.heartbeat(&constructer, "QUOTE");
                        } else if *e == "timed_out".to_owned() {
                            //Reading operation timed out, no response within timeout property
                        } else if *e == "heartbeat".to_owned() {
                            println!("Received heartbeat from server");
                        } else if *e == "connection_aborted".to_owned() {
                            thread::sleep(Duration::from_secs(60));
                            tls_client_price = TlsClient::new(host, price_port);
                            tls_client_price.logon(&constructer, "QUOTE");
                            for pair in pairs.iter_mut() {
                                tls_client_price
                                    .market_data_request_establishment(
                                        &constructer,
                                        pair.name,
                                        pair.id,
                                    )
                                    .unwrap();
                            }
                        } else if *e == "0_bytes_read".to_owned() {
                            //0 bytes in the buffer
                        } else {
                            panic!("{}", e);
                        }
                    }
                    Ok(instruments) => {
                        prev_partial = instruments.1.clone();
                        for instra in instruments.0.clone() {
                            for pair in pairs.iter_mut() {
                                if pair.id == instra.0 {
                                    pair.bid_price = instra.1;
                                    pair.offer_price = instra.2;
                                }
                            }
                        }
                    }
                }
            } else {
                instant = Instant::now();

                counter += 1;
                if counter >= 15 {
                    if tls_client_price.heartbeat(&constructer, "QUOTE")
                        == "connection_aborted".to_owned()
                    {
                        thread::sleep(Duration::from_secs(60));
                        tls_client_price = TlsClient::new(host, price_port);
                        tls_client_price.logon(&constructer, "QUOTE");
                        for pair in pairs.iter_mut() {
                            tls_client_price
                                .market_data_request_establishment(&constructer, pair.name, pair.id)
                                .unwrap();
                        }
                    }
                    counter = 0;
                }

                for pair in pairs.iter() {
                    file.write(
                        format!("{} {} {}\n", pair.name, pair.bid_price, pair.offer_price)
                            .as_bytes(),
                    )
                    .expect("Unable to write file");
                }
            }
        }
    }
}
