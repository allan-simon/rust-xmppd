use std::io::net::tcp::TcpStream;

static DOMAIN : &'static str = "localhost";

/// send the second <stream> to the client and start to
/// advertize the stream features for binding a resource
/// to the session
pub fn start (
    stream : &mut TcpStream
 ) {
    let newStream = "\
        <stream:stream xmlns='jabber:client' \
            xmlns:stream='http://etherx.jabber.org/streams' \
            id='c2s_345' \
            from='localhost' \
            version='1.0'
        >";

    let streamFeatures = "\
        <stream:features> \
            <bind xmlns='urn:ietf:params:xml:ns:xmpp-bind'/> \
        </stream:features>";

    let _ = stream.write(newStream.as_bytes());
    let _ = stream.write(streamFeatures.as_bytes());
}



///
///
pub fn treat (
    iq: &str,
    stream: &mut TcpStream
) -> Option<String> {

    //
    match ::IqParser::get_iq_first_child(iq).as_slice() {
        "bind" => {},
        _ => return None,
    }

    // find the value inside <resource>
    let tmpString = iq.splitn('>', 3).nth(3).unwrap();
    let resource = tmpString.splitn('<', 1).nth(0).unwrap();

    // find the iq  id 
    let id = ::IqParser::get_iq_id(iq);
    
    println!("{}", id);
    send_resource_binding_result(
        resource,
        id.as_slice(),
        stream
    );
     
    Some(resource.to_string())
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
