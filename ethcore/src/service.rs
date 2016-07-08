// Copyright 2015, 2016 Ethcore (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Creates and registers client and network services.

use util::*;
use util::panics::*;
use spec::Spec;
use error::*;
use client::{Client, ClientConfig, ChainNotify};
use miner::Miner;

/// Message type for external and internal events
#[derive(Clone)]
pub enum SyncMessage {
	/// Best Block Hash in chain has been changed
	NewChainHead,
	/// A block is ready
	BlockVerified,
	/// New transaction RLPs are ready to be imported
	NewTransactions(Vec<Bytes>),
}

/// Client service setup. Creates and registers client and network services with the IO subsystem.
pub struct ClientService {
	io_service: Arc<IoService<SyncMessage>>,
	client: Arc<Client>,
	panic_handler: Arc<PanicHandler>
}

impl ClientService {
	/// Start the service in a separate thread.
	pub fn start(
		config: ClientConfig,
		spec: Spec,
		db_path: &Path,
		miner: Arc<Miner>,
		) -> Result<ClientService, Error>
	{
		let panic_handler = PanicHandler::new_in_arc();
		let io_service = try!(IoService::<SyncMessage>::start());
		panic_handler.forward_from(&io_service);

		info!("Configured for {} using {} engine", spec.name.clone().apply(Colour::White.bold()), spec.engine.name().apply(Colour::Yellow.bold()));
		let client = try!(Client::new(config, spec, db_path, miner, io_service.channel()));
		panic_handler.forward_from(client.deref());
		let client_io = Arc::new(ClientIoHandler {
			client: client.clone()
		});
		try!(io_service.register_handler(client_io));

		Ok(ClientService {
			io_service: Arc::new(io_service),
			client: client,
			panic_handler: panic_handler,
		})
	}

	/// Add a node to network
	pub fn add_node(&mut self, _enode: &str) {
		unimplemented!();
	}

	/// Get general IO interface
	pub fn register_io_handler(&self, handler: Arc<IoHandler<SyncMessage> + Send>) -> Result<(), IoError> {
		self.io_service.register_handler(handler)
	}

	/// Get client interface
	pub fn client(&self) -> Arc<Client> {
		self.client.clone()
	}

	/// Get network service component
	pub fn io(&self) -> Arc<IoService<SyncMessage>> {
		self.io_service.clone()
	}

	/// Set the actor to be notified on certain chain events
	pub fn set_notify(&self, notify: &Arc<ChainNotify>) {
		self.client.set_notify(notify);
	}
}

impl MayPanic for ClientService {
	fn on_panic<F>(&self, closure: F) where F: OnPanicListener {
		self.panic_handler.on_panic(closure);
	}
}

/// IO interface for the Client handler
struct ClientIoHandler {
	client: Arc<Client>
}

const CLIENT_TICK_TIMER: TimerToken = 0;
const CLIENT_TICK_MS: u64 = 5000;

impl IoHandler<SyncMessage> for ClientIoHandler {
	fn initialize(&self, io: &IoContext<SyncMessage>) {
		io.register_timer(CLIENT_TICK_TIMER, CLIENT_TICK_MS).expect("Error registering client timer");
	}

	fn timeout(&self, _io: &IoContext<SyncMessage>, timer: TimerToken) {
		if timer == CLIENT_TICK_TIMER {
			self.client.tick();
		}
	}

	#[cfg_attr(feature="dev", allow(single_match))]
	fn message(&self, io: &IoContext<SyncMessage>, net_message: &SyncMessage) {
		match *net_message {
			SyncMessage::BlockVerified => { self.client.import_verified_blocks(&io.channel()); }
			SyncMessage::NewTransactions(ref transactions) => { self.client.import_queued_transactions(&transactions); }
			_ => {} // ignore other messages
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use tests::helpers::*;
	use devtools::*;
	use client::ClientConfig;
	use std::sync::Arc;
	use miner::Miner;

	#[test]
	fn it_can_be_started() {
		let temp_path = RandomTempPath::new();
		let service = ClientService::start(
			ClientConfig::default(),
			get_test_spec(),
			&temp_path.as_path(),
			Arc::new(Miner::with_spec(get_test_spec())),
		);
		assert!(service.is_ok());
	}
}
