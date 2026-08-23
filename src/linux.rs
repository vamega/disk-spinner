#[cfg(target_os = "linux")]
#[cfg(not(feature = "udev"))]
mod platform_specific;
#[cfg(target_os = "linux")]
#[cfg(feature = "udev")]
mod platform_specific_udev;
use std::path::PathBuf;

#[cfg(target_os = "linux")]
#[cfg(not(feature = "udev"))]
pub(crate) use platform_specific::*;
#[cfg(target_os = "linux")]
#[cfg(feature = "udev")]
pub(crate) use platform_specific_udev::*;

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn child_partitions(
    device_name: &str,
    block_partitions: impl Iterator<Item = PathBuf>,
) -> Vec<PathBuf> {
    block_partitions
        .filter(|part_path| {
            part_path
                .file_name()
                .and_then(|name| {
                    let name = name.to_string_lossy();
                    let suffix = name.strip_prefix(device_name)?;
                    if device_name
                        .as_bytes()
                        .last()
                        .is_some_and(u8::is_ascii_digit)
                    {
                        suffix.strip_prefix('p')?.parse::<usize>().ok()
                    } else {
                        suffix.parse::<usize>().ok()
                    }
                    .map(|_| true)
                })
                .unwrap_or(false)
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use test_case::test_case;

    #[test_case("sda", &["/dev/sdb1", "/dev/sda1"], &["/dev/sda1"]; "a normal partition of the given device")]
    #[test_case("sda", &["/dev/sdb1"], &[]; "no-partitions")]
    #[test_case("sda", &["/dev/sda", "/dev/sdb", "/dev/sdai"], &[]; "block devices above 26 are present")]
    #[test_case("nvme0n1", &["/dev/nvme0n1p1", "/dev/nvme0n11"], &["/dev/nvme0n1p1"]; "nvme partitions use a p separator")]
    fn detects_child_partitions(dev: &str, existing: &[&str], should: &[&str]) {
        let detected = child_partitions(dev, existing.iter().map(PathBuf::from));
        let should: Vec<PathBuf> = should.iter().map(PathBuf::from).collect();
        assert_eq!(detected, should);
    }
}
