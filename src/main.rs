// Attempting to exploit the inefficency's of the market for profit, i.e the prices oscillating
// around a mean / objective price enables reversions to the mean to be targeted.
//TODO: Look into sharpe ration and see the draw down, e.g. in the backtest
//TODO: Could try to implement a mean reversion strategy and momentum together to get the best of
//both worlds.
//TODO: Volitility could be added to strengthen the algorithm, ATR and realised volitality
//TODO: From james roche time series forcasting which enables you to find trends, TC_trending
//indicatorTSF or even a stochaistic.
//TODO: Look into Markov Chains / Hidden Markov Models / Markov regime switching models and
//possible incorporate ideas of stochastics into the decision making process.
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
        live();
    } else if args[1].trim() == "backtest" {
        backtest();
    } else if args[1].trim() == "data" {
        data();
    } else {
        panic!("Please input a valid command to initiate program");
    }
}

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
                let decision: u8 = algorithm::signal_gen(&mut pair);
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
                instant = Instant::now();
            }
        }
    }
}
fn backtest() {
    // Read the values of csv into bid and offer prices. Format for the analysis function.
    // Time, Open, High, Low, Close, Volume
    let b_regression_size = 60 * 24 * 1;
    let mut pair: CurrencyPair = CurrencyPair::new("EUR/USD", b_regression_size);
    let spread: f64 = 0.00001;
    let mut capital: f64 = 1000.0;

    let mut file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open("src/backtest.txt")
        .unwrap();

    let reader = BufReader::new(File::open("src/EURUSD_1M.txt").expect("CANNOT OPEN FILE"));

    let leverage = 10.0;
    let mut wins_counter = 0;
    let mut loss_counter = 0;
    let mut linec = 0;

    for line in reader.lines() {
        linec += 1;
        pair.bid_price = line.unwrap().trim().parse::<f64>().unwrap();
        pair.offer_price = pair.bid_price + spread;
        let decision = algorithm::signal_gen(&mut pair);
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

    let b_regression_size = 60 * 24 * 1;
    let constructer: MessageConstructer =
        MessageConstructer::new(username, password, sender_comp_id, target_comp_id);

    let mut pair: CurrencyPair = CurrencyPair::new("EUR/USD", b_regression_size);
    let mut tls_client_price = TlsClient::new(host, price_port);

    let prices = tls_client_price
        .market_data_request_establishment(&constructer, "EUR/USD", 1)
        .unwrap();

    pair.bid_price = prices[0];
    pair.offer_price = prices[1];

    let mut counter = 0;
    // Main Loop
    let mut instant = Instant::now();
    let mut connected = true;

    let mut file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open("src/eurusd_s.txt")
        .unwrap();

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
                let prices = tls_client_price
                    .market_data_request_establishment(&constructer, "EUR/USD", 1)
                    .unwrap();
                pair.bid_price = prices[0];
                pair.offer_price = prices[1];
                connected = true;
            }
            if instant.elapsed() >= Duration::from_millis(1000) {
                let prices = match &mut tls_client_price.market_data_update() {
                    Err(e) => {
                        if *e == "test_request".to_owned() {
                            tls_client_price.heartbeat(&constructer, "QUOTE");
                            [pair.bid_price, pair.offer_price]
                        } else if *e == "timed_out".to_owned() {
                            [pair.bid_price, pair.offer_price]
                        } else if *e == "heartbeat".to_owned() {
                            [pair.bid_price, pair.offer_price]
                        } else if *e == "connection_aborted".to_owned() {
                            thread::sleep(Duration::from_secs(60));
                            tls_client_price = TlsClient::new(host, price_port);
                            tls_client_price.logon(&constructer, "QUOTE");
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
                file.write(format!("{} {}\n", pair.bid_price, pair.offer_price).as_bytes())
                    .expect("Unable to write file");
            }
            counter += 1;
            if counter >= 15 {
                if tls_client_price.heartbeat(&constructer, "QUOTE")
                    == "connection_aborted".to_owned()
                {
                    thread::sleep(Duration::from_secs(60));
                    tls_client_price = TlsClient::new(host, price_port);
                    tls_client_price.logon(&constructer, "QUOTE");
                    tls_client_price
                        .market_data_request_establishment(&constructer, "EUR/USD", 1)
                        .unwrap();
                }
                counter = 0;
            }
            instant = Instant::now();
        }
    }
}
