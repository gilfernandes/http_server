use crate::args::{AuthMode};
use crate::RunCommand;

fn extract_basic_auth_folders(auth_folders: &String) -> Vec<&str> {
    let splits = auth_folders.split(",").map(|s| s.trim());
    return splits.collect::<Vec<&str>>()
}

pub(crate) fn process_basic_auth(uri: &String, run_args: &RunCommand) -> bool {
    let auth_mode = &run_args.auth_mode;
    match auth_mode {
        AuthMode::Basic(basic_auth_command) => {
            let protected_folders = &basic_auth_command.protected_folders;
            let folders_vec = extract_basic_auth_folders(protected_folders);
            let matches = folders_vec.iter().find(|&&s| uri.starts_with(s));
            return matches.is_some();
        }
        AuthMode::None(_) => {}
    }
    false
}

#[cfg(test)]
mod tests {
    use crate::args::{BasicAuthCommand, NoneAuthCommand};
    use super::*;

    #[test]
    fn when_extract_basic_auth_folders_should_extract_folders() {
        let str = &"/tmp/mdm-reports, /tmp/data ".to_string();
        let auth_folders = extract_basic_auth_folders(str);
        assert_eq!(auth_folders.len(), 2);
        assert_eq!(auth_folders[0], "/tmp/mdm-reports");
        assert_eq!(auth_folders[1], "/tmp/data");
    }

    #[test]
    fn when_process_basic_auth_should_not_process() {
        let uri = "/mdm-reports".to_string();
        let run_cmd = RunCommand {
            auth_mode: AuthMode::None(NoneAuthCommand{}),
            root_folder: "/tmp".to_string(),
            port: 80,
            host: "0.0.0.0".to_string(),
            pool_size: 4
        };
        assert_eq!(process_basic_auth(&uri, &run_cmd), false);
    }

    #[test]
    fn when_process_basic_auth_should_process() {
        let uri = "/mdm-reports".to_string();
        let run_cmd = RunCommand {
            auth_mode: AuthMode::Basic(BasicAuthCommand{
                protected_folders: uri.clone(),
                username: "root".to_string(),
                password: "test".to_string()
            }),
            root_folder: "/tmp".to_string(),
            port: 80,
            host: "0.0.0.0".to_string(),
            pool_size: 4
        };
        assert_eq!(process_basic_auth(&uri, &run_cmd), true);
    }

    fn basic_auth_factory(protected_folders: &String) -> AuthMode {
        AuthMode::Basic(BasicAuthCommand {
            protected_folders: protected_folders.clone(),
            username: "root".to_string(),
            password: "test".to_string()
        })
    }

    #[test]
    fn when_process_basic_auth_multiple_folders_should_process() {
        let uri = "/mdm-reports".to_string();
        let protected_folders = "/api,/test,/mdm-reports".to_string();
        let run_cmd = RunCommand {
            auth_mode: basic_auth_factory(&protected_folders),
            root_folder: "/tmp".to_string(),
            port: 80,
            host: "0.0.0.0".to_string(),
            pool_size: 4
        };
        assert_eq!(process_basic_auth(&uri, &run_cmd), true);
    }
}