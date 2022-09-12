use bakaproto::proto::message;

use protobuf::Message;

pub struct BakaMessage {
    pub author: String,
    pub content: String,
}

impl BakaMessage {
    pub fn build(&mut self) -> message::Message {
        let mut message = message::Message::new();

        message.author = self.author.clone();
        message.content = self.content.clone();

        return message;
    }
}

impl std::convert::From<String> for BakaMessage {
    fn from(buf: String) -> Self {
        let mut msg = message::Message::new();

        match msg.merge_from_bytes(buf.as_bytes()) {
            Ok(_) => {}
            Err(e) => panic!("{}", e),
        }

        BakaMessage {
            author: msg.author,
            content: msg.content,
        }
    }
}
