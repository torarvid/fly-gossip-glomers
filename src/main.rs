mod message;
mod message_receiver;
mod node;

use crate::message_receiver::MessageReceiver;

fn main() {
    MessageReceiver::start_reading();
}
