pub enum Validator {
    AlwaysTrue, // hmmm not every state would need validation
    HasRole { required_roles: Vec<String> },
}
