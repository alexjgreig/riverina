//TODO: Make the defaults None instead of 0 as this could cause random buy and sells at the start
//as the program thinks that the price is zero at the beginning that way the program waits until a
//previous price is not None.
use std::fmt;
pub struct CurrencyPair<'a> {
    pub bid_price: f64,
    pub offer_price: f64,
    pub buy_price: f64,
    pub sell_price: f64,
    pub owned: bool,
    //BUY = TRUE, SELL = FALSE
    pub direction: bool,
    pub b_b1: f64,
    pub b_b0: f64,
    pub s_b1: f64,
    pub s_b0: f64,
    pub pv: Vec<f64>,
    pub stop_loss: f64,
    pub profit_limit: f64,
    pub id: u32,
    pub name: &'a str,
}
impl<'a> fmt::Display for CurrencyPair<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Name: {}\nbid price: {}\noffer price: {}\nbuy price: {}\nsell price: {}\nowned: {}\ndirection: {}\nb_b1: {}\nb_b0: {}\ns_b1: {}\ns_b0: {}\npv length (hour): {}\n\n",
            self.name, self.bid_price, self.offer_price, self.buy_price, self.sell_price, self.owned, self.direction, self.b_b1, self.b_b0, self.s_b1, self.s_b0, (self.pv.len() as f64 / 60.0 / 60.0)
        )
    }
}
impl<'a> CurrencyPair<'a> {
    pub fn new(name: &str, id: u32) -> CurrencyPair {
        CurrencyPair {
            bid_price: 0.00,
            offer_price: 0.00,
            buy_price: 0.00,
            sell_price: 0.00,
            owned: false,
            direction: true,
            b_b1: 0.00,
            b_b0: 0.00,
            s_b1: 0.00,
            s_b0: 0.00,
            pv: Vec::new(),
            stop_loss: 0.00,
            profit_limit: 0.00,
            id: id,
            name: name,
        }
    }
}
