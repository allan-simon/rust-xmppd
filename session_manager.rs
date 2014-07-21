use std::sync::mpmc_bounded_queue::Queue;
use std::collections::HashMap;
use std::sync::Arc;



///
///
///
pub trait SessionManager {

    fn new() -> Self;

    fn add_domain (
        &mut self,
        domainName: &str
    ) -> bool;

    fn add_user(
        &mut self,
        domainName: &str, 
        username: &str
    ) -> bool;

    fn add_session_resource(
        &mut self,
        domainName: &str,
        username: &str,
        resource: &str
    ) -> Option<Queue<String>>;


    fn push_to(
        &self,
        to: &str,
        stanza: &str
    );
}

///
///
///
pub struct Session {
    pub queue: Queue<String>,
    domain: String,
    username: String,
    resource: String,
    pub jid: String,
    pub fullJid: String,
}

///
///
///
impl Session {
    
    pub fn new(
        domainName: &str,
        username: &str,
        resource: &str
    ) -> Session {

        Session {
            queue: Queue::with_capacity(42),
            domain: domainName.to_string(),
            username: username.to_string(),
            resource: resource.to_string(),
            jid: format!("{}@{}", domainName, username),
            fullJid: format!("{}@{}/{}", domainName, username, resource)

        }
    }

}


///
///
///
pub struct InMemorySessionManager {
    // host -> username -> resource
    storage: HashMap<
        String, HashMap< // domain => all users
            String, HashMap< // users => all resources
                String, Session // resource => session
            >   
        >
    >

}


///
///
///
impl SessionManager for InMemorySessionManager {

    fn new() -> InMemorySessionManager {

        InMemorySessionManager {
            storage: HashMap::new()
        }

    }

    fn add_domain (
        &mut self,
        domainName: &str
    ) -> bool {

        let domain = domainName.to_string();
        self.storage.insert(domain, HashMap::new());

        true
    }

    fn add_user(
        &mut self,
        domainName: &str, 
        username: &str
    ) -> bool {


        let domain = &domainName.to_string();
        let user = username.to_string();

        if !self.storage.contains_key(domain) {
            return false;
        }

        self.storage.get_mut(domain).insert(user, HashMap::new());

        true
    }

    fn add_session_resource(
        &mut self,
        domainName: &str,
        username: &str,
        resource: &str
    ) -> Option<Queue<String>> {

        let domain = &domainName.to_string();
        let user = &username.to_string();
        let stringResource = resource.to_string();

        if !self.storage.contains_key(domain) {
            return None;
        }

        if !self.storage.get(domain).contains_key(user) {
            return None;
        }

        let session = Session::new(
            domainName,
            username,
            resource
        );

        //let sharedSession = Arc::new(session);

        self.storage.get_mut(domain).get_mut(user).insert(
            resource.to_string(),
            session
        );

        Some(self.storage.get(domain).get(user).get(&stringResource).queue.clone())
    }


    ///
    ///
    fn push_to(
        &self,
        to: &str,
        stanza: &str
    ) {

        let username = to.splitn('@', 0).nth(0).unwrap_or("");
        let tmp = to.splitn('@', 0).nth(1).unwrap_or("");
        let resource = to.splitn('/', 0).nth(1).unwrap_or("");
        let domain = to.splitn('/', 0).nth(0).unwrap_or("");
        println!(
            "to user {} domain {} resource {}",
            username,
            resource,
            domain
        );
        // check if we need to send this message to the queue
        // of somebody else
        //for (sessionJid, extQueue) in queues.iter() {

        //    // we dont push message to our own queue to avoid
        //    // infinite loop of messages...
        //    if sessionJid.as_slice() == currentUser {continue;}
        //    
        //    // we ignore people who are not in the "to"
        //    // TODO: see if we need to make it like that or if
        //    // we can directly make a "get" (instead of for loop)
        //    if sessionJid.as_slice() != to.as_slice() {continue;}

        //    println!("{} sends to {}", currentUser, sessionJid);

        //    extQueue.push(messageToSend.clone());
        //}



    }
   

}
