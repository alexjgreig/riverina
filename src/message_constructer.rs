use chrono::prelude::*;
pub enum SessionMessageType {
    Logon,
    Heartbeat,
    Logout,
    // Resend,
    // Reject,
    // SequenceReset,
}
pub enum ApplicationMessageType {
    MarketDataRequest,
    NewOrderSingle,
    // MarketDataIncrementalRefresh,
    // ExecutionReport,
    // BusinessMessageReject,
}

pub struct MessageConstructer {
    pub username: String,
    pub password: String,
    pub sender_comp_id: String,
    pub target_comp_id: String,
}

impl MessageConstructer {
    pub fn new(
        username: String,
        password: String,
        sender_comp_id: String,
        target_comp_id: String,
    ) -> MessageConstructer {
        MessageConstructer {
            username,
            password,
            sender_comp_id,
            target_comp_id,
        }
    }

    pub fn logon(
        &self,
        qualifier: &str,
        message_sequence_number: u64,
        heart_beat_seconds: u32,
        reset_seqnum: bool,
    ) -> String {
        let mut body = String::new();

        //Defines a message encryption scheme.Currently, only transportlevel security
        //is supported.Valid value is "0"(zero) = NONE_OTHER (encryption is not used).
        body.push_str("98=0|");

        //Heartbeat interval in seconds.
        //Value is set in the 'config.properties' file (client side) as 'SERVER.POLLING.INTERVAL'.
        //30 seconds is default interval value. If HeartBtInt is set to 0, no heart beat message
        //is required.
        body = format!("{}108={}|", body, heart_beat_seconds);

        // All sides of FIX session should have
        //sequence numbers reset. Valid value
        //is "Y" = Yes(reset).
        if reset_seqnum {
            body.push_str("141=Y|");
        }

        //The numeric User ID. User is linked to SenderCompID (#49) value (the

        //user’s organization).
        body = format!("{}553={}|", body, self.username);

        //User Password
        body = format!("{}554={}|", body, self.password);

        let header = construct_header(
            qualifier,
            session_message_code(SessionMessageType::Logon),
            message_sequence_number,
            &body,
            &self,
        );
        let header_body = format!("{}{}", &header, &body);
        let trailer = construct_trailer(&header_body);
        let header_message_trailer = format!("{}{}{}", header, body, trailer);
        return header_message_trailer.replace("|", "\u{0001}");
    }
    pub fn heartbeat(&self, qualifier: &str, message_sequence_number: u64) -> String {
        let header = construct_header(
            qualifier,
            session_message_code(SessionMessageType::Heartbeat),
            message_sequence_number,
            &String::new(),
            &self,
        );
        let trailer = construct_trailer(&header);
        let header_message_trailer = format!("{}{}", &header, &trailer);
        return header_message_trailer.replace("|", "\u{0001}");
    }

    pub fn logout(&self, qualifier: &str, message_sequence_number: u64) -> String {
        let header = construct_header(
            qualifier,
            session_message_code(SessionMessageType::Logout),
            message_sequence_number,
            &String::new(),
            &self,
        );
        let trailer = construct_trailer(&header);
        let header_message_trailer = format!("{}{}", &header, &trailer);
        return header_message_trailer.replace("|", "\u{0001}");
    }

    pub fn market_data_request(
        &self,
        qualifier: &str,
        message_sequence_number: u64,
        market_data_request_id: &str,
        subscription_request_type: u32,
        market_depth: u32,
        no_related_symbol: u32,
        symbol: u32,
    ) -> String {
        let mut body = String::new();

        // Unique quote request id. New ID for a new subscription, same one as previously used for
        // subscription removal.
        body = format!("{}262={}|", body, market_data_request_id);
        // 1 = Snapshot plus updates (subscribe) 2 = Disable previous snapshot plus update request
        // (unsubscribe)
        body = format!("{}263={}|", body, subscription_request_type);
        // Full book will be provided, 0 = Depth subscription, 1 = Spot subscription
        body = format!("{}264={}|", body, market_depth);
        //Only Incremental refresh is supported.
        body = format!("{}265=1|", body);
        // Always set to 2 (both bid and ask will be sent).
        body = format!("{}267=2|", body);
        // contains a list of all types of Market Data Entries the requester wants to receive.
        body = format!("{}269=0|269=1|", body);
        //Number of symbols requested.
        body = format!("{}146={}|", body, no_related_symbol);
        // Symbol of the specific instrument, provided in the ctrader application.
        body = format!("{}55={}|", body, symbol);

        let header = construct_header(
            qualifier,
            application_message_code(ApplicationMessageType::MarketDataRequest),
            message_sequence_number,
            &body,
            &self,
        );
        let header_body = format!("{}{}", &header, &body);
        let trailer = construct_trailer(&header_body);
        let header_message_trailer = format!("{}{}{}", header, body, trailer);
        return header_message_trailer.replace("|", "\u{0001}");
    }

    pub fn single_order_request(
        &self,
        qualifier: &str,
        message_sequence_number: u64,
        order_id: u32,
        symbol: u32,
        side: u32,
        transact_time: &str,
        order_quantity: u64,
        order_type: u32,
        position_id: Option<String>,
    ) -> String {
        let mut body = String::new();
        //Unique identifier for the order, allocated by the client.
        body = format!("{}11={}|", body, order_id);
        //Instrument identificators are provided by Spotware.
        body = format!("{}55={}|", body, symbol);
        //1= Buy, 2 = Sell
        body = format!("{}54={}|", body, side);
        // Client generated request time.
        body = format!("{}60={}|", body, transact_time);
        //The fixed currency amount.
        body = format!("{}38={}|", body, order_quantity);
        //1 = Market, the Order will be processed by 'Immediate Or Cancel'scheme(see
        //TimeInForce(59): IOC);
        //
        //2 = Limit, the Order will be processed by 'Good Till Cancel' scheme(see
        //TimeInForce(59): GTC).
        //3 = Stop.
        body = format!("{}40={}|", body, order_type);

        match position_id {
            None => (),
            Some(id) => body = format!("{}721={}|", body, id),
        }

        let header = construct_header(
            qualifier,
            application_message_code(ApplicationMessageType::NewOrderSingle),
            message_sequence_number,
            &body,
            &self,
        );
        let header_body = format!("{}{}", &header, &body);
        let trailer = construct_trailer(&header_body);
        let header_message_trailer = format!("{}{}{}", header, body, trailer);
        return header_message_trailer.replace("|", "\u{0001}");
    }
}
fn construct_header(
    qualifier: &str,
    message_type: String,
    message_sequence_number: u64,
    body_message: &String,
    constructer: &MessageConstructer,
) -> String {
    let mut header = String::new();
    // Protocol version. FIX.4.4 (Always unencrypted, must be first field
    // in message.
    header.push_str("8=FIX.4.4|");

    let mut message = String::new();

    // Message type. Always unencrypted, must be third field in message.
    message = format!("{}35={}|", message, message_type);
    // ID of the trading party in following format: <BrokerUID>.<Trader Login>
    // where BrokerUID is provided by cTrader and Trader Login is numeric
    // identifier of the trader account.
    message = format!("{}49={}|", message, constructer.sender_comp_id);
    // Message target. Valid value is "CSERVER"
    message = format!("{}56={}|", message, constructer.target_comp_id);
    // Additional session qualifier. Possible values are: "QUOTE", "TRADE".
    message = format!("{}57={}|", message, qualifier);
    // Assigned value used to identify specific message originator.
    message = format!("{}50={}|", message, qualifier);
    // Message Sequence Number
    message = format!("{}34={}|", message, message_sequence_number);

    // Time of message transmission (always expressed in UTC(Universal Time
    // Coordinated, also known as 'GMT').
    let utc: DateTime<Utc> = Utc::now();
    message = format!("{}52={}|", message, utc.format("%Y%m%d-%H:%M:%S"));
    let length = message.len() + body_message.len();
    // Message body length. Always unencrypted, must be second field in message.

    header = format!("{}9={}|{}", header, length, message);
    return header;
}

fn construct_trailer(message: &String) -> String {
    //Three byte, simple checksum. Always last field in message; i.e. serves,
    //with the trailing<SOH>,
    //as the end - of - message delimiter. Always defined as three characters
    //(and always unencrypted).
    let msg = message.replace("|", "\u{0001}");
    let trailer = format!("10={}|", calculate_checksum(msg));
    return trailer;
}

fn calculate_checksum(data_to_calculate: String) -> String {
    let mut checksum: u32 = 0;

    for b in data_to_calculate.as_bytes() {
        checksum += *b as u32;
    }

    return format!("{:0>3}", checksum % 256);
}

fn session_message_code(message_type: SessionMessageType) -> String {
    match message_type {
        SessionMessageType::Heartbeat => "0".to_string(),
        SessionMessageType::Logon => "A".to_string(),
        SessionMessageType::Logout => "5".to_string(),
        // SessionMessageType::Reject => "3".to_string(),
        // SessionMessageType::Resend => "2".to_string(),
        // SessionMessageType::SequenceReset => "4".to_string(),
    }
}

fn application_message_code(message_type: ApplicationMessageType) -> String {
    match message_type {
        ApplicationMessageType::MarketDataRequest => "V".to_string(),
        ApplicationMessageType::NewOrderSingle => "D".to_string(),
        // ApplicationMessageType::MarketDataIncrementalRefresh => "X".to_string(),
        // ApplicationMessageType::ExecutionReport => "8".to_string(),
        // ApplicationMessageType::BusinessMessageReject => "j".to_string(),
    }
}
