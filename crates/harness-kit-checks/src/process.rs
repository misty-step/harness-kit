use std::process::Command;

pub fn command(program: &str) -> Command {
    let mut command = Command::new(program);
    clear_git_local_env(&mut command);
    command
}

pub fn clear_git_local_env(command: &mut Command) {
    // Git exports these to hooks; child gates must rediscover the repo from cwd
    // or their own temp checkouts instead of inheriting the hook's gitdir.
    for key in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_COMMON_DIR",
        "GIT_CONFIG",
        "GIT_CONFIG_PARAMETERS",
        "GIT_CONFIG_COUNT",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_PREFIX",
        "GIT_IMPLICIT_WORK_TREE",
        "GIT_GRAFT_FILE",
        "GIT_NO_REPLACE_OBJECTS",
        "GIT_REPLACE_REF_BASE",
        "GIT_SHALLOW_FILE",
    ] {
        command.env_remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_strip_git_hook_environment() {
        let mut command = Command::new("sh");
        command
            .env("GIT_DIR", "/tmp/outer/.git")
            .env("GIT_WORK_TREE", "/tmp/outer")
            .env("GIT_CONFIG_COUNT", "1")
            .env("GIT_INDEX_FILE", "/tmp/outer/.git/index")
            .env("PATH", "/tmp/bin");

        clear_git_local_env(&mut command);

        let envs = command
            .get_envs()
            .map(|(key, value)| {
                (
                    key.to_string_lossy().to_string(),
                    value.map(|value| value.to_string_lossy().to_string()),
                )
            })
            .collect::<Vec<_>>();

        assert!(envs.contains(&("GIT_DIR".to_string(), None)));
        assert!(envs.contains(&("GIT_WORK_TREE".to_string(), None)));
        assert!(envs.contains(&("GIT_CONFIG_COUNT".to_string(), None)));
        assert!(envs.contains(&("GIT_INDEX_FILE".to_string(), None)));
        assert!(envs.contains(&("PATH".to_string(), Some("/tmp/bin".to_string()))));
    }
}
