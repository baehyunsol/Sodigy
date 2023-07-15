use super::AST;

impl AST {

    pub fn encode(&self) -> Vec<u8> {
        todo!()
    }

    // we don't care about the error_kind of `decode`
    // the compiler will just re-generate the AST if the decode fails
    pub fn decode(b: &[u8]) -> Option<Self> {
        todo!()
    }

}