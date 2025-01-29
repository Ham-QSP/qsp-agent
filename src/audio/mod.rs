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

use std::sync::Arc;

use bytes::Bytes;
use tokio::time::Duration;

pub struct AudioFrame {
    pub(crate) data: Arc<Vec<f32>>,
}

pub struct AudioEncodedFrame {
    pub bytes: Bytes,
    pub duration: Duration,
}
