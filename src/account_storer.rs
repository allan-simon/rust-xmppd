extern crate serialize;
use self::serialize::json;
use std::collections::HashMap;
use std::io::File;

use std::clone::Clone;

/// Trait that represent a storage place for accounts
/// it can be anything as long as it can take a username/password/host
/// and tells if it's correct or not
pub trait AccountStorer {

    /// the string that identify the accounts storage
    /// for example path of the account file, connection string
    /// to the database etc. 
    fn new(storagePath: &str) -> Self;

    /// Check if a user authentication is correct or not
    /// TODO: add a third paramter "host" so that we can support
    /// multiple host on one server
    fn is_login_correct(
        &self,
        username: &str,
        password: &str
    ) -> bool;

}


/// Account storer where the storage place is a on-disk
/// json file that will be then loaded in memory
pub struct JsonAccountStorer {
    storage: HashMap<String, String>
}


///
///
///
impl AccountStorer for JsonAccountStorer {


    ///
    ///
    ///
    fn new(storagePath: &str) -> JsonAccountStorer {

        let path = Path::new(storagePath);

        let mut file = match File::open(&path) {
            Ok(f) => f,
            Err(e) => fail!("file error: {}", e)
        };
        
        let contents = match file.read_to_string() {

            Ok(string) => string,
            Err(e) => fail!("reading fail error: {}", e)
        };


        JsonAccountStorer {
            storage : json::decode(contents.as_slice()).unwrap()
        }

    }

    ///
    ///
    ///
    fn is_login_correct(
        &self,
        username: &str,
        password: &str
    ) -> bool {

        match self.storage.find_equiv(&username) {
            Some(storedPassword) => storedPassword.as_slice() == password,
            None => false
        }

    }
}

/// clone for JsonAccountStorer it will simply duplicate the in-memory
/// hashmap of username => password
///
impl Clone for JsonAccountStorer {

    fn clone(&self) -> JsonAccountStorer {

        JsonAccountStorer {
            storage: self.storage.clone()
        }

    }

} 
