use std::{collections::HashMap, io};
use actix_web::web::Bytes;
use tokio::sync::mpsc::{self, error::SendError, Sender};
use uuid::Uuid;

pub type ConnectionId = String;
pub type UserSender = mpsc::Sender<Message>;
pub type ArenaReceiver = mpsc::Receiver<Command>;

#[derive(Debug, Clone)]
pub struct Message {
    pub optional: Option<(String, Option<String>)>,
    pub audio: Option<Bytes>
}

pub enum Command {
    Connect {
        sender: mpsc::Sender<Message>
    },
    Disconnect {
        conn_id: ConnectionId,
        room_id: Option<String>
    },
    ClientMessage {
        conn_id: ConnectionId,
        room_id: String,
        msg: Message
    }
}

#[derive(Debug)]
pub struct Room {
    room_id: String,
    users: Vec<ConnectionId>,
    started: bool
}

impl Room {
    fn new() -> Self {
        Self {
            room_id: Uuid::new_v4().to_string(),
            users: Vec::with_capacity(5),
            started: false
        }
    }

    fn length(&self) -> usize {
        self.users.len()
    } 
}

pub struct ArenaHandler {
    user_map: HashMap<ConnectionId, UserSender>,
    rooms: Vec<Room>,
    receiver: ArenaReceiver
}

impl ArenaHandler {
    pub fn new() -> (Self, Sender<Command>) {
        let (rx, rc) = mpsc::channel::<Command>(100);
    (
        Self { 
            user_map: HashMap::<ConnectionId, UserSender>::new(), 
            rooms: Vec::<Room>::new(), 
            receiver:  rc
        },
        rx
    )
    }

    fn user_connect(&mut self, sender: UserSender) -> (ConnectionId, Option<String>) {
        let id: ConnectionId = Uuid::new_v4().to_string();
        let mut room_id: Option<String> = None;
        self.user_map.insert(id.clone(), sender);
        if self.rooms.len() == 0 || self.rooms[self.rooms.len() - 1_usize].length() == 5 {
            self.rooms.push(Room::new());
        } 
        for room in self.rooms.iter_mut() {
            if room.length() < 5 {
                room.users.push(id.clone());
                room_id = Some(room.room_id.clone());
                if room.length() == 5 {
                    room.started = true;
                }
            }
        }
        (id, room_id)
    }

    fn user_disconnect(&mut self, conn_id: ConnectionId, room_id: Option<String>) -> () {
        self.user_map.remove_entry(&conn_id);
        match room_id {
            Some(id) => {
                let mut i: usize = 0;
                for room in self.rooms.iter_mut() {
                    if (*room).room_id == id {
                        for (index, user_id) in room.users.iter().enumerate() {
                            if *user_id == conn_id {
                                i = index;        
                            }
                        }
                        room.users.remove(i); 
                    }
                }
            },
            None => return
        }
    }

    async fn send_message(&self, conn_id: ConnectionId, msg: Message, room_id: String) -> Result<(), SendError<Message>> {
        for room in self.rooms.iter() {
            if (*room).room_id == room_id {
                for id in room.users.iter() {
                    if *id != conn_id {
                        let user_channel =  self.user_map.get(id).unwrap();
                        user_channel.send(msg.clone()).await?;
                    }
                }
                break
            }
            continue
        }
        Ok(())
    }

    pub async fn run(mut self) -> io::Result<()> {
        while let Some(command) = self.receiver.recv().await {
            match command {
                Command::Connect { sender } => {
                    let user_data = self.user_connect(sender.clone());
                    let _ = sender.send(Message { optional: Some(user_data), audio: None }).await.unwrap();
                },
                Command::Disconnect { conn_id, room_id } => {
                    self.user_disconnect(conn_id, room_id);
                },
                Command::ClientMessage { conn_id, room_id, msg } => {
                    let _ = self.send_message(conn_id, msg, room_id).await.unwrap();
                }
            }
        }

        Ok(())
    } 
}