extern crate serialize;

use std::str;
use std::io::{Listener, Acceptor};
use std::io::net::tcp::TcpListener;
use std::slice::ImmutableVector;

use serialize::base64::FromBase64;

fn main() {
    let mut acceptor = TcpListener::bind("127.0.0.1", 5222).listen().unwrap();
    println!("listening started, ready to accept");
    for opt_stream in acceptor.incoming() {
        spawn(proc() {
            let mut authenticated = false;
            let mut stream = opt_stream.unwrap();
            let mut buf = [0, ..1024]; loop {
                match stream.read(buf) {
                    Ok(n) => {
                        let optString = str::from_utf8(buf.slice_to(n));
                        let string = optString.unwrap();

                        // start of stream client side, we also start our <stream>
                        // and we advertize we only support PLAIN SASL for the moment
                        if string.starts_with("<stream:stream") && !authenticated {
                            send_initial_stream(&mut stream);
                        // the client start to send us authentification stuff
                        } else if string.starts_with("<auth") {
                            authenticated = treat_login(
                                string,
                                &mut stream
                            );
                        } else if  string.starts_with("<stream:stream") && authenticated {

                            println!("we are authenticated !!!!!");
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
                                    <session xmlns='urn:ietf:params:xml:ns:xmpp-session'/> \
                                </stream:features>";

                            let _ = stream.write(newStream.as_bytes());
                            let _ = stream.write(streamFeatures.as_bytes());
                        }
                    },
                    Err(_) => break,
                };
            }
        })
    }
}

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

fn treat_login (
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

    is_login_correct(
        username.as_slice(),
        password.as_slice()
    )
}

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


fn is_login_correct(
    username: &str,
    password: &str
) -> bool {

    static USERNAME: &'static str = "him";
    static PASSWORD: &'static str = "mypassword";


    return username == USERNAME &&
        password == PASSWORD;

}
