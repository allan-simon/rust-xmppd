extern crate serialize;
use self::serialize::base64::FromBase64;

use std::str;
use std::io::net::tcp::TcpStream;

use account_storer::JsonAccountStorer;
use account_storer::AccountStorer;


/// take a authentication <auth> xml tag and treat it
/// depending of the content different answer may be answered back
/// at the end we return if the user is not authenticated or not
///
pub fn treat_login (
    accountStorer: &JsonAccountStorer,
    saslAuth: &str,
    stream : &mut TcpStream
) -> String {
    //naive split to the text content inside <auth>
    let tmpString = saslAuth.splitn('>', 1).nth(1).unwrap();
    let base64Auth = tmpString.splitn('<', 1).nth(0).unwrap();

    //get the username and password out of the base64 string
    let (_, username, password) = extract_real_username_password(base64Auth);

    println!("{} {}", username, password);


    let authenticated = accountStorer.is_login_correct(
        username.as_slice(),
        password.as_slice()
    );


    if authenticated {
        let answer = "<success xmlns='urn:ietf:params:xml:ns:xmpp-sasl'/>";
        let _ = stream.write(answer.as_bytes());

        username
    } else  {
        let answer = "\
            <failure xmlns='urn:ietf:params:xml:ns:xmpp-sasl'>\
                <not-authorized/>\
            </failure>";
        let _ = stream.write(answer.as_bytes());

        "".to_string()
    }

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

    let realm = str::from_utf8(split[0].as_slice()).unwrap().to_string();
    let username = str::from_utf8(split[1].as_slice()).unwrap().to_string();
    let password = str::from_utf8(split[2].as_slice()).unwrap().to_string();

    (realm, username, password)
}


