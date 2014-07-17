///
///
pub fn get_iq_id (
    iq: &str
) -> String {
    let iqString = iq.to_string();
    let iqTag = iqString.as_slice().splitn('>', 1).nth(0).unwrap();
    let idAttr = iqTag.splitn(' ', 3).find(|&x| x.starts_with("id=")).unwrap();
    let id = idAttr.splitn('\'', 2).nth(1).unwrap();
    
    id.to_string()
}

///
///
pub fn get_iq_first_child (
    iq: &str
) -> String {
    let iqString = iq.to_string();
    let tagStart = iqString.as_slice().splitn('>', 2).nth(1).unwrap();
    let tmp = tagStart.splitn(' ', 1).nth(0).unwrap();
    let firstChild = tmp.splitn('<', 1).nth(1).unwrap();

    firstChild.to_string()
}
