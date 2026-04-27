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
    use super::*;
    use crate::hamlib;

    #[test]
    fn list_rigs() {
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
        let mut hamlib = hamlib::Hamlib::new();
        let rig = hamlib.rig_connect(1);
        assert!(rig.is_ok())
    }

    #[test]
    fn get_freq() {
        let mut hamlib = hamlib::Hamlib::new();
        let rig = hamlib.rig_connect(1).unwrap();
        rig.set_freq(0, 100.0);
        let freq = rig.get_freq(0).unwrap();

        assert_eq!(freq, 100.0);
    }
}
