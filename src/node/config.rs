use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::fs::create_dir_all;
use serde_derive::{Deserialize, Serialize};
use crate::DEFAULT_DVF_ROOT_DIR;
use directory::{DEFAULT_VALIDATOR_DIR, DEFAULT_SECRET_DIR};
/// The file name for the serialized `OperatorCommitteeDefinition` struct.
pub const NODE_KEY_FILENAME: &str = "node_key.json";
pub const DB_FILENAME: &str = "dvf_node_db";

pub const DEFAULT_BASE_PORT: u16 = 25_000;
pub const TRANSACTION_PORT_OFFSET: u16 = 0;
pub const MEMPOOL_PORT_OFFSET: u16 = 1;
pub const CONSENSUS_PORT_OFFSET: u16 = 2;
pub const SIGNATURE_PORT_OFFSET: u16 = 3; 
pub const DISCOVERY_PORT_OFFSET: u16 = 4;
pub const BASE_ADDRESS: [u8; 4] = [127, 0, 0, 1];
pub const BASE64_ENR : &str = "-IS4QFerXwqYOe28mrjkMppF0MOHlWPXz4a3HPZK3wwX4oSVFhmgzEJQbaUMrsAV1e-8cyPJ3ooVQz_Z9fhgloUFE6UBgmlkgnY0gmlwhCNYD_SJc2VjcDI1NmsxoQPKY0yuDUmstAHYpMa2_oxVtw0RW_QAdpzBQA8yWM0xOIN1ZHCCIyg";
pub const CONTRACT_ABI_PATH: &str = "abi/abi.json";
pub const BACKEND_IP: [u8; 4] = [127, 0, 0, 1];
pub const BACKEND_PORT: u16 = 80;
#[derive(Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    // pub id: u64,
    pub base_address: SocketAddr,
    pub transaction_address: SocketAddr,
    pub mempool_address: SocketAddr,
    pub consensus_address: SocketAddr,
    pub signature_address: SocketAddr,
    pub base_store_path: PathBuf,
    pub node_key_path: PathBuf,
    pub validator_dir: PathBuf,
    pub secrets_dir: PathBuf,
    pub backend_address: SocketAddr,
    pub boot_enr: String
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self::new(
            1,
            IpAddr::V4(Ipv4Addr::new(BASE_ADDRESS[0], BASE_ADDRESS[1], BASE_ADDRESS[2], BASE_ADDRESS[3])),
            DEFAULT_BASE_PORT, 
        ) 
    }
}

impl NodeConfig {
    pub fn new(id: u64, ip: IpAddr, base_port: u16) -> Self {
        if id == 0 {
            panic!("Invalid id");
        }

        let base_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(DEFAULT_DVF_ROOT_DIR);
        let base_store_path = base_dir.join(DB_FILENAME);
        let node_key_path = base_dir.join(NODE_KEY_FILENAME);
        let validator_dir = base_dir.join(DEFAULT_VALIDATOR_DIR);
        let secrets_dir = base_dir.join(DEFAULT_SECRET_DIR);
        let backend_ip = IpAddr::V4(Ipv4Addr::new(BACKEND_IP[0], BACKEND_IP[1], BACKEND_IP[2], BACKEND_IP[3]));
        Self {
            // id,
            base_address: SocketAddr::new(ip.clone(), base_port),
            transaction_address: SocketAddr::new(ip.clone(), base_port + TRANSACTION_PORT_OFFSET),
            mempool_address: SocketAddr::new(ip.clone(), base_port + MEMPOOL_PORT_OFFSET),
            consensus_address: SocketAddr::new(ip.clone(), base_port + CONSENSUS_PORT_OFFSET),
            signature_address: SocketAddr::new(ip.clone(), base_port + SIGNATURE_PORT_OFFSET),
            base_store_path,
            node_key_path,
            validator_dir,
            secrets_dir,
            backend_address: SocketAddr::new(backend_ip, BACKEND_PORT),
            boot_enr: BASE64_ENR.to_string()
        }
    }

    pub fn set_id(mut self, id: u64) -> Self {
        if id == 0 {
            panic!("Invalid id");
        }
        // self.id = id;
        self
    }

    pub fn set_base_dir(mut self, path: PathBuf) -> Self {
        if !path.exists() {
            let _ = create_dir_all(&path);
        }
        self.base_store_path = path.join(DB_FILENAME);
        self.node_key_path = path.join(NODE_KEY_FILENAME);
        self
    }

    pub fn set_base_port(mut self, new_port: u16) -> Self {
        self.base_address.set_port(new_port);
        self.transaction_address.set_port(new_port + TRANSACTION_PORT_OFFSET);
        self.mempool_address.set_port(new_port + MEMPOOL_PORT_OFFSET);
        self.consensus_address.set_port(new_port + CONSENSUS_PORT_OFFSET);
        self.signature_address.set_port(new_port + SIGNATURE_PORT_OFFSET);
        self
    }
}
