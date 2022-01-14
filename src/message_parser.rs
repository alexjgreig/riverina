pub fn parse_fix_message(data: String) -> Result<String, String> {
    let mut input = data.clone();
    let mut fix_msgs: Vec<String> = Vec::new();
    loop {
        // 8 due to the offset between checksum and end of msg.
        let offset = match input.find("\u{0001}10=") {
            Some(x) => x,
            None => break,
        };
        fix_msgs.push(input.drain(..(offset + 8)).collect());
    }

    let mut prices: String = String::new();

    for msg in fix_msgs {
        if msg.contains("35=A") {
            return parse_logon(msg);
        } else if msg.contains("35=5") {
            return parse_logout(msg);
        } else if msg.contains("35=W") {
            if prices.len() != 0 {
                prices = format!("{}\u{0001}{}", prices, parse_market_request(msg).unwrap());
            } else {
                prices = parse_market_request(msg).unwrap();
            }
        } else if msg.contains("35=3") {
            return parse_market_request(msg);
        } else if msg.contains("35=1") {
            return Ok("test_request".to_owned());
        } else if msg.contains("35=0") {
            return Ok("heartbeat".to_owned());
        } else if msg.contains("35=8") || msg.contains("35=j") {
            return parse_order_request(msg);
        } else {
            return Err(format!("Could not parse fix message: {}", msg));
        }
    }
    return Ok(prices);
}
fn parse_logon(msg: String) -> Result<String, String> {
    let tags: Vec<&str> = msg.split("\u{0001}").collect::<Vec<&str>>();

    if msg.contains("35=A") {
        return Ok("Successful connection established with remote host.".to_string());
    } else if msg.contains("35=5") {
        for tag in tags {
            if tag.contains("58=") {
                return Err(format!(
                    "Connection establishment failed, {}",
                    tag.to_string()
                ));
            }
        }
        return Err(format!(
            "Connection establishment failed, FIX Response: {}",
            msg
        ));
    } else {
        return Err("Could not parse FIX Message".to_owned());
    }
}
fn parse_market_request(msg: String) -> Result<String, String> {
    let tags: Vec<&str> = msg.split("\u{0001}").collect::<Vec<&str>>();
    let mut bid_price: String = String::new();
    let mut offer_price: String = String::new();
    let mut symbol: u32 = 0;
    if msg.contains("35=3") {
        for tag in tags {
            if tag.contains("58=") {
                return Err(format!(
                    "There was an error when requesting market data, FIX Error: {}",
                    tag
                ));
            }
        }
        return Err(format!("Could not parse FIX Message, Message: {}", msg));
    } else if msg.contains("35=W") {
        for (index, value) in tags.iter().enumerate() {
            if value.contains("55=") {
                symbol = tags[index][3..].parse::<u32>().unwrap();
            } else if value.contains("269=0") {
                bid_price = tags[index + 1][4..].to_string();
            } else if value.contains("269=1") {
                offer_price = tags[index + 1][4..].to_string();
            }
        }
        return Ok(format!("{},{},{}", symbol, bid_price, offer_price));
    } else {
        return Err(format!("Could not parse FIX Message, Message: {}", msg));
    }
}
fn parse_order_request(msg: String) -> Result<String, String> {
    let tags: Vec<&str> = msg.split("\u{0001}").collect::<Vec<&str>>();
    let mut position_id: String = String::new();
    let mut price_filled: f64 = 0.00;
    // let mut fill_price: String = String::new();
    // let mut quantity_filled: String = String::new();
    println!("{}", msg);
    if msg.contains("35=j") {
        for tag in tags {
            if tag.contains("58=") {
                return Err(format!(
                    "There was an error when requesting market data, FIX Error: {}",
                    tag
                ));
            }
        }
        return Err(format!("{}", msg));
    } else if msg.contains("35=8") {
        if msg.contains("39=8") {
            return Err("order_cancelled".to_string());
        } else {
            for tag in tags {
                if tag.contains("721=") {
                    position_id = tag[4..].to_string();
                }
                if tag.contains("6=") {
                    price_filled = tag[2..].to_string().parse::<f64>().unwrap();
                }
            }
            return Ok(format!("{},{}", position_id, price_filled));
        }
    } else {
        return Err(format!("Could not parse FIX Message, Message: {}", msg));
    }
}

fn parse_logout(msg: String) -> Result<String, String> {
    if msg.contains("35=5") {
        return Ok("Successfully disconnected from remote host.".to_string());
    } else {
        return Err(format!(
            "Connection disconnection failed, FIX Response: {}",
            msg
        ));
    }
}
