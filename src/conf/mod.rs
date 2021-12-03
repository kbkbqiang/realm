use std::fs;
use std::io::{Result, Error, ErrorKind};

use serde::{Serialize, Deserialize};

mod log;
pub use self::log::{LogLevel, LogConf};

mod dns;
pub use dns::{DnsMode, DnsProtocol, DnsConf, CompatibleDnsConf};

mod endpoint;
pub use endpoint::EndpointConf;

#[derive(Debug, Default)]
pub struct GlobalOpts {
    pub log_level: Option<LogLevel>,
    pub log_output: Option<String>,
    pub dns_mode: Option<DnsMode>,
    pub dns_protocol: Option<DnsProtocol>,
    pub dns_servers: Option<Vec<String>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FullConf {
    #[serde(default)]
    pub log: LogConf,

    #[serde(default)]
    pub dns: CompatibleDnsConf,

    pub endpoints: Vec<EndpointConf>,
}

impl FullConf {
    #[allow(unused)]
    pub fn new(
        log: LogConf,
        dns: DnsConf,
        endpoints: Vec<EndpointConf>,
    ) -> Self {
        FullConf {
            log,
            dns: CompatibleDnsConf::DnsConf(dns),
            endpoints,
        }
    }

    pub fn from_conf_file(file: &str) -> Self {
        let conf = fs::read_to_string(file)
            .unwrap_or_else(|e| panic!("unable to open {}: {}", file, &e));
        match Self::from_conf_str(&conf) {
            Ok(x) => x,
            Err(e) => panic!("failed to parse {}: {}", file, &e),
        }
    }

    pub fn from_conf_str(conf: &str) -> Result<Self> {
        let toml_err = match toml::from_str(conf) {
            Ok(x) => return Ok(x),
            Err(e) => e,
        };
        let json_err = match serde_json::from_str(conf) {
            Ok(x) => return Ok(x),
            Err(e) => e,
        };

        Err(Error::new(
            ErrorKind::Other,
            format!(
                "parse as toml: {0}; parse as json: {1}",
                &toml_err, &json_err
            ),
        ))
    }

    pub fn add_endpoint(&mut self, endpoint: EndpointConf) -> &mut Self {
        self.endpoints.push(endpoint);
        self
    }

    // move CompatibleDnsConf::DnsMode into CompatibleDnsConf::DnsConf
    pub fn move_dns_conf(&mut self) -> &mut Self {
        if let CompatibleDnsConf::None = self.dns {
            let conf = DnsConf::default();
            self.dns = CompatibleDnsConf::DnsConf(conf);
        }
        if let CompatibleDnsConf::DnsMode(mode) = self.dns {
            let conf = DnsConf {
                mode,
                ..Default::default()
            };
            self.dns = CompatibleDnsConf::DnsConf(conf);
        }
        self
    }

    pub fn apply_global_opts(&mut self, opts: GlobalOpts) -> &mut Self {
        let GlobalOpts {
            log_level,
            log_output,
            dns_mode,
            dns_protocol,
            dns_servers,
        } = opts;

        if dns_mode.is_some() || dns_protocol.is_some() || dns_servers.is_some()
        {
            self.move_dns_conf();
        }

        macro_rules! reset {
            ($res: expr, $field: ident) => {
                if let Some($field) = $field {
                    $res = $field
                }
            };
        }
        reset!(self.log.level, log_level);
        reset!(self.log.output, log_output);
        reset!(self.dns.as_mut().mode, dns_mode);
        reset!(self.dns.as_mut().protocol, dns_protocol);
        reset!(self.dns.as_mut().nameservers, dns_servers);

        self
    }
}
