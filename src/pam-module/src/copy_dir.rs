use std::fs;
use std::path::PathBuf;

pub fn recursive_copy_dir_all<P>(
    src_dir: P,
    dest_dir: P,
    dest_owner_uid: Option<u32>,
    dest_owner_gid: Option<u32>,
) -> anyhow::Result<()>
where
    P: Into<PathBuf>,
{
    let src = src_dir.into();

    if !fs::exists(&src)? {
        return Err(anyhow::Error::msg(format!(
            "{src:?} doest not exist - skipping copy"
        )));
    }

    copy_dir_all(src, dest_dir.into(), dest_owner_uid, dest_owner_gid)
}

fn copy_dir_all(
    src: PathBuf,
    dest: PathBuf,
    dest_uid: Option<u32>,
    dest_gid: Option<u32>,
) -> anyhow::Result<()> {
    fs::create_dir_all(&dest)?;

    for entry in fs::read_dir(&src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dest.join(entry.file_name());

        if entry.metadata()?.is_dir() {
            copy_dir_all(from, to.clone(), dest_uid, dest_gid)?;
        } else {
            fs::copy(from, to.clone())?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::*;

            if dest_uid.is_some() || dest_gid.is_some() {
                chown(to, dest_uid, dest_gid)?;
            }
        }
    }

    Ok(())
}
