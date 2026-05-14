#[allow(unused_imports)]
use tracing::{debug, error, info, instrument, trace, warn};

use gpui::*;
use gpui::{AppContext, Entity, Global, SharedString};
use reticulum::iface::RxMessage;
use tokio::sync::broadcast::error::RecvError;

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

const RING_CAPACITY: usize = 50_000;

// Strings and formatting

#[derive(Clone)]
pub struct FormattedPacket {
    pub hops: SharedString,
    pub destination: SharedString,
    pub interface: SharedString,
    pub transport: SharedString,
    // Badge fields use &'static str via SharedString::from_static — no heap alloc.
    pub context: SharedString,
    pub destination_type: SharedString,
    pub header_type: SharedString,
    pub propagation_type: SharedString,
    pub ifac_flag: SharedString,
}

impl FormattedPacket {
    pub fn new(msg: &RxMessage) -> Self {
        Self {
            hops: format!("{}", msg.packet.header.hops).into(),
            destination: msg.packet.destination.to_hex_string().into(),
            interface: msg.address.to_hex_string().into(),
            transport: msg
                .packet
                .transport
                .map_or_else(|| SharedString::from("—"), |a| a.to_hex_string().into()),
            // Badge fields: match to a static string, zero heap allocation.
            context: fmt_static_context(&msg.packet.context),
            destination_type: fmt_static_destination_type(&msg.packet.header.destination_type),
            header_type: fmt_static_header_type(&msg.packet.header.header_type),
            propagation_type: fmt_static_propagation_type(&msg.packet.header.propagation_type),
            ifac_flag: fmt_static_ifac_flag(&msg.packet.header.ifac_flag),
        }
    }
}

#[rustfmt::skip]
fn fmt_static_context(v: &reticulum::packet::PacketContext) -> SharedString {
    use reticulum::packet::PacketContext;
    SharedString::from(match v {
        PacketContext::None => "None",                           // Generic data packet
        PacketContext::Resource => "Resource",                   // Packet is part of a resource
        PacketContext::ResourceAdvertisement => "ResourceAdv",   // Packet is a resource advertisement
        PacketContext::ResourceRequest => "ResourceReq",         // Packet is a resource part request
        PacketContext::ResourceHashUpdate => "ResourceHmu",      // Packet is a resource hashmap update
        PacketContext::ResourceProof => "ResourcePrf",           // Packet is a resource proof
        PacketContext::ResourceInitiatorCancel => "ResourceIcl", // Packet is a resource initiator cancel message
        PacketContext::ResourceReceiverCancel => "ResourceRcl",  // Packet is a resource receiver cancel message
        PacketContext::CacheRequest => "CacheRequest",           // Packet is a cache request
        PacketContext::Request => "Request",                     // Packet is a request
        PacketContext::Response => "Response",                   // Packet is a response to a request
        PacketContext::PathResponse => "PathResponse",           // Packet is a response to a path request
        PacketContext::Command => "Command",                     // Packet is a command
        PacketContext::CommandStatus => "CommandStatus",         // Packet is a status of an executed command
        PacketContext::Channel => "Channel",                     // Packet contains link channel data
        PacketContext::KeepAlive => "Keepalive",                 // Packet is a keepalive packet
        PacketContext::LinkIdentify => "LinkIdentify",           // Packet is a link peer identification proof
        PacketContext::LinkClose => "LinkClose",                 // Packet is a link close message
        PacketContext::LinkProof => "LinkProof",                 // Packet is a link packet proof
        PacketContext::LinkRTT => "LrssProof",                   // Packet is a link request round-trip time measurement
        PacketContext::LinkRequestProof => "SymmetryBreaker",    // Packet is a link request proof
    })
}

fn fmt_static_destination_type(v: &reticulum::packet::DestinationType) -> SharedString {
    use reticulum::packet::DestinationType::*;
    SharedString::from(match v {
        Single => "Single",
        Group => "Group",
        Plain => "Plain",
        Link => "Link",
    })
}

fn fmt_static_header_type(v: &reticulum::packet::HeaderType) -> SharedString {
    use reticulum::packet::HeaderType::*;
    SharedString::from(match v {
        Type1 => "Type1",
        Type2 => "Type2",
    })
}

fn fmt_static_propagation_type(v: &reticulum::packet::PropagationType) -> SharedString {
    use reticulum::packet::PropagationType::*;
    SharedString::from(match v {
        Broadcast => "Broadcast",
        Transport => "Transport",
        Reserved1 => "Reserved1",
        Reserved2 => "Reserved2",
    })
}

fn fmt_static_ifac_flag(v: &reticulum::packet::IfacFlag) -> SharedString {
    use reticulum::packet::IfacFlag;
    SharedString::from(match v {
        IfacFlag::Open => "Open",
        IfacFlag::Authenticated => "Authenticated",
    })
}

// State

pub struct State {
    pub items: VecDeque<FormattedPacket>,
    pub pending: Arc<Mutex<Vec<RxMessage>>>,
    _subscription_task: Option<gpui::Task<()>>,
}

#[derive(Clone)]
pub struct StateModel {
    pub inner: Entity<State>,
}

impl State {
    fn drain_pending(&mut self) -> bool {
        let mut pending = self.pending.lock().unwrap();
        if pending.is_empty() {
            return false;
        }
        for msg in pending.drain(..) {
            if self.items.len() >= RING_CAPACITY {
                self.items.pop_front();
            }
            self.items.push_back(FormattedPacket::new(&msg));
        }
        true
    }
}

impl StateModel {
    pub fn init(cx: &mut App, channel: tokio::sync::broadcast::Sender<RxMessage>) {
        let pending = Arc::new(Mutex::new(Vec::new()));
        let pending_tokio = pending.clone();

        let inner = cx.new(|_| State {
            items: VecDeque::with_capacity(RING_CAPACITY),
            pending: pending.clone(),
            _subscription_task: None,
        });

        // Pure Tokio task — writes into the pending mutex.
        tokio::spawn(async move {
            let mut rx = channel.subscribe();
            loop {
                match rx.recv().await {
                    Ok(msg) => pending_tokio.lock().unwrap().push(msg),
                    Err(RecvError::Lagged(n)) => warn!("Receiver lagged by {n} messages"),
                    Err(RecvError::Closed) => break,
                }
            }
        });

        // GPUI timer task — drains pending into State every 50 ms.
        let model_handle = inner.clone();
        let listener_task = cx.spawn(async move |cx| {
            loop {
                cx.background_spawn(async {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                })
                .await;

                model_handle
                    .update(cx, |state, cx| {
                        if state.drain_pending() {
                            cx.notify();
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
