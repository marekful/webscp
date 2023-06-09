pub const COMMAND_GET_LOCAL_VERSION: &str = "get-local-version";
pub const COMMAND_GET_LOCAL_RESOURCE: &str = "get-local-resource";
pub const COMMAND_GET_REMOTE_VERSION: &str = "get-remote-version";
pub const COMMAND_GET_REMOTE_RESOURCE: &str = "get-remote-resource";
pub const COMMAND_REMOTE_BEFORE_COPY: &str = "remote-before-copy";
pub const COMMAND_REMOTE_DO_COPY: &str = "remote-do-copy";
pub const COMMAND_LOCAL_BEFORE_COPY: &str = "local-before-copy";
pub const COMMAND_EXCHANGE_KEYS: &str = "exchange-keys";
pub const COMMAND_GET_REMOTE_USER: &str = "get-remote-user";
pub const COMMAND_GET_TOKEN_USER: &str = "get-token-user";
pub const COMMAND_GET_LOCAL_USER: &str = "get-local-user";
pub const COMMAND_PING: &str = "ping";

pub struct Defaults {
    pub cli_executable_path: &'static str,
    pub default_fb_api_address: &'static str,
    pub authorized_keys_file: &'static str,
    pub known_hosts_file: &'static str,
    pub private_key_file: &'static str,
    pub public_key_file: &'static str,
    pub temp_data_dir: &'static str,
    pub with_contenv: &'static str,
    pub env_name_fb_api_address: &'static str,
    pub uploader_script_path: &'static str,
    pub cancel_transfer_script_path: &'static str,
    pub generate_key_pair_script_path: &'static str,
    pub revoke_key_pair_script_path: &'static str,
    pub extract_archive_script_path: &'static str,
    pub temporary_key_file_name: &'static str,
    pub ssh_dir_path: &'static str,
}

pub const DEFAULTS: Defaults = Defaults {
    cli_executable_path: "/app/cli",
    default_fb_api_address: "http://files",
    authorized_keys_file: "/app/data/client/.ssh/authorized_keys",
    known_hosts_file: "/app/data/client/.ssh/known_hosts",
    private_key_file: "/app/data/client/.ssh/id_rsa",
    public_key_file: "/app/data/client/.ssh/id_rsa.pub",
    temp_data_dir: "/app/data/temp/",
    with_contenv: "with-contenv",
    env_name_fb_api_address: "FILES_ADDRESS",
    uploader_script_path: "/etc/scripts/uploader.sh",
    cancel_transfer_script_path: "/etc/scripts/cancel-transfer.sh",
    generate_key_pair_script_path: "/etc/scripts/generate-key-pair.sh",
    revoke_key_pair_script_path: "/etc/scripts/revoke-key-pair.sh",
    extract_archive_script_path: "/etc/scripts/extract-archive.sh",
    temporary_key_file_name: "/app/data/client/.ssh/id_ecdsa-pem",
    ssh_dir_path: "/app/data/client/.ssh",
};
