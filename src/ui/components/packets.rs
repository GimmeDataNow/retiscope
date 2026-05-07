#[allow(unused_imports)]
use tracing::{debug, error, info, instrument, trace, warn};

use gpui::*;
use gpui::{AppContext, Entity, Global};
use reticulum::iface::RxMessage;
use tokio::sync::broadcast::error::RecvError;

use std::sync::{Arc, Mutex};

// #[derive(Clone)]
pub struct State {
    pub items: Vec<RxMessage>,
    pub pending: Arc<Mutex<Vec<RxMessage>>>,
    _subscription_task: Option<gpui::Task<()>>,
}

impl Drop for State {
    fn drop(&mut self) {
        println!("STATE IS BEING DROPPED!");
    }
}

#[derive(Clone)]
pub struct StateModel {
    pub inner: Entity<State>,
}

impl StateModel {
    pub fn init(cx: &mut App, channel: tokio::sync::broadcast::Sender<RxMessage>) {
        let pending = Arc::new(Mutex::new(Vec::new()));
        let pending_tokio = pending.clone();

        let inner = cx.new(|_| State {
            items: vec![],
            pending: pending.clone(),
            _subscription_task: None,
        });

        // Pure Tokio task - writes into the mutex
        tokio::spawn(async move {
            let mut rx = channel.subscribe();
            loop {
                match rx.recv().await {
                    Ok(msg) => {
                        pending_tokio.lock().unwrap().push(msg);
                    }
                    Err(RecvError::Lagged(n)) => warn!("Lagged by {n}"),
                    Err(RecvError::Closed) => break,
                }
            }
        });

        let model_handle = inner.clone();

        // GPUI timer task - drains the mutex into State every 50ms
        let listener_task = cx.spawn(async move |mut cx| {
            let mut count = 0;
            loop {
                cx.background_spawn(async {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                })
                .await;

                model_handle
                    .update(cx, |state, cx| {
                        let mut pending = state.pending.lock().unwrap();
                        if !pending.is_empty() {
                            state.items.append(&mut pending);
                            cx.notify();
                            count += 1;
                            info!("count = {count}");
                        }
                    })
                    .ok();
            }
        });

        inner.update(cx, |state, _| {
            state._subscription_task = Some(listener_task);
        });

        cx.set_global(Self { inner });
    }
}

impl Global for StateModel {}

#[derive(Clone, Debug)]
pub struct ListChangedEvent {}

impl EventEmitter<ListChangedEvent> for State {}
