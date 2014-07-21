use session_manager::SessionManager;
use std::io::net::tcp::TcpStream;
use std::sync::RWLockReadGuard;

use stanza_parser::get_root_attribute;

pub fn route_presence (
    currentUser: &str,
    sessions: RWLockReadGuard<Box<SessionManager+Share+Send>>,
    presence: &str,
    writerStream : &mut TcpStream
) {


    // TODO: check in the RFC what we're really supposed to do
    // get the "to" attribute, ignore the message if not present
    let from = get_root_attribute(presence, "from").unwrap_or("".to_string());
    let to = get_root_attribute(presence, "to").unwrap_or("".to_string());
    let typeAttr =get_root_attribute(presence, "type").unwrap_or("".to_string());


    // if the presence is destinated to us, we directly write
    // it on the output stream and stop here
    if to.as_slice() == currentUser {
        let _ = writerStream.write(presence.as_bytes());
        return;
    }

    // according to RFC, if a presence has no 'to' and no 'type' attribute
    // we multicast it to everybody allowed by the user in the user's roster

    if to.is_empty() && typeAttr.is_empty() {
        //TODO: for the moment we broadcast to EVERYONE
        // instead we should follow the RFC and only broadcast to allowed
        // people
        //for (sessionJid, extQueue) in queues.iter() {
        //    if sessionJid.as_slice() == currentUser {continue;}

        //    extQueue.push(format!(
        //        "<presence from='{}' to='{}'>\
        //            {}\
        //        </presence>",
        //        currentUser,
        //        sessionJid,
        //        ::stanza_parser::get_inside(presence)
        //    ));
        //}
    }

}
