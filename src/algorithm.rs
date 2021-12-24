use crate::forex::CurrencyPair;

//TODO: Write co-integration algorithm to find the 'correlation' of the forex pairs, and create a
//portfolio using this. Then

//Every pair combination within the list of pairs will have the cointegration tested.
pub fn pairs_coint(pairs: &Vec<CurrencyPair>) {
    for i in 0..pairs.len() {
        for j in (i + 1)..pairs.len() {
            let value = cointegration(pairs[i], pairs[j]);
            i += 1;
        }
    }
}

// Finds the co-integration value of two forex pairs given a certain time series of data. Used to
// find possible correlation between the two data sets.
fn cointegration(pair1: &CurrencyPair, pair2: &CurrencyPair) {}

pub fn linear_regression(pair: &mut CurrencyPair, regression_size: usize) -> u8 {
    // Length of previous values vector is less than the desired population_size then push offer
    // price.
    if pair.pv.len() < regression_size {
        pair.pv.push(pair.bid_price);
        return 0;
    } else {
        let mut b_x_sum: f64 = 0.00;
        let mut b_x_c: f64 = 0.00;
        let mut b_y_sum: f64 = 0.00;
        let mut b_y_c: f64 = 0.00;

        for (index, value) in pair.pv.iter().enumerate() {
            //kahan summation algorithm
            let b_x_y = index as f64 - b_x_c;
            let b_x_t = b_x_sum + b_x_y;
            b_x_c = (b_x_t - b_x_sum) - b_x_y;
            b_x_sum = b_x_t;

            let b_y_y = value - b_y_c;
            let b_y_t = b_y_sum + b_y_y;
            b_y_c = (b_y_t - b_y_sum) - b_y_y;
            b_y_sum = b_y_t;
        }
        let b_x_mean: f64 = b_x_sum / regression_size as f64;
        let b_y_mean: f64 = b_y_sum / regression_size as f64;

        let mut b_sumx_dif2: f64 = 0.00;
        let mut b_sumy_dif2: f64 = 0.00;

        for (index, value) in pair.pv.iter().enumerate() {
            b_sumx_dif2 += (index as f64 - b_x_mean).powf(2.0);
            b_sumy_dif2 += (value - b_y_mean).powf(2.0);
        }

        let b_s_x: f64 = (1.0 / (regression_size as f64 - 1.0) * b_sumx_dif2).sqrt();
        let b_s_y: f64 = (1.0 / (regression_size as f64 - 1.0) * b_sumy_dif2).sqrt();

        let mut b_r: f64 = 0.00;
        for (index, value) in pair.pv.iter().enumerate() {
            b_r += ((index as f64 - b_x_mean) / b_s_x) * ((value - b_y_mean) / b_s_y);
        }
        b_r = b_r * 1.0 / (regression_size as f64 - 1.0);
        pair.b_b1 = b_r * (b_s_y / b_s_x);
        pair.b_b0 = b_y_mean - pair.b_b1 * b_x_mean;

        let mut s_x_sum: f64 = 0.00;
        let mut s_x_c: f64 = 0.00;
        let mut s_y_sum: f64 = 0.00;
        let mut s_y_c: f64 = 0.00;

        let s_regression_size = regression_size / 2;
        let lower_bound = regression_size - s_regression_size;
        for (index, value) in (&pair.pv[lower_bound..pair.pv.len()]).iter().enumerate() {
            //kahan summation algorithm
            let s_x_y = index as f64 - s_x_c;
            let s_x_t = s_x_sum + s_x_y;
            s_x_c = (s_x_t - s_x_sum) - s_x_y;
            s_x_sum = s_x_t;

            let s_y_y = value - s_y_c;
            let s_y_t = s_y_sum + s_y_y;
            s_y_c = (s_y_t - s_y_sum) - s_y_y;
            s_y_sum = s_y_t;
        }
        let s_x_mean: f64 = s_x_sum / s_regression_size as f64;
        let s_y_mean: f64 = s_y_sum / s_regression_size as f64;

        let mut s_sumx_dif2: f64 = 0.00;
        let mut s_sumy_dif2: f64 = 0.00;

        for (index, value) in (&pair.pv[lower_bound..pair.pv.len()]).iter().enumerate() {
            s_sumx_dif2 += (index as f64 - s_x_mean).powf(2.0);
            s_sumy_dif2 += (value - s_y_mean).powf(2.0);
        }

        let s_s_x: f64 = (1.0 / (s_regression_size as f64 - 1.0) * s_sumx_dif2).sqrt();
        let s_s_y: f64 = (1.0 / (s_regression_size as f64 - 1.0) * s_sumy_dif2).sqrt();

        let mut s_r: f64 = 0.00;
        for (index, value) in (&pair.pv[lower_bound..pair.pv.len()]).iter().enumerate() {
            s_r += ((index as f64 - s_x_mean) / s_s_x) * ((value - s_y_mean) / s_s_y);
        }
        s_r = s_r * 1.0 / (s_regression_size as f64 - 1.0);

        pair.s_b1 = s_r * (s_s_y / s_s_x);
        pair.s_b0 = s_y_mean - pair.s_b1 * s_x_mean;
        /*
                let mut pos: u64 = 0;
                let mut neg: u64 = 0;
                for i in 0..pair.pv.len() / 60 - 1 {
                    if (pair.pv[i * 60] - pair.pv[(i + 1) * 60]) > 0.0 {
                        neg += 1;
                    } else if (pair.pv[i * 60] - pair.pv[(i + 1) * 60]) < 0.0 {
                        pos += 1;
                    }
                }
                pair.trend = (pos as f64) / ((neg as f64) + (pos as f64));

                let stop_loss: f64 = 0.007;
                let profit_limit: f64 = 100.0;

        let mut pos: u64 = 0;
        let mut neg: u64 = 0;
        for i in 0..pair.pv.len() - 1 {
            if (pair.pv[i] - pair.pv[(i + 1)]) > 0.0 {
                neg += 1;
            } else if (pair.pv[i] - pair.pv[(i + 1)]) < 0.0 {
                pos += 1;
            }
        }
        pair.trend = (pos as f64) / ((neg as f64) + (pos as f64));


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
        */
        let stop_loss: f64 = 0.007;
        let profit_limit: f64 = 100.0;

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
        /*

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
        */

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
