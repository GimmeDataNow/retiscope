// use gpui::AsyncAppContext;
use gpui::AsyncWindowContext;
use gpui::*;
use gpui::{AppContext, Context, Entity, Global, Pixels, Size, px, size};
use reticulum::iface::RxMessage;
use std::rc::Rc;
use tokio::sync::broadcast::{Receiver, Sender};

struct RxMessageStore(Entity<Vec<RxMessage>>);
impl Global for RxMessageStore {}

// use gpui::*;

// use crate::todo::TodoItem;

#[derive(Clone)]
pub struct State {
    pub items: Vec<RxMessage>,
}

#[derive(Clone)]
pub struct StateModel {
    pub inner: Entity<State>,
}

// impl StateModel {}o
// pub struct StateModel {
//     pub inner: Entity<State>,
// }

impl StateModel {
    // pub fn init(cx: &mut AppContext, channel: tokio::sync::broadcast::Sender<RxMessage>) {
    //     let model = cx.new(|_| State { items: vec![] });
    //     let this = Self { inner: model };
    //     cx.set_global(this.clone());

    //     // Clone once for the closure
    //     let this_model = this.clone();

    //     // 1. Use 'move' here to take ownership of this_model and rx
    //     cx.spawn(move |_view_handle, cx: &mut AsyncApp| {
    //         // 2. Clone 'cx' to get an OWNED AsyncApp instead of a reference.
    //         // This is the key to satisfying the 'static lifetime.
    //         let mut cx_owned = cx.clone();
    //         // let mut rx = rx.clone();
    //         let this_model = this_model.clone();

    //         let mut rx = channel.subscribe();

    //         async move {
    //             while let Ok(msg) = rx.recv().await {
    //                 // 3. Use the owned cx_owned inside the async block
    //                 this_model
    //                     .inner
    //                     .update::<(), AsyncApp>(&mut cx_owned, |state, cx| {
    //                         state.items.push(msg);
    //                         cx.notify();
    //                     })
    //                     .ok();
    //             }
    //         }
    //     })
    //     .detach();
    // }
    pub fn init(cx: &mut App, channel: tokio::sync::broadcast::Sender<RxMessage>) {
        let model = cx.new(|_| State { items: vec![] });
        let this = Self { inner: model };

        // This will now work because App implements UpdateGlobal
        cx.set_global(this.clone());

        let this_model = this.clone();

        // This will now work because App implements the spawn methods
        cx.spawn(move |cx: &mut AsyncApp| {
            let mut cx_owned = cx.clone();
            let this_model = this_model.clone();
            let mut rx = channel.subscribe();

            async move {
                while let Ok(msg) = rx.recv().await {
                    this_model
                        .inner
                        .update::<(), AsyncApp>(&mut cx_owned, |state, cx| {
                            state.items.push(msg);
                            cx.notify();
                        })
                        .ok();
                }
            }
        })
        .detach();
    }
}

impl Global for StateModel {}

#[derive(Clone, Debug)]
pub struct ListChangedEvent {}

impl EventEmitter<ListChangedEvent> for State {}
