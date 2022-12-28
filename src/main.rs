/*
This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License,
or (at your option) any later version.

This program is distributed in the hope that it will be useful, but
WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>
 */

use std::env;

mod signaling;

use crate::signaling::signaling_server_connection::SignalingServerManager;

const APPLICATION_VERSION: &'static str = "0.1.0";
const AGENT_TYPE_NAME: &'static str = "F4FEZ Agent";

#[tokio::main]
async fn main() {
    let connect_addr =
        env::args().nth(1).unwrap_or_else(|| panic!("this program requires as argument the signaling server url"));

    let url = url::Url::parse(&connect_addr).unwrap();

    let mut signal_server_session = SignalingServerManager::new();
    signal_server_session.start(url).await.expect("Can't start. Failed to connect the signaling server");
}
