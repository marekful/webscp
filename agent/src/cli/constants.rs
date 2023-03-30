pub const COMMAND_GET_LOCAL_VERSION: &'static str = "get-local-version";
pub const COMMAND_GET_LOCAL_RESOURCE: &'static str = "get-local-resource";
pub const COMMAND_GET_REMOTE_VERSION: &'static str = "get-remote-version";
pub const COMMAND_GET_REMOTE_RESOURCE: &'static str = "get-remote-resource";
pub const COMMAND_REMOTE_BEFORE_COPY: &'static str = "remote-before-copy";
pub const COMMAND_REMOTE_DO_COPY: &'static str = "remote-do-copy";
pub const COMMAND_LOCAL_BEFORE_COPY: &'static str = "local-before-copy";
pub const COMMAND_EXCHANGE_KEYS: &'static str = "exchange-keys";
pub const COMMAND_PING: &'static str = "ping";

pub struct Defaults {
    pub cli_command: &'static str,
    pub cli_executable_path: &'static str,
    pub default_fb_api_address: &'static str,
    pub authorized_keys_file: &'static str,
    pub known_hosts_file: &'static str,
    pub private_key_file: &'static str,
    pub public_key_file: &'static str,
    pub temp_data_dir: &'static str,
    pub with_contenv: &'static str,
    pub env_name_fb_api_address: &'static str,
    //fb_root_dir: &'static str,
}

pub const DEFAULTS: Defaults = Defaults {
    cli_command: "with-contenv /app/target/debug/cli",
    cli_executable_path: "/app/target/debug/cli",
    default_fb_api_address: "http://filebrowser:80",
    authorized_keys_file: "/home/agent/.ssh/authorized_keys",
    known_hosts_file: "/home/agent/.ssh/known_hosts",
    private_key_file: "/home/agent/.ssh/id_rsa",
    public_key_file: "/home/agent/.ssh/id_rsa.pub",
    temp_data_dir: "/home/agent/.tmp-data/",
    with_contenv: "with-contenv", //fb_root_dir: "/srv",
    env_name_fb_api_address: "FILEBROWSER_ADDRESS",
};
