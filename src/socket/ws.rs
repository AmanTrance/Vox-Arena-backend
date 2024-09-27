use super::manager::{Command, Message};
use actix_ws::{AggregatedMessage, Session, MessageStream};
use futures_util::{future::{select, Either}, StreamExt as _};
use tokio::{sync::mpsc, time::interval};
use std::{sync::Arc, time::{Duration, Instant}, pin::pin};

const HEARTBEATS: Duration = Duration::from_secs(5);
const TIMEOUT: Duration = Duration::from_secs(15); 

pub async fn handle_ws<'s>(sender: Arc<mpsc::Sender<Command>>, mut session: Session, stream: MessageStream ) {
    let mut connection_id: Option<String> = None;
    let mut room_id: Option<String> = None;
    let mut last_heartbeat = Instant::now();
    let mut interval = interval(HEARTBEATS);
    let (crx, mut crc) = mpsc::channel::<Message>(100);
    sender.send(Command::Connect { sender: crx }).await.unwrap();
    let msg_stream: actix_ws::AggregatedMessageStream = stream.max_frame_size(5*1024*1024).aggregate_continuations().max_continuation_size(10*1024*1024);
    let mut msg_stream: std::pin::Pin<&mut actix_ws::AggregatedMessageStream> = pin!(msg_stream);
    let reason = loop {
        let tick = pin!(interval.tick());
        let msg_crc = pin!(crc.recv());
        let messages = pin!(select(msg_stream.next(), msg_crc));

        match select(messages, tick).await {
            Either::Left((Either::Left((Some(Ok(msg)), _)), _)) => {
                match msg {
                    AggregatedMessage::Ping(bytes) => {
                        last_heartbeat = Instant::now();
                        session.pong(&bytes).await.unwrap();
                    },
                    AggregatedMessage::Pong(_) => {
                        last_heartbeat = Instant::now();
                    },
                    AggregatedMessage::Text(_) => { () },
                    AggregatedMessage::Binary(audio) => {
                        sender.send(Command::ClientMessage { conn_id: connection_id.clone().unwrap(), room_id: room_id.clone().unwrap(), msg: Message { optional: None, audio: Some(audio) } }).await.unwrap();
                    },
                    AggregatedMessage::Close(reason) => break reason
                }
            },
            Either::Left((Either::Left((Some(Err(_)), _)), _)) => break None,
            Either::Left((Either::Left((None, _)), _)) => break None,
            Either::Left((Either::Right((Some(msg), _)), _)) => {
                match msg {
                    Message { optional, audio} => {
                        if optional.is_none() && audio.is_none() {
                            break None 
                        } else if optional.is_none() && !audio.is_none(){
                            session.binary(audio.unwrap()).await.unwrap();
                        } else {
                            let (temp_connection_id, temp_room_id) = optional.unwrap();
                            connection_id = Some(temp_connection_id);
                            room_id = temp_room_id; 
                        }
                    }
                }
            },
            Either::Left((Either::Right((None, _)), _)) => unreachable!("server panicked"),
            Either::Right((_, _)) => {
                if Instant::now().duration_since(last_heartbeat) > TIMEOUT {
                    break None
                }
                session.ping(b"").await.unwrap();
            }
        }
    };

    sender.send(Command::Disconnect { conn_id: connection_id.unwrap(), room_id: room_id }).await.unwrap();
    session.close(reason).await.unwrap();
}