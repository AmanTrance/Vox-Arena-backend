use std::{collections::HashMap, sync::mpsc::{self, Sender}};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type ConnectionId = String;
pub type UserSender = mpsc::Sender<Message>;
pub type ArenaReceiver = mpsc::Receiver<Command>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    audio: Box<[u8]>
}

pub enum Command {
    Connect {

    }
}

pub struct Room {
    room_id: String,
    users: Vec<ConnectionId>,
    started: bool
}

impl Room {
    pub fn new() -> Self {
        Self {
            room_id: Uuid::new_v4().to_string(),
            users: Vec::with_capacity(5),
            started: false
        }
    }

    pub fn length(&self) -> usize {
        self.users.len()
    } 
}

pub struct ArenaHandler {
    user_map: HashMap<ConnectionId, UserSender>,
    rooms: Vec<Room>,
    receiver: ArenaReceiver
}

impl ArenaHandler {
    fn new() -> (Self, Sender<Command>) {
        let (rx, rc) = mpsc::channel::<Command>();
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
        for room in self.rooms.iter_mut() {
            if room.length() < 5 {
                room.users.push(id.clone());
                room_id = Some(room.room_id.clone());
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
                    if room.room_id == id {
                        for (index, user_id) in room.users.iter().enumerate() {
                            if user_id == &conn_id {
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
}