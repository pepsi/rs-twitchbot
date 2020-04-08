use std::io::prelude::*;
use std::net::TcpStream;
use std::char;
use regex::Regex;
use hexdump;
use std::collections::HashMap;
use dotenv;
// liechtenstein
#[derive(Debug)]
struct Message<'a> {
    username: &'a str,
    content : &'a str,
    channel : &'a str,
    _stream : &'a TcpStream
}

struct Command<'a>{
    name: &'a str,
    func: &'a dyn Fn(Context)
}
impl Message<'_> {
    fn send_message(&mut self, message: &str){
        println!("trying send message! C:{} n:{}", self.channel, message);
        self._stream.write_all(format!("PRIVMSG #{} :{}\n\r", self.channel, message).as_bytes()).expect("Failed to send a message!");
    }
}
struct Context<'a>{
    message: Message<'a>,
    commands: &'a HashMap<&'a str, Command<'a>>,
    command_name: Option<&'a str>,
    args: Option<Vec<&'a str>>

}
fn on_message(mut ctx: Context){
    println!("{:>25}@{:<25}: {}", ctx.message.username, ctx.message.channel, ctx.message.content);
    /* 
    Debug to see the trailing characters. 0D: \r, 0A: \n. 
    http://dc.org/files/asciitable.pdf
    */
    // hexdump::hexdump(ctx.message.content.as_bytes());
}
fn on_command(mut ctx: Context){
    if ctx.commands.contains_key(ctx.command_name.unwrap()){
        (ctx.commands[ctx.command_name.unwrap()].func)(ctx);
    }else{
        ctx.message.send_message(&format!("Could not find command {}.", ctx.command_name.unwrap()));
    }
}
fn get_prefix<'a>(message: &Message) -> &'a str{
    if message.channel == "liechtenstein" {
        "!"
    }else{
        "omerdied."
    }
}
fn cmd1(mut ctx: Context){
    ctx.message.send_message("Command 1 invoked!");
    println!("Here is command 1");
}
fn main() -> std::io::Result<()> {
    let channels = [
    //     "thegameawards",
    //  "dota2ti",
    //   "fortnite",
    //    "xqcow",
    //     "timthetatman",
    //      "brax",
    //       "myth",
    //        "drdisrespect",
            "liechtenstein"
            ];
    let mut stream: TcpStream = TcpStream::connect("irc.chat.twitch.tv:6667").expect("Connection to server failed!");
    let mut commands: HashMap<&str, Command> = HashMap::new();
    let command1 = Command{
        name: "Command #1",
        func: &cmd1
    }; 
    let command2 = Command{
        name: "Command #2",
        func: &|mut ctx|{
            ctx.message.send_message("Inline command 2 invoked!");
            println!("Inline Command 2");
        }
    };
    dotenv::dotenv().ok();
    commands.insert("command-1", command1);
    commands.insert("command-2", command2);
    commands.insert("help", Command{
        name: "Help",
        func: &|mut ctx|{
            let mut command_names: Vec<&str> = ctx.commands
                .into_iter()
                .map(|c| *c.0)
                .collect();
                command_names.sort_by(|a ,b| a.to_lowercase().cmp(&b.to_lowercase()));
            ctx.message.send_message(&format!("The list of valid commands are: {}", command_names.join(", ")));
        }
    });
    commands.insert("debug",Command{
        name: "debug",
        func: &|mut ctx: Context|{
            ctx.message.send_message(&ctx.args.unwrap().join(", "));
        }
    });
    //Send message macro, not used much
    macro_rules! send_message {
        ($channel:expr, $message:expr) => (
            let _w = stream.write_all(format!("PRIVMSG #{} :{}\n\r", $channel, $message).as_bytes());
        );
    }
    macro_rules! join_channel {
        ($channel:expr) => {
            stream.write(format!("JOIN #{}\n\r", $channel).as_bytes()).expect("Failed to join channel");
        };
    }
    /*
    Doing this all manually so that i dont need to write macros that i only use once.
    */
    stream.write(format!("PASS {}\n\r", dotenv::var("oauth").unwrap()).as_bytes()).expect("Failed to send password");
    stream.write(b"NICK liechtenstein\n\r").expect("Failed to send nickname");
    for channel in channels.iter(){
        join_channel!(channel);
    }
    /*
    Test send_message! macro.
    */
    // send_message!("liechtenstein", "Test");
    loop {
        //start 2kb buffer
        let mut buffer = vec![0; 2048];
        //read stream into said buffer
        stream.read(&mut buffer)?;
        //convert buffer to string and trim null bytes from the end
        let temp_msg = String::from_utf8(buffer);
        if temp_msg.is_err(){
            continue;
        }
        let mut msg = temp_msg.expect("I didnt think this could be called :/");
        msg = msg.trim_matches(char::from(0)).to_string();
        //Handle twitch ping requests
        if msg == "PING :tmi.twitch.tv\r\n" {
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
            // print!("{:?}", m);
            //Create context object so commands can have proper information when invoked.
            let prefix = get_prefix(&m);
            if m.content.starts_with(prefix){
                let parts: Vec<&str> = m.content.split_ascii_whitespace().collect();
                if parts[0].starts_with(prefix){
                    let command = &parts[0].replace(prefix, "");
                    let ctx = Context{
                        message: m,
                        commands: &commands,
                        command_name: Some(command),
                        args: Some(parts)
                    };
                    on_command(ctx);
                }
            }else{
                let ctx = Context{
                    message: m,
                    commands: &commands,
                    command_name: None,
                    args: None  
                };
                on_message(ctx);
            }
        }
    }
} 