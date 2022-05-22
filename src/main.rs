
use dvf::validation::{OperatorCommittee};
use dvf::validation::operator::TOperator;
use dvf::validation::operator::{HotStuffOperator, LocalOperator};
use dvf::crypto::{ThresholdSignature};
use std::sync::Arc;
use types::Hash256;
use eth2_hashing::{Context, Sha256Context};
use hotstuff_config::Export as _;
use dvf::node::node::Node;
use hotstuff_config::{Committee, Secret};
use dvf::node::dvfcore::DvfInfo;
use consensus::Committee as ConsensusCommittee;
use mempool::Committee as MempoolCommittee;
use std::fs;
use log::{error, info};
use tokio::task::JoinHandle;
use futures::future::join_all;
use dvf::node::dvfcore::{DvfCore, SignatureInfo, DvfSignatureReceiverHandler};
use network::{SimpleSender, DvfMessage};
use bytes::Bytes;
use std::net::SocketAddr;
use mempool::{MempoolMessage, Batch, Transaction};
use parking_lot::{RwLock};
use tokio::sync::mpsc::{channel, Sender};
use types::Keypair;
use std::{thread, time};
use env_logger::Env;
use futures::executor::block_on;
fn deploy_testbed(nodes: usize, kps: &Vec<Keypair>, tx_signature: Sender<SignatureInfo>, ids: &Vec<u32>) -> Result<Vec<JoinHandle<()>>, Box<dyn std::error::Error>> {

  let mut logger = env_logger::Builder::from_env(Env::default().default_filter_or("error"));
  logger.format_timestamp_millis();
  logger.init();
  let keys: Vec<_> = (0..nodes).map(|_| Secret::new()).collect();

  // Print the committee file.
  let epoch = 1;
  let mempool_committee = MempoolCommittee::new(
      keys.iter()
          .enumerate()
          .map(|(i, key)| {
              let name = key.name;
              let stake = 1;
              let front = format!("127.0.0.1:{}", 25_000 + i).parse().unwrap();
              let mempool = format!("127.0.0.1:{}", 25_100 + i).parse().unwrap();
              let dvf = format!("127.0.0.1:{}", 25_300 + i).parse().unwrap();
              let signature = format!("127.0.0.1:{}", 25_400 + i).parse().unwrap();
              (name, stake, front, mempool, dvf, signature)
          })
          .collect(),
      epoch,
  );
  let consensus_committee = ConsensusCommittee::new(
      keys.iter()
          .enumerate()
          .map(|(i, key)| {
              let name = key.name;
              let stake = 1;
              let addresses = format!("127.0.0.1:{}", 25_200 + i).parse().unwrap();
              (name, stake, addresses)
          })
          .collect(),
      epoch,
  );
  let committee_file = "committee.json";
  let _ = fs::remove_file(committee_file);
  let committee = Committee {
      mempool: mempool_committee,
      consensus: consensus_committee,
  };
  
  committee.write(committee_file)?;
  
  // Write the key files and spawn all nodes.
  keys.iter()
      .enumerate()
      .map(|(i, keypair)| {
          let key_file = format!("node_{}.json", i);
          let _ = fs::remove_file(&key_file);
          keypair.write(&key_file)?;
          let secret = keypair.clone();
          let store_path = format!("db_{}", i);
          let _ = fs::remove_dir_all(&store_path);
          let name = keypair.name.clone();
          let mem_address = committee.mempool
          .mempool_address(&name)
          .expect("Our public key is not in the committee");

          let tx_address = committee.mempool.transactions_address(&name)
          .expect("Our public key is not in the committee");

          let consensus_address = committee.consensus.address(&name)
          .expect("Our public key is not in the committee");

          let dvf_address = committee.mempool.dvf_address(&name)
          .expect("Our public key is not in the committee");

          let signature_address = committee.mempool.signature_address(&name)
            .expect("Our public key is not in the committee");
          let kp = kps[i].clone();
          let sender_signature = tx_signature.clone();
          let id = ids[i].clone();
          Ok(tokio::spawn(async move {
              match Node::new(&tx_address.to_string(), &mem_address.to_string(), &consensus_address.to_string(), &dvf_address.to_string(), &signature_address.to_string(), secret, &store_path, None).await {
                  Ok(mut node) => {
                      // Sink the commit channel.
                      // while node.commit.recv().await.is_some() {}

                      // node.process_dvfinfo().await;

                      info!("start dvf node {} success", name);

                      let committee_file = "committee.json";
                      let mut network = SimpleSender::new();
                      let committee = Committee::read(&committee_file).unwrap();
                      let validator_id: u64= 1;
                      // println!("received validator {}", validator_id);
                      
                      let ten_millis = time::Duration::from_millis(1);

                      match DvfCore::new(
                        committee,
                        node.name.clone(),
                        node.secret_key.clone(),
                        validator_id,
                        node.base_store_path.clone(),
                        Arc::clone(&node.tx_handler_map),
                        Arc::clone(&node.mempool_handler_map),
                        Arc::clone(&node.consensus_handler_map),
                        Arc::clone(&node.signature_handler_map),
                        kp,
                        id
                      ).await {
                        Ok(mut dvfcore) => {
                          dvfcore.save_committed_block().await;
                        }
                        Err(e) => {
                          error!("{}", e);
                        }
                      }
                        
                  }
                  Err(e) => error!("{}", e),
              }
          }))
      })
      .collect::<Result<_, Box<dyn std::error::Error>>>()
}

// async fn process_consensus_block(dvfcore: &mut DvfCore, keypair: Arc<Keypair>, id: u32) {
//   let operator = LocalOperator::from_keypair(keypair);
//   let mut network = SimpleSender::new();
  
//   while let Some(block) = dvfcore.commit.recv().await {
//     // This is where we can further process committed block.
//     if !block.payload.is_empty() {
//       for payload in block.payload {
//         match dvfcore.store.read(payload.to_vec()).await {
//           Ok(value) => {
//             match value {
//               Some(data) => {
//                 let message: MempoolMessage  = bincode::deserialize(&data[..]).expect("Failed to deserialize our own block");
//                 match message {
//                   MempoolMessage::Batch(batches) => {
//                     for batch in batches {
//                       let msg = Hash256::from_slice(&batch[..]);
//                       println!(" broadcast msg {:02x?}", msg);
//                       let sig = operator.sign(msg.clone());
//                       let pk = operator.public_key().unwrap();
//                       let sig_info = SignatureInfo { from: pk.clone(), signature: sig, msg: msg, id: id};
//                       let siginfo_data = serde_json::to_vec(&sig_info).unwrap();
//                       let dvg_message = DvfMessage { validator_id : dvfcore.validator_id, message: siginfo_data};
//                       let serialized_msg = bincode::serialize(&dvg_message).unwrap(); 
//                       for address in &boradcast_address {
//                         let add = address.clone();
//                         network.send(add, Bytes::from(serialized_msg.clone())).await;
//                       }
//                       // consensus batch origin data
//                       //  
//                       // get msg Hash256

//                       // sign msg and construct signature info 
//                       // broadcast to network
//                     }
//                   }
//                   MempoolMessage::BatchRequest(_, _) => { }
//                 }

                     
//               }
//               None => { }
//             }
//           }
//           Err(e) => println!("{}", e)
//         }
//       }
//     }
//   }
// }

async fn start_dvf_committee(node: &mut Node, tx_signature: Sender<SignatureInfo>, keypair: Arc<Keypair>) {

    
} 

// #[tokio::main(worker_threads = 2000)]
#[tokio::main]
async fn main() {
    let t: usize = 5;
    let n: usize = 10;
    let mut m_threshold = ThresholdSignature::new(t);
    let (kp, kps, ids) = m_threshold.key_gen(n);
    let (tx_signature, mut rx_signature) = channel(n + 1);
    let self_kp = kps[0].clone();
    if n > 1 {
      match deploy_testbed(n, &kps, tx_signature, &ids) {
        Ok(handles) => {
          // read committee from file 
          let committee_file = "committee.json";
          let mut network = SimpleSender::new();
          let committee = Committee::read(&committee_file).unwrap();
          let mut addresses : Vec<SocketAddr>= Vec::new();
          for (key, authority) in &committee.mempool.authorities {
            addresses.push(authority.dvf_address.clone());
          }
          let validator_id :u64 = 1;
          let dvfinfo = DvfInfo { validator_id, committee };

          let empty_vec : Vec<u8>= vec![48;88];
          let mut prefix_msg : Vec<u8> = Vec::new();
          prefix_msg.extend(empty_vec);
          let dvfinfo_bytes = serde_json::to_vec(&dvfinfo).unwrap();
          prefix_msg.extend(dvfinfo_bytes);


          // block_on(network.broadcast(addresses, Bytes::from(prefix_msg)));
          // wait for network
          let ten_millis = time::Duration::from_millis(20);
          thread::sleep(ten_millis);
          info!("committee sign");
          let mut committee = OperatorCommittee::new(0, t);
      //     // transaction address
          let address = "127.0.0.1:25001".parse::<SocketAddr>().unwrap();
          let operator = Arc::new(
            RwLock::new(HotStuffOperator::new(Arc::new(self_kp), address, rx_signature)));  
          committee.add_operator(ids[0], operator);
          let message = "hello world";
          let mut context = Context::new();
          context.update(message.as_bytes());
          let message = Hash256::from_slice(&context.finalize());
            println!("propose {:02x?}", message);
          let sig1 = committee.sign(message).unwrap();
          let sig2 = kp.sk.sign(message);

          let status1 = sig1.verify(&kp.pk, message);
          let status2 = sig2.verify(&kp.pk, message);

          println!("status1 {}", status1);
          println!("status2 {}", status2);

          let _ = join_all(handles).await;
        }
        Err(e) => error!("Failed to deploy testbed: {}", e),
      }
    }
}