mod queue_handling;

use bitcoin::secp256k1::PublicKey;
use maybenot::{MachineId, TriggerEvent};
use nfq::{Queue, Verdict};
use once_cell::sync::OnceCell;
use rand::{
	rngs::{adapter::ReseedingRng, OsRng},
	SeedableRng,
};
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};
use std::{
	fs, io,
	path::Path,
	sync::{Arc, Mutex},
	time::{Duration, Instant},
};

use crate::ChannelManager;

type Rng = ReseedingRng<rand_chacha::ChaCha12Core, OsRng>;
const RNG_RESEED_THRESHOLD: u64 = 1024 * 64; // 64 KiB

/// Maximum number of events that can be stored in the underlying buffer, faster if power of two
/// Increase the capacity events bother runs out of space.
pub const EVENTS_CAPACITY: usize = 8;
type EventBuffer = ConstGenericRingBuffer<TriggerEvent, EVENTS_CAPACITY>;

/// Default queue number to use
const DEFAULT_QUEUE: u16 = 1;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	/// Failed to find maybenot machines
	#[error("Failed to enumerate maybenot machines")]
	EnumerateMachines(#[source] io::Error),
	/// Failed to parse maybenot machine
	#[error("Failed to parse maybenot machine \"{0}\"")]
	InvalidMachine(String),
	/// Failed to initialize maybenot framework
	#[error("Failed to initialize maybenot framework: {0}")]
	InitializeMaybenot(String),
}

pub struct Session {
	counterparty_node_id: PublicKey,
	channel_manager: Arc<ChannelManager>,
	event_buffer: EventBuffer,
}

impl Session {
	pub fn new(channel_manager: Arc<ChannelManager>) -> io::Result<Session> {
		let counterparty_node_id = channel_manager.list_channels()[0].counterparty.node_id;
		let event_buffer = ConstGenericRingBuffer::<TriggerEvent, { EVENTS_CAPACITY }>::new();
		Ok(Self { counterparty_node_id, channel_manager, event_buffer })
	}

	/// Sends a padding message to the peer's node.
	pub fn send_padding(&mut self, machine: MachineId) -> io::Result<()> {
		self.channel_manager.send_padding_message(&self.counterparty_node_id)?;
		println!("Sent padding message");
		Ok(self.event_buffer.push(TriggerEvent::PaddingSent { machine }))
	}
}

pub struct Machinist {
	session: Arc<Mutex<Session>>,
	// machine_ids: MachineMap,
	tokio_handle: tokio::runtime::Handle,
}

impl Machinist {
	/// Spawn an actor that handles scheduling of maybenot actions and forwards maybenot events to the framework.
	pub fn spawn(
		resource_dir: &Path, session: Session,
	) -> std::result::Result<std::thread::JoinHandle<()>, Error> {
		const MAX_PADDING_BYTES: f64 = 0.0;
		const MAX_BLOCKING_BYTES: f64 = 0.0;

		static MAYBENOT_MACHINES: OnceCell<Vec<maybenot::Machine>> = OnceCell::new();

		let machines =
			MAYBENOT_MACHINES.get_or_try_init(|| load_maybenot_machines(resource_dir))?;

		let framework = maybenot::Framework::new(
			machines.clone(),
			MAX_PADDING_BYTES,
			MAX_BLOCKING_BYTES,
			std::time::Instant::now(),
			Rng::new(rand_chacha::ChaCha12Core::from_entropy(), RNG_RESEED_THRESHOLD, OsRng),
		)
		.map_err(|error| Error::InitializeMaybenot(error.to_string()))?;

		let session = Arc::new(Mutex::new(session));
		let tokio_handle = tokio::runtime::Handle::current();

		Ok(std::thread::spawn(move || {
			Self { session, tokio_handle }.event_loop(framework);
		}))
	}

	/// Adds a new event to the buffer
	pub fn add_event(&mut self, event: TriggerEvent) -> io::Result<()> {
		let cloned_session = self.session.clone();
		let mut share_session = cloned_session.lock().unwrap();
		Ok(share_session.event_buffer.push(event))
	}

	/// Loop that runs continously to check for new events in the buffer
	fn event_loop(mut self, mut framework: maybenot::Framework<Vec<maybenot::Machine>, Rng>) {
		// temporary workaround: use nfq + iptables rule to get notified when a packet is sent on the node's port
		// create netfilter rules by running the `queue-setup.sh` script before executing this
		let mut nqueue = Queue::open().unwrap();
		nqueue.bind(DEFAULT_QUEUE).unwrap();
		nqueue.set_nonblocking(true);
		loop {
			match nqueue.recv() {
				Ok(mut msg) => {
					// TODO: how to prevent padding messages from triggering as well?
					if msg.get_original_len() == 1538 {
						println!("Received a message from nfq");
						self.add_event(TriggerEvent::NormalRecv).unwrap();
					}
					msg.set_verdict(Verdict::Accept);
					nqueue.verdict(msg).unwrap();
				},
				// Errors are expected, since this returns an error every time queue.recv() gets calle on empty queue
				Err(_e) => {},
			}
			let events = match self.wait_for_events() {
				Ok(opt_events) => {
					// println!("Ok(events)");
					match opt_events {
						Some(events) => {
							if events.is_empty() {
								break;
							}
							events
						},
						None => continue,
					}
				},
				Err(error) => {
					println!("Error(events)");
					log::error!("Error while waiting for maybenot events: {error}");
					break;
				},
			};

			for action in framework.trigger_events(&events, Instant::now()) {
				println!("Action: {:?}", action);
				self.handle_action(action);
			}
		}

		log::debug!("Stopped defense event loop");
	}

	/// Execute the action returned by the machine
	fn handle_action(&mut self, action: &maybenot::action::TriggerAction) {
		match *action {
			maybenot::action::TriggerAction::SendPadding {
				timeout,
				bypass: _,
				replace: _,
				machine,
			} => {
				let cloned_session = self.session.clone();
				if timeout == Duration::ZERO {
					let mut share_session = cloned_session.lock().unwrap();
					if let Err(error) = share_session.send_padding(machine) {
						log::error!("Failed to send maybenot action: {error}");
					}
				} else {
					// Schedule action on the tokio runtime
					let _task = self.tokio_handle.spawn(async move {
						tokio::time::sleep(timeout).await;

						let mut share_session = cloned_session.lock().unwrap();
						if let Err(error) = share_session.send_padding(machine) {
							log::error!("Failed to send Maybenot action: {error}");
						}
					});
				}
			},
			maybenot::action::TriggerAction::Cancel { .. } => {
				// FUTURE: implement ability to cancel scheduled actions waiting for timeout
				if cfg!(debug_assertions) {
					unimplemented!("received Cancel action");
				}
			},
			maybenot::action::TriggerAction::BlockOutgoing { .. } => {
				if cfg!(debug_assertions) {
					unimplemented!("received BlockOutgoing action");
				}
			},
			maybenot::action::TriggerAction::UpdateTimer { .. } => {
				if cfg!(debug_assertions) {
					unimplemented!("received UpdateTimer action");
				}
			},
		}
	}

	/// Take all events from the ring buffer while there are any left.
	/// If there are no events available, wait for events to arrive.
	/// Otherwise, break and return a non-zero number of events to be processed.
	/// If the quit event was signaled, this returns an empty vector.
	fn wait_for_events(&mut self) -> io::Result<Option<Vec<maybenot::TriggerEvent>>> {
		let cloned_session = self.session.clone();
		let mut share_session = cloned_session.lock().unwrap();

		loop {
			let events: Vec<_> = share_session.event_buffer.drain().collect();
			if !events.is_empty() {
				return Ok(Some(events));
			} else {
				return Ok(None);
			}
		}
	}
}

// Read serialized maybenot machines from the machine file
fn load_maybenot_machines(resource_dir: &Path) -> Result<Vec<maybenot::Machine>, Error> {
	let path = resource_dir.join("machines");
	log::debug!("Reading maybenot machines from {}", path.display());

	let mut machines = vec![];
	let machines_str = fs::read_to_string(path).map_err(Error::EnumerateMachines)?;
	for machine_str in machines_str.lines() {
		let machine_str = machine_str.trim();
		if matches!(machine_str.chars().next(), None | Some('#')) {
			continue;
		}
		log::debug!("Adding maybenot machine: {machine_str}");
		machines.push(
			machine_str
				.parse::<maybenot::Machine>()
				.map_err(|_error| Error::InvalidMachine(machine_str.to_owned()))?,
		);
	}
	Ok(machines)
}

#[cfg(test)]
mod test {
	use super::load_maybenot_machines;
	use std::path::PathBuf;

	/// Test whether `machines` in maybenot_machines contains valid machines.
	#[test]
	fn test_load_maybenot_machines() {
		let resource_dir = std::env::var("CARGO_MANIFEST_DIR")
			.map(PathBuf::from)
			.expect("CARGO_MANIFEST_DIR env var not set")
			.join("maybenot_machines");

		load_maybenot_machines(&resource_dir).unwrap();
	}
}
