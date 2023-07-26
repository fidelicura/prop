use std::fs::{File as RawFile, Permissions, Metadata, FileType};
use std::fmt::{Result as FmtResult, Formatter, Display};
use std::os::unix::prelude::PermissionsExt;
use time::{Duration, OffsetDateTime, UtcOffset};
use std::time::SystemTime;
use std::path::Path;



#[derive(Debug)]
pub(crate) struct File {
    name: String,
    size: String,
    permissions: [Option<FilePermissions>; 3],
    kind: FileKind,
    date: FileDate,
}

impl File {
    pub(crate) fn new(arg: &String) -> Self {
        let path = Path::new(arg.as_str());
        let raw_file = RawFile::open(path)
            .expect("failed to get data from file {raw_file:#?}");
        let metadata = raw_file.metadata()
            .expect("failed to get metadata from file {raw_file:#?}");

        let name = path
            .file_name()
            .map_or_else(|| "unknown".to_owned(), |val| val.to_str().unwrap().to_owned());
        let size = format!("{} MB", (metadata.len() as f32 / 1024. / 1024.));
        let permissions = FilePermissions::new(metadata.permissions());
        let kind = FileKind::from(metadata.file_type());
        let date = FileDate::from(metadata);

        Self { name, size, permissions, kind, date }
    }
}

impl Display for File {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let horizontal_line = "─".repeat(65);

        let name_line = format!("│ {:^63.63} │", self.name);
        let size_line = format!("│ Size: {:<57} │", self.size);
        let permissions = self.permissions
            .iter()
            .map(|perm| match perm {
                Some(perm) => format!("{}", perm),
                None => "".to_string(),
            })
            .collect::<Vec<_>>()
            .join("+");
        let permissions_line = format!("│ Permissions: {:<50} │", permissions);
        let kind_line = format!("│ Type: {:<57} │", self.kind);
        let created_line = format!("│ Created: {:<54} │", self.date.created);
        let modified_line = format!("│ Modified: {:<53} │", self.date.modified);
        let accessed_line = format!("│ Accessed: {:<53} │", self.date.accessed);


        write!(f, "╭{}╮\n", horizontal_line)?;
        write!(f, "{}\n", name_line)?;
        write!(f, "├{}┤\n", horizontal_line)?;

        write!(f, "{}\n", size_line)?;
        write!(f, "{}\n", permissions_line)?;
        write!(f, "{}\n", kind_line)?;
        write!(f, "{}\n", created_line)?;
        write!(f, "{}\n", modified_line)?;
        write!(f, "{}\n", accessed_line)?;
        write!(f, "╰{}╯", horizontal_line)
    }
}

#[derive(Debug)]
pub(crate) enum FilePermissions {
    Read,
    Write,
    Executable,
}

impl FilePermissions {
    pub(crate) fn new(value: Permissions) -> [Option<Self>; 3] {
        match value.readonly() {
            true => {
                let executableness = Self::is_executable(value.mode());
                match executableness {
                    true => [Some(FilePermissions::Read), None, Some(FilePermissions::Executable)],
                    false => [Some(FilePermissions::Read), None, None],
                }
            },
            false => {
                let executableness = Self::is_executable(value.mode());
                match executableness {
                    true => [Some(FilePermissions::Read), Some(FilePermissions::Write), Some(FilePermissions::Executable)],
                    false => [Some(FilePermissions::Read), Some(FilePermissions::Write), None],
                }
            },
        }
    }

    fn is_executable(value: u32) -> bool {
        value & 0o111 != 0
    }
}

impl Display for FilePermissions {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Read => write!(f, "Read"),
            Self::Write => write!(f, "Write"),
            Self::Executable => write!(f, "Executable"),
        }
    }
}

#[derive(Debug)]
pub(crate) enum FileKind {
    Regular,
    Folder,
    Symlink,
}

impl From<FileType> for FileKind {
    fn from(value: FileType) -> Self {
        if value.is_file() {
            FileKind::Regular
        } else if value.is_dir() {
            FileKind::Folder
        } else if value.is_symlink() {
            FileKind::Symlink
        } else {
            unreachable!("is your file from other universe? you've reached unreachable!")
        }
    }
}

impl Display for FileKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Regular => write!(f, "Regular"),
            Self::Folder => write!(f, "Folder"),
            Self::Symlink => write!(f, "Symlink"),
        }
    }
}

#[derive(Debug)]
pub(crate) struct FileDate {
    created: String,
    modified: String,
    accessed: String,
}

impl FileDate {
    fn parse_time(systime: SystemTime, epoch: OffsetDateTime) -> String {
        let utc = epoch + Duration::try_from(systime.duration_since(epoch.into()).unwrap())
            .unwrap();
        let local = utc.to_offset(UtcOffset::local_offset_at(utc).unwrap());

        let date = local.date();
        let time = local.time();

        format!("{} {}", date, time)
    }
}

impl From<Metadata> for FileDate {
    fn from(value: Metadata) -> Self {
        let epoch = OffsetDateTime::UNIX_EPOCH;

        let created = value
            .created()
            .map_or_else(|_| "unknown".to_string(), |val| Self::parse_time(val, epoch));
        let modified = value
            .modified()
            .map_or_else(|_| "unknown".to_string(), |val| Self::parse_time(val, epoch));
        let accessed = value
            .modified()
            .map_or_else(|_| "unknown".to_string(), |val| Self::parse_time(val, epoch));

        Self { created, modified, accessed }
    }
}
