use std::io::net::tcp::TcpStream;

static DOMAIN : &'static str = "localhost";

///
///
///
pub fn route_iq (
    iq: &str,
    stream : &mut TcpStream

) {

    match ::IqParser::get_iq_first_child(iq).as_slice() {
        "session" => treat_session(iq, stream),
        "ping" => treat_ping(iq, stream),
        _ => {
            println!("iq not treated!");
            println!("{}", iq);
            send_dummy_iq_result(iq, stream);
            return;
        }
    }
}

///
///
fn treat_session(
    sessionIq: &str,
    stream: &mut TcpStream
) {
    send_dummy_iq_result(sessionIq, stream);
}

/// as of XEP 0199 replying a pong is simply
/// replying a iq result
///
fn treat_ping(
    sessionIq: &str,
    stream: &mut TcpStream
) {
    send_dummy_iq_result(sessionIq, stream);
}

/// send on the wire a dummy iq of type result
/// with the same ID
///
fn send_dummy_iq_result (
    iq: &str,
    stream : &mut TcpStream
) {
    let id = ::IqParser::get_iq_id(iq);
    let result = format!(
        "<iq from='{domain}' type='result' id='{id}'/>",
        domain = DOMAIN,
        id = id
    );

    let _ = stream.write(result.as_bytes());
}

