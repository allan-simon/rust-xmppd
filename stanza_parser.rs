use std::option::Option;

pub fn get_root_attribute(
    stanza: &str,
    attributeName: &str
) -> Option<String> {

    let stanzaString = stanza.to_string();
    let stanzaTag = stanzaString.as_slice().splitn('>', 1).nth(0).unwrap();
    match stanzaTag.split(' ').find(|&x| x.starts_with(attributeName)) {
        Some(attr) => match attr.splitn('\'', 2).nth(1) {
            Some(value) => return Some(value.to_string()),
            None => return None
        },
        None => return None
    };
} 


pub fn get_inside(stanza: &str) -> String {

    let stanzaString = stanza.to_string();
    let tmpString = stanzaString.as_slice().splitn('>', 1).nth(1).unwrap();

    tmpString.as_slice().rsplitn('<', 1).nth(1).unwrap().to_string()
}
