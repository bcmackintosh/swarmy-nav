pub trait Signal {
    fn serialize(&self) -> String;
    fn deserialize(data: String) -> Self;
}