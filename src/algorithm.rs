use crate::forex::CurrencyPair;

//TODO: Write co-integration algorithm to find the 'correlation' of the forex pairs, and create a
//portfolio using this. Then

// Finds the co-integration value of two forex pairs given a certain time series of data. Used to
// find possible correlation between the two data sets.
fn cointegration(pair1: &CurrencyPair, pair2: &CurrencyPair) -> f64 {
    return 0.00;
}

//Every pair combination within the list of pairs will have the cointegration tested.
pub fn pairs_coint(pairs: &Vec<CurrencyPair>) {
    for i in 0..pairs.len() {
        for j in (i + 1)..pairs.len() {
            let value = cointegration(&pairs[i], &pairs[j]);
        }
    }
}

pub fn linear_regression(values: Vec<f64>, regression_size: usize) -> (f64, f64) {
    // Length of previous values vector is less than the desired population_size then push offer
    // price.
    let mut x_sum: f64 = 0.00;
    let mut x_c: f64 = 0.00;
    let mut y_sum: f64 = 0.00;
    let mut y_c: f64 = 0.00;

    for (index, value) in values.iter().enumerate() {
        //kahan summation algorithm
        let x_y = index as f64 - x_c;
        let x_t = x_sum + x_y;
        x_c = (x_t - x_sum) - x_y;
        x_sum = x_t;

        let y_y = value - y_c;
        let y_t = y_sum + y_y;
        y_c = (y_t - y_sum) - y_y;
        y_sum = y_t;
    }
    let x_mean: f64 = x_sum / regression_size as f64;
    let y_mean: f64 = y_sum / regression_size as f64;

    let mut sumx_dif2: f64 = 0.00;
    let mut sumy_dif2: f64 = 0.00;

    for (index, value) in values.iter().enumerate() {
        sumx_dif2 += (index as f64 - x_mean).powf(2.0);
        sumy_dif2 += (value - y_mean).powf(2.0);
    }

    let s_x: f64 = (1.0 / (regression_size as f64 - 1.0) * sumx_dif2).sqrt();
    let s_y: f64 = (1.0 / (regression_size as f64 - 1.0) * sumy_dif2).sqrt();

    let mut r: f64 = 0.00;
    for (index, value) in values.iter().enumerate() {
        r += ((index as f64 - x_mean) / s_x) * ((value - y_mean) / s_y);
    }
    r = r * 1.0 / (regression_size as f64 - 1.0);
    let b1 = r * (s_y / s_x);
    let b0 = y_mean - b1 * x_mean;

    return (b0, b1);
}

pub fn pair_linear_regression(pair: &mut CurrencyPair, regression_size: usize) -> u8 {
    // Length of previous values vector is less than the desired population_size then push offer
    // price.
    if pair.pv.len() < regression_size {
        pair.pv.push(pair.bid_price);
        return 0;
    } else {
        let base = linear_regression(pair.pv.clone(), regression_size);
        let signal = linear_regression(pair.pv.clone(), regression_size / 2);

        pair.b_b0 = base.0;
        pair.b_b1 = base.1;
        pair.s_b0 = signal.0;
        pair.s_b1 = signal.1;

        let stop_loss: f64 = 0.007;
        let profit_limit: f64 = 100.0;

        if pair.owned == true
            && pair.direction == true
            && pair.bid_price - stop_loss > pair.stop_loss
        {
            pair.stop_loss = pair.bid_price - stop_loss;
        } else if pair.owned == true
            && pair.direction == false
            && pair.offer_price + stop_loss < pair.stop_loss
        {
            pair.stop_loss = pair.offer_price + stop_loss;
        }

        if pair.direction == false && pair.owned == true && pair.offer_price > pair.stop_loss {
            // Buy from stop loss being hit
            pair.pv.drain(0..pair.pv.len() / 2);
            pair.pv.push(pair.bid_price);
            pair.owned = false;
            pair.buy_price = pair.offer_price;
            return 1;
        } else if pair.direction == true && pair.owned == true && pair.bid_price < pair.stop_loss {
            // sell from stop loss being hit
            pair.pv.drain(0..pair.pv.len() / 2);
            pair.pv.push(pair.bid_price);

            pair.owned = false;
            pair.sell_price = pair.bid_price;

            return 2;
        }

        if pair.direction == false && pair.owned == true && pair.offer_price < pair.profit_limit {
            //Buy from profit limit being hit
            pair.pv.drain(0..pair.pv.len());
            pair.pv.push(pair.bid_price);
            pair.owned = false;
            pair.buy_price = pair.offer_price;
            return 1;
        } else if pair.direction == true && pair.owned == true && pair.bid_price > pair.profit_limit
        {
            //Buy from profit limit being hit
            pair.pv.drain(0..pair.pv.len());
            pair.pv.push(pair.bid_price);
            pair.owned = false;
            pair.sell_price = pair.bid_price;
            return 2;
        }

        if pair.s_b1 < pair.b_b1 && pair.s_b0 < pair.b_b0 {
            if pair.owned == false {
                // Buy
                pair.owned = true;
                pair.direction = true;
                pair.buy_price = pair.offer_price;
                pair.stop_loss = pair.bid_price - stop_loss;
                pair.profit_limit = pair.bid_price + profit_limit;

                pair.pv.remove(0);
                pair.pv.push(pair.bid_price);

                return 1;
            } else if pair.owned == true && pair.direction == false {
                pair.owned = true;
                pair.direction = true;
                pair.buy_price = pair.offer_price;
                pair.stop_loss = pair.bid_price - stop_loss;
                pair.profit_limit = pair.bid_price + profit_limit;

                pair.pv.remove(0);
                pair.pv.push(pair.bid_price);

                return 3;
            } else {
                pair.pv.remove(0);
                pair.pv.push(pair.bid_price);
                return 0;
            }
        } else if pair.s_b1 > pair.b_b1 && pair.s_b0 > pair.b_b0 {
            if pair.owned == false {
                // Sell
                pair.owned = true;
                pair.direction = false;
                pair.sell_price = pair.bid_price;
                pair.stop_loss = pair.offer_price + stop_loss;
                pair.profit_limit = pair.offer_price - profit_limit;

                pair.pv.remove(0);
                pair.pv.push(pair.bid_price);

                return 2;
            } else if pair.owned == true && pair.direction == true {
                pair.owned = true;
                pair.direction = false;
                pair.sell_price = pair.bid_price;
                pair.stop_loss = pair.offer_price + stop_loss;
                pair.profit_limit = pair.offer_price - profit_limit;

                pair.pv.remove(0);
                pair.pv.push(pair.bid_price);

                return 4;
            } else {
                pair.pv.remove(0);
                pair.pv.push(pair.bid_price);
                return 0;
            }
        } else {
            pair.pv.remove(0);
            pair.pv.push(pair.bid_price);

            return 0;
        }
    }
}
