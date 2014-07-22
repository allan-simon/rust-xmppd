use std::io::net::tcp::TcpStream;
use std::sync::RWLockReadGuard;
use session_manager::SessionManager;

pub fn route_message (
    currentUser: &str,
    sessions: RWLockReadGuard<Box<SessionManager+Share+Send>>,
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

    //TODO: move that in a jid util module
    let user = currentUser.splitn('@', 1).nth(0).unwrap_or("");
    let tmp = currentUser.splitn('@', 1).nth(1).unwrap_or("");
    let resource = tmp.splitn('/', 1).nth(1).unwrap_or("");
    let domain = tmp.splitn('/', 1).nth(0).unwrap_or("");


    // if  the message is for us, we directly write it
    // on the stream
    // by for us we mean, either directed as a broadcast to our user
    // or specifically to this specific resource
    if format!("{}@{}",user,domain) == to || currentUser.to_string() == to {
        println!("{}: message for us", to);
        let _ = writerStream.write(messageToSend.as_bytes());
        return;
    }

    println!("{} domain {} res:{} and {}",user,domain,resource,to);

    sessions.push_to(
        currentUser.as_slice(),
        to.as_slice(),
        messageToSend.as_slice()
    );
}
