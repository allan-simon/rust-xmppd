//! rust-xmppd, xmpp server written in rust
//!
use std::str;
use std::io::{Listener, Acceptor};
use std::io::net::tcp::TcpListener;
use std::slice::ImmutableVector;
use std::sync::Arc;


use std::sync::RWLock;
use std::io::timer::sleep;

use account_storer::JsonAccountStorer;
use account_storer::AccountStorer;

use session_manager::SessionManager;
use session_manager::InMemorySessionManager;

mod IqParser;
mod IqRouter;
mod account_storer;
mod message_router;
mod stanza_parser;
mod presence_router;
mod resource_binding;
mod session_manager;
mod auth;


fn main() {
    let mut acceptor = TcpListener::bind("127.0.0.1", 5222).listen().unwrap();
    println!("listening started, ready to accept");

    let accountStorer: JsonAccountStorer = AccountStorer::new("data/login.json");
    let sharedAccountStorer = Arc::new(accountStorer);


    let mut sessionManager : InMemorySessionManager = SessionManager::new();
    sessionManager.add_domain("localhost".as_slice());
    let immutableSessionManager = sessionManager;
    let sharedSessionManager = Arc::new(RWLock::new(box immutableSessionManager as Box<SessionManager+Send+Share>));


    for opt_stream in acceptor.incoming() {
        // create a clone of shared ressources that need to be
        // accessed by each connection
        let localAccountStorer = sharedAccountStorer.clone();
        let localSessionManager = sharedSessionManager.clone();

        spawn(proc() {
            let mut stream = opt_stream.unwrap();
            let mut buf = [0, ..1024];

            let mut username : String;
            //////////////////////////
            // before authentication
            /////////////////////////

            loop { match stream.read(buf) {
                Ok(n) => {
                    let optString = str::from_utf8(buf.slice_to(n));
                    let string = optString.unwrap();
                    // start of stream client side, we also start
                    // our <stream> and we advertize we only support
                    // PLAIN SASL for the moment
                    if string.starts_with("<stream:stream") {
                        send_initial_stream(&mut stream);

                    // the client start to send us authentification
                    // stuff
                    } else if string.starts_with("<auth") {
                        username = ::auth::treat_login(
                            // dereference the counted reference
                            // to have access to it as a normal &
                            &*localAccountStorer,
                            string,
                            &mut stream
                        );

                        if !username.is_empty() {break;}

                    // if the client close the xmpp stream
                    // we close the TCP connection
                    } else if string.starts_with("</stream:stream>") {
                        return;
                    } else  {
                        println!("not auth, not treated!");
                        println!("{}", string);
                    }
                },
                Err(_) => return,
            };}


            // we add user to session
            let mut sessionWriter = localSessionManager.write();
            sessionWriter.add_user(
                "localhost".as_slice(),
                username.as_slice()
            );
            sessionWriter.downgrade();

            let jid = format!("{}@localhost",username.clone());

            ///////////////////////////
            // resource binding part
            //////////////////////////

            let mut resource : String;

            loop { match stream.read(buf) {
                Ok(n) => {
                    let string = str::from_utf8(buf.slice_to(n)).unwrap();
                    println!("{}: {}", jid, string);

                    if string.starts_with("<stream:stream") {
                        ::resource_binding::start(&mut stream);
                    } else if string.starts_with("<iq ") {
                        match ::resource_binding::treat(string, &mut stream) {
                            Some(data) => {
                                resource = data;
                                break;
                            },
                            None => return,
                        }
                    } else {
                        println!("session binding fail {}: {}", jid, string);
                        return;
                    }
                },
                Err(_) => return,
            };}


            ///////////////////////////
            // authenticated part
            //////////////////////////
            println!("we are authenticated and bound!");

            //now that we are authenticated we are ready to
            //receive messages from others

            let mut sessionWriter = localSessionManager.write();
            let queue = sessionWriter.add_session_resource(
                "localhost".as_slice(),
                username.as_slice(),
                resource.as_slice()
            ).unwrap();
            sessionWriter.downgrade();


            //TODO: replace by something smarter
            // we map the current full jid to current queue
            let fullJid = format!("{}/{}", jid, resource);

            let sharedQueue = Arc::new(queue);
            let queueReader = sharedQueue.clone();
            let queueWriter = sharedQueue.clone();

            let mut readerStream = stream.clone();
            let localFullJid = fullJid;

            // process that keep reading for new stanza on our
            // internal Queue
            spawn(proc() {
                let mut writerStream = stream.clone();
                loop { match queueReader.pop() {
                    Some(data) => {
                        //let optString = str::from_utf8(buf.slice_to(n));
                        //let string = optString.unwrap();
                        let string = data.as_slice(); 
                        println!("{}: {}", localFullJid, string);

                        if string.starts_with("<iq ") {
                            ::IqRouter::route_iq(string, &mut writerStream);
                        } else if string.starts_with("<message ") {

                            ::message_router::route_message(
                                localFullJid.as_slice(),
                                localSessionManager.read(),
                                string,
                                &mut writerStream
                            );
                        } else if
                            string.starts_with("<presence>") ||
                            string.starts_with("<presence ") ||
                            string.starts_with("<presence/>")
                        {
                            //::presence_router::route_presence(
                            //    localFullJid.as_slice(),
                            //    localSessionManager.read(),
                            //    string,
                            //    &mut writerStream
                            //);

                        } else {
                            println!("not treated!");
                            println!("{}", string);
                        }
                    },
                    None => sleep(10),
                };}


            });
            
            // loop that keep reading the TCP stream
            // TODO: make it output Stanza object to gives to the
            // the queue
            loop { match readerStream.read(buf) {
                Ok(n) => {
                    let optString = str::from_utf8(buf.slice_to(n));
                    let string = optString.unwrap();
                    queueWriter.push(string.to_string());
                },
                Err(_) => break,
            };}

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


