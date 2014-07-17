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
        "bind" => treat_resource_binding(iq, stream),
        "session" => treat_session(iq, stream),
        _ => {
            println!("iq not treated!");
            println!("{}", iq);
            send_dummy_result(iq, stream);
            return;
        }
    }
}

///
///
fn treat_resource_binding (
    bindIq: &str,
    stream: &mut TcpStream
) {

    //

    // find the value inside <resource>
    let tmpString = bindIq.splitn('>', 3).nth(3).unwrap();
    let resource = tmpString.splitn('<', 1).nth(0).unwrap();

    // find the iq  id 
    let id = ::IqParser::get_iq_id(bindIq);
    
    println!("{}", id);
    send_resource_binding_result(
        resource,
        id.as_slice(),
        stream
    );
     
}


///
///
fn treat_session(
    sessionIq: &str,
    stream: &mut TcpStream
) {
    send_dummy_result(sessionIq, stream);
}


///
///
fn send_dummy_result (
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


///
///
fn send_resource_binding_result (
    resource: &str,
    id: &str,
    stream : &mut TcpStream
) {
    let result = format!(
        "<iq type='result' id='{id}'>\
          <bind xmlns='urn:ietf:params:xml:ns:xmpp-bind'>\
            <jid>him@{domain}/{resource}</jid>\
          </bind>\
        </iq>",
        id = id,
        domain = DOMAIN,
        resource = resource
    );

    let _ = stream.write(result.as_bytes());
}

