use event_listener::{Event, Listener};
use maybenot::{action, MachineId};
use maybenot::{Framework, Machine, TriggerAction, TriggerEvent};
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use crate::ChannelManager;
use crate::PeerManager;

/// Maximum number of events that can be stored in the underlying buffer, faster if power of two
pub const EVENTS_CAPACITY: usize = 128;

// let event = Arc::new(Event::new());
// let event = event.clone();

pub async fn event_loop(
	peer_manager: Arc<PeerManager>, channel_manager: Arc<ChannelManager>,
	mut event_buffer: ConstGenericRingBuffer<TriggerEvent, EVENTS_CAPACITY>, event: Arc<Event>,
) {
	tokio::time::sleep(Duration::from_secs(3)).await;
	// std::thread::sleep(Duration::from_secs(3));
	let peer_pubkey = "0296a55b43139bace6217bac4b68f852f302760b3cf338fbbb573fce5af4ed09cb";
	let peer_pubkey = match bitcoin::secp256k1::PublicKey::from_str(peer_pubkey) {
		Ok(pubkey) => pubkey,
		Err(e) => {
			println!("ERROR: {}", e.to_string());
			return;
		},
	};
	let s = "02eNqV2AlQVHUcwPFdEHZZlktwuUEQQUTlUANE5cdgooVgiOMgeDEe00ha2lRCxmWojVIKiaDYhGFOqCmNpJMj9ZajZTni3DiEQMApPFggTpHy/en3/Ds10//NMMN+mT+f2f97//++fZNT04fo5WPx3z9isVgkSgp84Zfp4/lLsehZ/s7YhD7QKcjLUU34BuFf+T4B4umX/dPj1oLoXw/hn4vFOv8wvDB5MLvwwnwNlNZsKbCsmYMC6Q8YBB0UdCnh/IYHbdtM6kBvUOaeGeiAAuldDIIuCjMoYfHGwDSVthrG6t56ErnVGgXS2xmEGSjoUcLwTd0zFUVq+CNZujw4QIEC6a0Mgh4K+pQQUzMQoI5XgSI5pvjNSXMUSG9mEPRRkFCCZtBnWVhTOQT5Np58FDITBdI1DIIEBSklVK3I8v4pswy+l304urvPFAXSmxgEKQoGlLBm7yfawLOlYCvrd3o/2wQF0hsZBAMUZJQgyrkStae6BKLfjWpYXWQkrDi+NzAIMhQMKeGh4cchAR4lEFqlbve0lKNAOotgiIKcEpR7jWWnLykhsOOSjf5VGQqk1zMIchSM6DNtFRNdsEwJd4fWpg7FGQhnmu8sghEKxpTg2JHz7db7HBwKeT1BJ1yKAul1DIIxCiaU8On1BZHncjlY/+sJjcc6CQqkswgmKJhSwjv+iWGWeziYe7y/qTJWHwXSWQRTFMwoIeG1R742KznY5JW3AU7ooUA6i2CGwkx6PUi3n4hy4kAaEeKdWzZDWA98ZxFmomBOCZ0DP8NvJhxUprV1HDURBNJZBHMULOhZ8nLtnZRycCfc1M1sl64wS3xnESxQmEUJ7qc5SYCcg5Q87WBuqQ4KpLMIs1BQ0LO0pHrdG9YcxH5xcVejpyCQziIoULCk91ZN0nuuXhzsu3VgVUaeWNhb+c4iWKJgRa9p/9FTqvUcNGka8u/ZCQLpLIIVCtaUIPvhqseBBA58d3Z3tuWKUCCdRbBGwYb+FJ26EZB8nYO1ZarARhdBIJ1FsEHB9qW9tbIztZ+D9P3HGoJjp0DYW593FsEWBTtKuHbU3txvuRIu294v2nHkGY4nnWX3tkPBnp6l8riTGZlKkKdulkxdm8TxpLMI9ig40LMUWZhVNa6EoKasCc/Op8Is8Z1FcEDBkb6Win/ZEvd2CbxS/zjK3kYQSGf5nHZEYTa9a/iNWm0aKQHv6G1hoqgJHE86izAbBSdK2J1joHp8qhQOcZFfO+SO43jSWe6XnFBwpt/DZ1ecvYLLwHjJxdDynjHhPfCd5Z7PGYU5lGDhHP5dl3E5pPiNcmE+gkA6y33rHBRc6L017fi+1rFy8Kk4e8wwZRTHk84iuKAwl14PiVUJMTIVjI+YySMaRoT1wHeWu/u5KLjS9xq3syLz1lTAj91tvfvnCQLpLQyCKwpulBBxeTD0drEadm2Xt+cfHsbxpLN8B3JDYR4l3Nqcraf+qAqSkr/pdqz/E8eTfo9BmIeCO/0tKySxetm5Gjhe2976u5sgkN7BILijMJ++liLMU+1sakGtb9jWHj8kXEt872QQ5qPgQZ+HI3A0404dFB8s70+vGBTOA99Zvk97oLCA3jXOH1L3tDTAl34Kd/NZgkB6N4OwAIWFlPBVaMdmzU0NLB11jb27aQDHk87y1GEhCovofSkz59WUsla4sXGpRVK6VtiX+N7HICxCwZMSVuw02u8u6YTD0RdaEgv7cTzpWgbBEwUv+modqV0d/nkvSPx3KOLjnghXK9+HGQQvFLzpfak57UyzUgsre2IU8X0PhX2J7+Kg/y94o+BDCQu1/nEfpFsEZXW5Tqwq7MXxpLu8IATzD8nW/Kfgg8JT8jSNKH8B7z8Nrw==";
	let m = vec![Machine::from_str(s).unwrap()];
	let mut f: Framework<&Vec<Machine>, rand::prelude::ThreadRng> =
		Framework::new(&m, 0.0, 0.0, Instant::now(), rand::thread_rng()).unwrap();

	// whait for notification if there are no events in the buffer
	if event_buffer.is_empty() {
		println!("Event buffer is empty!");
		let listener = event.listen();
		listener.wait();
	}

	loop {
		// this loop should only run if there are events in the buffer to process
		let listener = event.listen();
		println!("Before: {:?}", event_buffer.len());
		// println!("{:?}", event_buffer.drain().collect::<Vec<_>>());
		let actions: Vec<&TriggerAction> =
			f.trigger_events(event_buffer.drain(), Instant::now()).collect();
		println!("After: {:?}", event_buffer.len());
		println!("Actions: {:?}", actions);
		for action in actions {
			match action {
				TriggerAction::SendPadding { timeout, bypass: _, replace: _, machine } => {
					if do_send_padding_message(
						peer_pubkey,
						peer_manager.clone(),
						channel_manager.clone(),
						timeout,
					)
					.is_ok()
					{
						println!("SUCCESS: sent padding message to peer {}", peer_pubkey);
						// TODO: have structure to manage multiple machines
						event_buffer.push(TriggerEvent::PaddingSent { machine: *machine });
						println!("{:?}", event.notify(1));
					}
				},
				TriggerAction::Cancel { machine: _, timer: _ } => {},
				TriggerAction::BlockOutgoing {
					timeout: _,
					duration: _,
					bypass: _,
					replace: _,
					machine: _,
				} => {},
				TriggerAction::UpdateTimer { duration: _, replace: _, machine: _ } => {},
			}
		}
		// break;
		// println!("{:?}", event_buffer);
		// Wait for a notification and continue the loop.
		listener.wait();
	}
}

fn do_send_padding_message(
	pubkey: bitcoin::secp256k1::PublicKey, peer_manager: Arc<PeerManager>,
	channel_manager: Arc<ChannelManager>,
	timeout: &<std::time::Instant as maybenot::time::Instant>::Duration,
) -> Result<(), ()> {
	// check the pubkey matches a valid connected peer
	// TODO: maybe move check outside of function?
	if peer_manager.peer_by_node_id(&pubkey).is_none() {
		println!("Error: Could not find peer {}", pubkey);
		return Err(());
	}

	// sleep for duration of timout before sending
	std::thread::sleep(*timeout);
	// tokio::time::sleep(*timeout).await;
	channel_manager.send_padding_message(&pubkey);
	Ok(())
}
