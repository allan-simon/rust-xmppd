//! rust-xmppd, xmpp server written in rust
//!
use std::str;
use std::io::{Listener, Acceptor};
use std::io::net::tcp::TcpListener;
use std::slice::ImmutableVector;
use std::sync::Arc;


use std::collections::HashMap;
// TODO though mpsc would have been the more semantically appropriate
// (i.e only the current session pop from its own queue, and the other
// only push), currently there's no "bounded" version of it, and under
// heavy load mpsc will run out of memory and make the program to be
// OOM killed
//use std::sync::mpsc_queue::Queue;
use std::sync::mpmc_bounded_queue::Queue;
use std::sync::RWLock;
use std::io::timer::sleep;

use account_storer::JsonAccountStorer;
use account_storer::AccountStorer;

mod IqParser;
mod IqRouter;
mod account_storer;
mod message_router;
mod stanza_parser;
mod auth;


fn main() {
    let mut acceptor = TcpListener::bind("127.0.0.1", 5222).listen().unwrap();
    println!("listening started, ready to accept");

    let accountStorer: JsonAccountStorer = AccountStorer::new("data/login.json");
    let sharedAccountStorer = Arc::new(accountStorer);

    // made to map a  Full JID to a Queue
    let queuesByFullJid: HashMap<String, Queue<String>> = HashMap::new();
    let sharedQueuesByFullJid = Arc::new(RWLock::new(queuesByFullJid));

    for opt_stream in acceptor.incoming() {
        // create a clone of shared ressources that need to be
        // accessed by each connection
        let localAccountStorer = sharedAccountStorer.clone();
        let localQueues = sharedQueuesByFullJid.clone();

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

            ///////////////////////////
            // authenticated part
            //////////////////////////

            //now that we are authenticated we are ready to
            //receive messages from others
            let queue : Queue<String> = Queue::with_capacity(42);
            let mut hash = localQueues.write();
            //TODO: replace by something smarter and containing 
            //the resource not only the bare JID
            let jid = format!("{}@localhost",username.clone());
            hash.insert(jid.clone(), queue.clone());
            hash.downgrade();

            let sharedQueue = Arc::new(queue);
            let queueReader = sharedQueue.clone();
            let queueWriter = sharedQueue.clone();

            let mut readerStream = stream.clone();
            let localJid = jid;

            // process that keep reading for new stanza on our
            // internal Queue
            spawn(proc() {
                let mut writerStream = stream.clone();
                loop { match queueReader.pop() {
                    Some(data) => {
                        //let optString = str::from_utf8(buf.slice_to(n));
                        //let string = optString.unwrap();
                        let string = data.as_slice(); 
                        println!("{}: {}", localJid, string);

                        if string.starts_with("<stream:stream") {
                            start_resource_binding(&mut writerStream);
                        } else if string.starts_with("<iq ") {

                            ::IqRouter::route_iq(string, &mut writerStream);

                        } else if string.starts_with("<message ") {

                            ::message_router::route_message(
                                localJid.as_slice(),
                                localQueues.read(),
                                string,
                                &mut writerStream
                            );
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



