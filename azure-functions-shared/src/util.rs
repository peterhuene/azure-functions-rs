pub fn to_camel_case(input: &str) -> String {
    let mut result = String::new();
    let mut capitalize = false;
    let mut first = true;
    for ch in input.chars() {
        if ch == '_' {
            capitalize = true;
        } else {
            result.push(if capitalize && !first {
                ch.to_ascii_uppercase()
            } else {
                ch
            });
            first = false;
            capitalize = false;
        }
    }
    result
}
