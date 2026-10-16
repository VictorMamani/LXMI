use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemInfo {
    pub os: String,
    pub architecture: String,
    pub home_directory: Option<PathBuf>,
    pub xdg_data_home: Option<PathBuf>,
    pub xdg_data_dirs: Vec<PathBuf>,
}

impl SystemInfo {
    pub fn current() -> Self {
        let home_directory = env::var_os("HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute());
        let xdg_data_home = env::var_os("XDG_DATA_HOME").map(PathBuf::from);
        let xdg_data_dirs = env::var_os("XDG_DATA_DIRS").map(PathBuf::from);

        Self::from_values(home_directory, xdg_data_home, xdg_data_dirs)
    }

    fn from_values(
        home_directory: Option<PathBuf>,
        xdg_data_home: Option<PathBuf>,
        xdg_data_dirs: Option<PathBuf>,
    ) -> Self {
        let home_directory = home_directory.filter(|path| path.is_absolute());
        let xdg_data_home = xdg_data_home.filter(|path| path.is_absolute()).or_else(|| {
            home_directory
                .as_ref()
                .map(|home| home.join(".local/share"))
        });

        let xdg_data_dirs = match xdg_data_dirs.filter(|value| !value.as_os_str().is_empty()) {
            Some(value) => value
                .to_string_lossy()
                .split(':')
                .map(PathBuf::from)
                .filter(|path| path.is_absolute())
                .collect(),
            None => ["/usr/local/share", "/usr/share"]
                .into_iter()
                .map(PathBuf::from)
                .collect(),
        };

        Self {
            os: std::env::consts::OS.to_owned(),
            architecture: std::env::consts::ARCH.to_owned(),
            home_directory,
            xdg_data_home,
            xdg_data_dirs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SystemInfo;
    use std::path::PathBuf;

    #[test]
    fn uses_xdg_defaults_when_environment_values_are_missing() {
        let info = SystemInfo::from_values(Some(PathBuf::from("/home/tester")), None, None);

        assert_eq!(
            info.xdg_data_home,
            Some(PathBuf::from("/home/tester/.local/share"))
        );
        assert_eq!(
            info.xdg_data_dirs,
            vec![
                PathBuf::from("/usr/local/share"),
                PathBuf::from("/usr/share")
            ]
        );
    }

    #[test]
    fn ignores_relative_home_and_xdg_paths() {
        let info = SystemInfo::from_values(
            Some(PathBuf::from("relative-home")),
            Some(PathBuf::from("relative-data")),
            Some(PathBuf::from("relative:/usr/share")),
        );

        assert_eq!(info.home_directory, None);
        assert_eq!(info.xdg_data_home, None);
        assert_eq!(info.xdg_data_dirs, vec![PathBuf::from("/usr/share")]);
    }

    #[test]
    fn treats_an_empty_xdg_data_dirs_value_as_unset() {
        let info = SystemInfo::from_values(
            Some(PathBuf::from("/home/tester")),
            None,
            Some(PathBuf::new()),
        );

        assert_eq!(
            info.xdg_data_dirs,
            vec![
                PathBuf::from("/usr/local/share"),
                PathBuf::from("/usr/share")
            ]
        );
    }
}
