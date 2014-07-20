use std::collections::HashMap;
use std::sync::mpmc_bounded_queue::Queue;
use std::io::net::tcp::TcpStream;
use std::sync::RWLockReadGuard;

pub fn route_message (
    currentUser: &str,
    queues: RWLockReadGuard<HashMap<String, Queue<String>>>,
    message: &str,
    writerStream : &mut TcpStream
) {

    // TODO: check in the RFC what we're really supposed to do
    // get the "to" attribute, ignore the message if not present
    let to = match ::stanza_parser::get_root_attribute(message, "to") {
        Some(toAttr) => toAttr,
        None => return
    };

    // we generate the message to send from the information
    // we got through the original message, we dont try to be
    // smart about the content inside <message> and we simply
    // copy it, however for the attribute of message itself
    // we set the "from" and "to" ourselves
    // TODO: setting the from should certainly be done 
    // in the TCP Stream to Stanza process
    let messageToSend = format!(
        //TODO: 'chat' is hardcoded and should not
        "<message from='{}' to='{}' type='chat'>\
            {}\
        </message>",
        currentUser,
        to,
        ::stanza_parser::get_inside(message)
    );


    // if  the message is for us, we directly write it
    // on the stream
    if currentUser == to.as_slice() {
        println!("{}: message for us", to);
        let _ = writerStream.write(messageToSend.as_bytes());
        return;
    }

    // check if we need to send this message to the queue
    // of somebody else
    for (sessionJid, extQueue) in queues.iter() {

        // we dont push message to our own queue to avoid
        // infinite loop of messages...
        if sessionJid.as_slice() == currentUser {continue;}
        
        // we ignore people who are not in the "to"
        // TODO: see if we need to make it like that or if
        // we can directly make a "get" (instead of for loop)
        if sessionJid.as_slice() != to.as_slice() {continue;}

        println!("{} sends to {}", currentUser, sessionJid);

        extQueue.push(messageToSend.clone());
    }

}
