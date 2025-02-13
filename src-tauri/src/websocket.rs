use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::broadcast;
use warp::ws::Message;
use warp::Filter;
use std::sync::atomic::{AtomicBool, Ordering};

static SERVER_RUNNING: AtomicBool = AtomicBool::new(false);

pub async fn start_server_ws(tx: broadcast::Sender<String>) {
    if SERVER_RUNNING.load(Ordering::SeqCst) {
        return;
    }
    SERVER_RUNNING.store(true, Ordering::SeqCst);

    let tx = Arc::new(tx);
    
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let tx = tx.clone();
            ws.on_upgrade(move |socket| websocket_handler(socket, tx))
        });

    println!("WebSocket server starting on ws://127.0.0.1:21297");
    warp::serve(ws_route)
        .run(([127, 0, 0, 1], 21297))
        .await;
}

async fn websocket_handler(
    ws: warp::ws::WebSocket,
    tx: Arc<broadcast::Sender<String>>,
) {
    println!("New WebSocket connection established");
    let (mut ws_tx, mut ws_rx) = ws.split();
    let mut rx = tx.subscribe();

    // Handle incoming messages
    let mut receive_task = tokio::spawn(async move {
        while let Some(result) = ws_rx.next().await {
            match result {
                Ok(msg) => {
                    if msg.is_close() {
                        println!("WebSocket connection closed by client");
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("WebSocket receive error: {}", e);
                    break;
                }
            }
        }
    });

    // Forward broadcast messages to WebSocket
    let mut forward_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Err(e) = ws_tx.send(Message::text(msg)).await {
                eprintln!("WebSocket send error: {}", e);
                break;
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = &mut receive_task => forward_task.abort(),
        _ = &mut forward_task => receive_task.abort(),
    }
}