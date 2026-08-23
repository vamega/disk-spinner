use super::child_partitions;
use crate::Args;
use aligned_buffer::UniqueAlignedBuffer;
use anyhow::Context as _;
use std::{
    ffi::OsStr,
    fs,
    os::unix::fs::FileTypeExt as _,
    path::{Path, PathBuf},
    str::FromStr,
};
use tracing::warn;

pub type IOBuffer = UniqueAlignedBuffer<4096>;

pub const OPEN_FLAGS: i32 = libc::O_DIRECT | libc::O_EXCL;

#[derive(Debug, Clone)]
pub(crate) struct ValidDevice {
    pub path: PathBuf,
    pub partition: Option<u64>,
    pub device: DeviceMetadata,
}

#[derive(Debug, Clone)]
pub(crate) struct DeviceMetadata {
    pub name: String,
    pub media_type: MediaType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum MediaType {
    SolidState,
    Rotational,
    Loopback,
    MdRaid,
    Nvme,
    Ram,
    Unknown,
}

impl FromStr for ValidDevice {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let path = PathBuf::from(s);
        let metadata = fs::metadata(&path)
            .with_context(|| format!("Reading metadata for device path {path:?}"))?;
        if !metadata.file_type().is_block_device() {
            anyhow::bail!("The device under test must be a valid block device.");
        }

        let name = device_name_from_path(&path)?;
        let sysfs_path = sysfs_block_path(&name);
        if !sysfs_path.exists() {
            anyhow::bail!("The device under test must be a valid block device.");
        }

        let partition = read_partition_number(&sysfs_path)?;
        let media_sysfs_path = if partition.is_some() {
            parent_sysfs_block_path(&sysfs_path)?
        } else {
            sysfs_path
        };
        let media_type = get_media_type(&name, &media_sysfs_path)?;

        Ok(Self {
            path,
            partition,
            device: DeviceMetadata { name, media_type },
        })
    }
}

pub(crate) fn sanity_checks(
    args: &Args,
    partition: Option<u64>,
    device_path: &Path,
    device: &DeviceMetadata,
) -> anyhow::Result<()> {
    // Sanity checks:
    if partition.is_some() {
        if !args.allow_any_block_device {
            anyhow::bail!("Device is not a whole disk but a partition - pass --allow-any-block-device to run tests anyway.");
        } else {
            warn!(
                ?partition,
                ?device_path,
                "Testing a partition but running tests anyway."
            );
        }
    }
    if device.media_type != MediaType::Rotational {
        if !args.allow_any_media {
            anyhow::bail!("Device is not a rotational disk - this tool may be harmful to solid-state drives and others! Pass --allow-any-media to run anyway.");
        } else {
            warn!(?device.media_type, ?device_path, "Media type is not as expected but running tests anyway.");
        }
    }
    let child_partitions = if partition.is_none() {
        child_partitions(&device.name, get_block_partitions()?)
    } else {
        Vec::new()
    };

    if !child_partitions.is_empty() {
        anyhow::bail!("Detected child partitions on the device - I won't help you destroy an in-use drive: Delete those partitions yourself. Partitions found: {child_partitions:?}", );
    }
    Ok(())
}

fn device_name_from_path(path: &Path) -> anyhow::Result<String> {
    let canonical =
        fs::canonicalize(path).with_context(|| format!("Resolving device path {path:?}"))?;
    canonical
        .file_name()
        .and_then(OsStr::to_str)
        .map(str::to_owned)
        .ok_or_else(|| anyhow::anyhow!("Device path {path:?} has no device name"))
}

fn sysfs_block_path(device_name: &str) -> PathBuf {
    Path::new("/sys/class/block").join(device_name)
}

fn parent_sysfs_block_path(sysfs_path: &Path) -> anyhow::Result<PathBuf> {
    let canonical = fs::canonicalize(sysfs_path)
        .with_context(|| format!("Resolving sysfs block path {sysfs_path:?}"))?;
    let parent_name = canonical
        .parent()
        .and_then(Path::file_name)
        .and_then(OsStr::to_str)
        .ok_or_else(|| anyhow::anyhow!("Sysfs block path {sysfs_path:?} has no parent device"))?;

    Ok(sysfs_block_path(parent_name))
}

fn read_partition_number(sysfs_path: &Path) -> anyhow::Result<Option<u64>> {
    let partition_path = sysfs_path.join("partition");
    if !partition_path.exists() {
        return Ok(None);
    }

    let partition = fs::read_to_string(&partition_path)
        .with_context(|| format!("Reading partition metadata from {partition_path:?}"))?
        .trim()
        .parse()
        .with_context(|| format!("Parsing partition metadata from {partition_path:?}"))?;
    Ok(Some(partition))
}

fn get_media_type(device_name: &str, sysfs_path: &Path) -> anyhow::Result<MediaType> {
    if device_name.starts_with("loop") {
        return Ok(MediaType::Loopback);
    }
    if device_name.starts_with("ram") {
        return Ok(MediaType::Ram);
    }
    if device_name.starts_with("md") {
        return Ok(MediaType::MdRaid);
    }
    if device_name.starts_with("nvme") {
        return Ok(MediaType::Nvme);
    }

    let rotational_path = sysfs_path.join("queue/rotational");
    if !rotational_path.exists() {
        return Ok(MediaType::Unknown);
    }

    match fs::read_to_string(&rotational_path)
        .with_context(|| format!("Reading rotational metadata from {rotational_path:?}"))?
        .trim()
    {
        "0" => Ok(MediaType::SolidState),
        "1" => Ok(MediaType::Rotational),
        _ => Ok(MediaType::Unknown),
    }
}

fn get_block_partitions() -> anyhow::Result<impl Iterator<Item = PathBuf>> {
    let mut partitions = Vec::new();
    for entry in fs::read_dir("/sys/class/block").context("Reading /sys/class/block")? {
        let entry = entry.context("Reading /sys/class/block entry")?;
        let path = entry.path();
        if path.join("partition").exists() {
            partitions.push(Path::new("/dev").join(entry.file_name()));
        }
    }

    Ok(partitions.into_iter())
}
