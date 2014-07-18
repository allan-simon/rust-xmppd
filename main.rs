//! rust-xmppd, xmpp server written in rust
//!
extern crate serialize;

use std::str;
use std::io::{Listener, Acceptor};
use std::io::net::tcp::TcpListener;
use std::slice::ImmutableVector;

use serialize::base64::FromBase64;

use account_storer::JsonAccountStorer;
use account_storer::AccountStorer;

mod IqParser;
mod IqRouter;
mod account_storer;


fn main() {
    let mut acceptor = TcpListener::bind("127.0.0.1", 5222).listen().unwrap();
    println!("listening started, ready to accept");

    let accountStorer : JsonAccountStorer = AccountStorer::new("data/login.json");

    for opt_stream in acceptor.incoming() {
        let localAccountStorer = accountStorer.clone();
        spawn(proc() {

            let mut authenticated = false;
            let mut stream = opt_stream.unwrap();
            let mut buf = [0, ..1024];
            loop {
                match stream.read(buf) {
                    Ok(n) => {
                        let optString = str::from_utf8(buf.slice_to(n));
                        let string = optString.unwrap();

                        //////////////////////////
                        // before authentication
                        /////////////////////////
                        if !authenticated {

                            // start of stream client side, we also start
                            // our <stream> and we advertize we only support
                            // PLAIN SASL for the moment
                            if string.starts_with("<stream:stream") {
                                send_initial_stream(&mut stream);

                            // the client start to send us authentification
                            // stuff
                            } else if string.starts_with("<auth") {
                                authenticated = treat_login(
                                    &localAccountStorer,
                                    string,
                                    &mut stream
                                );
                            } else {
                                println!("not auth, not treated!");
                                println!("{}", string);
                            }

                            continue;
                        }

                        ///////////////////////////
                        // authenticated part
                        //////////////////////////

                        if string.starts_with("<stream:stream") {
                            start_resource_binding(&mut stream);
                        } else if string.starts_with("<iq ") {

                            ::IqRouter::route_iq(string, &mut stream);

                        } else {
                            println!("not treated!");
                            println!("{}", string);
                        }
                    },
                    Err(_) => break,
                };
            }
        })
    }
}

/// send on the wire the beginning of a xmpp communication (<stream:stream>)
/// and <stream:features> to advertize the auth mechanism we support
fn send_initial_stream (stream : &mut std::io::net::tcp::TcpStream) {

    
    let streamBeginning = "\
        <?xml version='1.0' ?>\
        <stream:stream \
            from='127.0.0.1' \
            id='someid' \
            xmlns='jabber:client' \
            xmlns:stream='http://etherx.jabber.org/streams' \
            version='1.0' \
        >";

    //xml tag to advertize the authentication mechanism we support
    let supportedAuth = "\
        <stream:features>\
            <mechanisms xmlns='urn:ietf:params:xml:ns:xmpp-sasl'>\
                <mechanism>PLAIN</mechanism>\
            </mechanisms>\
        </stream:features>";

    let _ = stream.write(streamBeginning.as_bytes());
    let _ = stream.write(supportedAuth.as_bytes());

}

/// take a authentication <auth> xml tag and treat it
/// depending of the content different answer may be answered back
/// at the end we return if the user is not authenticated or not
///
fn treat_login (
    accountStorer: &JsonAccountStorer,
    saslAuth: &str,
    stream : &mut std::io::net::tcp::TcpStream
) -> bool {
    //naive split to the text content inside <auth>
    let tmpString = saslAuth.splitn('>', 1).nth(1).unwrap();
    let base64Auth = tmpString.splitn('<', 1).nth(0).unwrap();

    //get the username and password out of the base64 string
    let (_, username, password) = extract_real_username_password(base64Auth);

    println!("{} {}", username, password);

    let answer = "<success xmlns='urn:ietf:params:xml:ns:xmpp-sasl'/>";
    let _ = stream.write(answer.as_bytes());

    accountStorer.is_login_correct(
        username.as_slice(),
        password.as_slice()
    )
}

/// take a base64 encoded plain SASL auth payload
/// realm\0username\0password and extract these 3 information
///
fn extract_real_username_password(
    plainSASLBase64Auth : &str
) -> (String, String, String) {

    let saslAuth = plainSASLBase64Auth.from_base64().unwrap();
    let split : Vec<&[u8]> = saslAuth.as_slice().splitn(
        3, // max number of fields  realm+username+password
        |&x| x == 0
    ).collect();

    let realm = str::from_utf8(split.get(0).as_slice()).unwrap().to_string();
    let username = str::from_utf8(split.get(1).as_slice()).unwrap().to_string();
    let password = str::from_utf8(split.get(2).as_slice()).unwrap().to_string();

    (realm, username, password)
}

/// send the second <stream> to the client and start to
/// advertize the stream features for binding a resource
/// to the session
fn start_resource_binding (
    stream : &mut std::io::net::tcp::TcpStream
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



