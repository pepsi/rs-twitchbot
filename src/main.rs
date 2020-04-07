use std::io::prelude::*;
use std::net::TcpStream;
use std::char;
use regex::Regex;
use hexdump;
// liechtenstein
// oauth:q9o9s9xyto1sffu72gwn5l6dmhleqd
#[derive(Debug)]
struct Message<'a> {
    username: &'a str,
    content : &'a str,
    channel : &'a str,
    _stream : &'a TcpStream
}
impl Message<'_> {
    fn send_message(&mut self, message: &str){
        println!("trying send message! C:{} n:{}", self.channel, message);
        self._stream.write_all(format!("PRIVMSG #{} :{}\n\r", self.channel, message).as_bytes()).expect("Failed to send a message!");
    }
}
fn on_message(mut message: Message){
    println!("{{\n  Username: {}\n  Channel: {}\n  Content: {}\n}}", message.username, message.channel, message.content);
    // Debug to see the trailing characters. 0D: \r, 0A: \n. 
    // http://dc.org/files/asciitable.pdf
    hexdump::hexdump(message.content.as_bytes());

    if message.content == "!test"{
        message.send_message("Test!");
    }
}
fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("irc.chat.twitch.tv:6667").expect("Connection to server failed!");
    macro_rules! send_message {
        ($channel:expr, $message:expr) => (
            let _w = stream.write_all(format!("PRIVMSG #{} :{}\n\r", $channel, $message).as_bytes());
        );
    }
    /*
    Doing this all manually so that i dont need to write macros that i only use once.
    */
    stream.write(b"PASS oauth:q9o9s9xyto1sffu72gwn5l6dmhleqd\n\r").expect("Failed to send password");
    stream.write(b"NICK liechtenstein\n\r").expect("Failed to send nickname");
    stream.write(b"JOIN #liechtenstein\n\r").expect("Failed to join channel");
    stream.write(b"JOIN #primalzachfps\n\r").expect("Failed to join channel");
    /*
    Test send_message! macro.
    */
    send_message!("liechtenstein", "Test");
    loop {
        //start 2kb buffer
        let mut buffer = vec![0; 2048];
        //read stream into said buffer
        stream.read(&mut buffer)?;
        //convert buffer to utf and trim null bytes from the end
        let mut msg = String::from_utf8(buffer).expect("Invalid UTF-8");
        msg = msg.trim_matches(char::from(0)).to_string();
        //Handle twitch ping requests
        if msg == "PING :tmi.twitch.tv\n" {
            println!("Trying to PONG");
            stream.write_all(b"PONG :tmi.twitch.tv\n\r").expect("Could not send PONG to twitch servers!");
        }
        // Parse 
        let message_regex = Regex::new(r":(.*)!(?:.*)@(?:.*)\.tmi\.twitch\.tv PRIVMSG #(.*) :(.*)").unwrap();
        for cap in message_regex.captures_iter(&msg){
            let m = Message{
                username: &cap[1],
                channel: &cap[2],
                content: &cap[3].replace("\n", "").replace("\r", ""),
                _stream: &stream
            };
            //call message event
            print!("{:?}", m);
            on_message(m);

        }
    }
} 