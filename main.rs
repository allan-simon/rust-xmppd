extern crate serialize;

use std::str;
use std::io::{Listener, Acceptor};
use std::io::net::tcp::TcpListener;

use serialize::base64::FromBase64;

fn main() {
    let mut acceptor = TcpListener::bind("127.0.0.1", 5222).listen().unwrap();
    println!("listening started, ready to accept");
    for opt_stream in acceptor.incoming() {
        spawn(proc() {
            let mut stream = opt_stream.unwrap();
            let mut buf = [0, ..1024];
            loop {
                match stream.read(buf) {
                    Ok(n) => {
                        let optString = str::from_utf8(buf.slice_to(n));
                        let string = optString.unwrap();
                        println!("{}", string);

                        // start of stream client side, we also start our <stream>
                        // and we advertize we only support PLAIN SASL for the moment
                        if string.starts_with("<stream:stream") {

                            let answer = "\
                                <?xml version='1.0' ?>\
                                <stream:stream \
                                    from='127.0.0.1' \
                                    id='someid' \
                                    xmlns='jabber:client' \
                                    xmlns:stream='http://etherx.jabber.org/streams' \
                                    version='1.0' \
                                >";
                            let answer2 = "\
                                <stream:features>\
                                    <mechanisms xmlns='urn:ietf:params:xml:ns:xmpp-sasl'>\
                                        <mechanism>PLAIN</mechanism>\
                                    </mechanisms>\
                                </stream:features>";

                            let _ = stream.write(answer.as_bytes());
                            let _ = stream.write(answer2.as_bytes());

                        // the client start to send us authentification stuff
                        } else if string.starts_with("<auth") {
                            let tmpString = string.splitn('>', 1).nth(1).unwrap();
                            let base64Auth = tmpString.splitn('<', 1).nth(0).unwrap();
                            println!("{}", base64Auth);
                            let auth = base64Auth.from_base64();
                            println!("{}", auth);
                        }
                    },
                    Err(_) => break,
                };
            }
        })
    }
}
