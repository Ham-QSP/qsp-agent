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
mod errors;
pub mod hamlib;
mod hamlib_raw;
pub mod rig;

#[cfg(test)]
mod tests {
    use crate::hamlib;
    use std::collections::HashMap;
    use std::sync::{Mutex, MutexGuard, OnceLock};

    fn hamlib_test_guard() -> MutexGuard<'static, ()> {
        static HAMLIB_TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

        HAMLIB_TEST_MUTEX
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap()
    }

    #[test]
    fn list_rigs() {
        let _guard = hamlib_test_guard();
        let mut hamlib = hamlib::Hamlib::new();
        hamlib.list_rigs({
            |caps| {
                println!(
                    "RIG: {} {} ({:?})",
                    caps.manufacturer_name, caps.model_name, caps.rig_model
                );
            }
        });
    }

    #[test]
    fn open_rig() {
        let _guard = hamlib_test_guard();
        let mut hamlib = hamlib::Hamlib::new();
        let rig = hamlib.rig_connect(1, HashMap::new());
        assert!(rig.is_ok())
    }

    #[test]
    fn get_freq() {
        let _guard = hamlib_test_guard();
        let mut hamlib = hamlib::Hamlib::new();
        let rig = hamlib.rig_connect(1, HashMap::new()).unwrap();
        rig.set_freq(0, 100.0);
        let freq = rig.get_freq(0).unwrap();

        assert_eq!(freq, 100.0);
    }
}
