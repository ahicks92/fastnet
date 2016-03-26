pub enum PacketDecodingError {
    //We need more bytes than what we got.
    TooSmall,
    //The packet has a checksum, but we didn't match it.
    ChecksumMismatch,
    UnknownChannel,
    InvalidFormat,
}
