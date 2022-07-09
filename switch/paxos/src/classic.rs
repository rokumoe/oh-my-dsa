use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use crate::proto::paxos as paxospb;

use bytes::Bytes;
use bytes::BytesMut;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct Persist {
    dir: PathBuf,
}

impl Persist {
    pub fn new<P: AsRef<Path>>(p: P) -> io::Result<Self> {
        fs::create_dir_all(p.as_ref())?;
        Ok(Self {
            dir: p.as_ref().to_path_buf(),
        })
    }

    fn load_proposer_ballot(&self) -> io::Result<u64> {
        let file_path = self.dir.join("proposer_ballot");
        if !file_path.exists() {
            return Ok(0);
        }
        fs::read_to_string(file_path).map(|s| s.parse().unwrap_or(0))
    }

    fn store_proposer_ballot(&self, ballot_num: u64) -> io::Result<()> {
        fs::write(self.dir.join("proposer_ballot"), ballot_num.to_string())
    }

    fn load_acceptor_ballot(&self) -> io::Result<u64> {
        let file_path = self.dir.join("acceptor_ballot");
        if !file_path.exists() {
            return Ok(0);
        }
        fs::read_to_string(file_path).map(|s| s.parse().unwrap_or(0))
    }

    fn store_acceptor_ballot(&self, ballot_num: u64) -> io::Result<()> {
        fs::write(self.dir.join("acceptor_ballot"), ballot_num.to_string())
    }

    fn load_acceptor_state(&self) -> io::Result<Option<Promised>> {
        let file_path = self.dir.join("acceptor_accepted");
        if !file_path.exists() {
            return Ok(None);
        }

        let mut payload = Bytes::from(fs::read(file_path)?);
        let ln = payload.iter().position(|&c| c == b'\n').unwrap_or(0);
        let ballot_num = std::str::from_utf8(&payload.split_to(ln))
            .unwrap_or("0")
            .parse()
            .unwrap_or(0);
        let value = payload.slice(1..);
        Ok(Some(Promised { ballot_num, value }))
    }

    fn store_acceptor_state(&self, state: &Option<Promised>) -> io::Result<()> {
        if let Some(state) = state {
            let mut buf = BytesMut::new();
            buf.extend_from_slice(state.ballot_num.to_string().as_bytes());
            buf.extend_from_slice(&[b'\n']);
            buf.extend_from_slice(&state.value);
            fs::write(self.dir.join("acceptor_accepted"), &buf)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone)]
pub struct Promised {
    pub ballot_num: u64,
    pub value: Bytes,
}

struct AcceptorInner {
    max_seen_ballot: u64,
    accepted: Option<Promised>,
}

pub struct Acceptor {
    inner: Mutex<AcceptorInner>,
    persist: Persist,
}

impl Acceptor {
    pub fn new(persist: Persist) -> io::Result<Self> {
        let max_seen_ballot = persist.load_acceptor_ballot()?;
        let accepted = persist.load_acceptor_state()?;
        Ok(Self {
            inner: Mutex::new(AcceptorInner {
                max_seen_ballot,
                accepted,
            }),
            persist,
        })
    }
}

#[tonic::async_trait]
impl paxospb::acceptor_server::Acceptor for Acceptor {
    async fn prepare(
        &self,
        request: Request<paxospb::Prepare>,
    ) -> Result<Response<paxospb::Promise>, Status> {
        log::info!("acceptor| prepare: {:?}", request);

        let prepare = request.get_ref();
        let mut inner = self.inner.lock().await;
        if inner.max_seen_ballot > prepare.ballot_num {
            return Ok(Response::new(paxospb::Promise {
                ok: false,
                ballot_num: inner.max_seen_ballot,
                value: Vec::new(),
            }));
        }

        self.persist.store_acceptor_ballot(prepare.ballot_num)?;

        inner.max_seen_ballot = prepare.ballot_num;
        if let Some(ref accepted) = inner.accepted {
            Ok(Response::new(paxospb::Promise {
                ok: true,
                ballot_num: accepted.ballot_num,
                value: accepted.value.to_vec(),
            }))
        } else {
            Ok(Response::new(paxospb::Promise {
                ok: true,
                ballot_num: 0,
                value: Vec::new(),
            }))
        }
    }

    async fn accept(
        &self,
        request: Request<paxospb::Propose>,
    ) -> Result<Response<paxospb::Accept>, Status> {
        log::info!("acceptor| accept: {:?}", request);

        let propose = request.into_inner();
        let mut inner = self.inner.lock().await;
        if inner.max_seen_ballot > propose.ballot_num {
            return Ok(Response::new(paxospb::Accept { ok: false }));
        }

        self.persist.store_acceptor_ballot(propose.ballot_num)?;
        let state = Some(Promised {
            ballot_num: propose.ballot_num,
            value: Bytes::from(propose.value),
        });
        self.persist.store_acceptor_state(&state)?;

        inner.max_seen_ballot = propose.ballot_num;
        inner.accepted = state;

        Ok(Response::new(paxospb::Accept { ok: true }))
    }
}

type AcceptorClient = paxospb::acceptor_client::AcceptorClient<tonic::transport::Channel>;

pub struct Configuraion {
    pub acceptors: Vec<Arc<Mutex<AcceptorClient>>>,
}

pub struct Proposer {
    id: u64,
    ballot_num: u64,
    persist: Persist,
}

impl Proposer {
    pub fn new(id: u64, persist: Persist) -> io::Result<Self> {
        let ballot_num = persist.load_proposer_ballot()?;
        Ok(Self {
            id,
            ballot_num,
            persist,
        })
    }

    pub fn next_ballot_num(&mut self) -> io::Result<u64> {
        let ballot_num = self.ballot_num + 1;
        self.persist.store_proposer_ballot(ballot_num)?;
        self.ballot_num = ballot_num;
        Ok(ballot_num * 1000 + self.id)
    }

    pub async fn prepare(&self, cfg: &Configuraion, ballot_num: u64) -> Promised {
        let n = cfg.acceptors.len();
        let (tx, mut rx) = mpsc::channel::<paxospb::Promise>(n);
        for (idx, acceptor) in cfg.acceptors.iter().enumerate() {
            let acceptor = acceptor.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                let mut fails = 0;
                while fails < 3 {
                    let res = acceptor
                        .lock()
                        .await
                        .prepare(paxospb::Prepare { ballot_num })
                        .await;
                    match res {
                        Ok(resp) => {
                            let _ = tx.send(resp.into_inner()).await;
                            return;
                        }
                        Err(err) => {
                            log::error!("proposor| acceptor {} prepare error: {}", idx, err);
                            fails += 1;
                        }
                    }
                }
                let _ = tx
                    .send(paxospb::Promise {
                        ok: false,
                        ballot_num: 0,
                        value: Vec::new(),
                    })
                    .await;
            });
        }

        let majority = n / 2 + 1;

        let mut chosen = Promised {
            ballot_num: ballot_num,
            value: Bytes::new(),
        };
        let mut promised = 0;
        while let Some(promise) = rx.recv().await {
            if !promise.ok {
                continue;
            }

            if !promise.value.is_empty() {
                if chosen.value.is_empty() || promise.ballot_num > chosen.ballot_num {
                    chosen.ballot_num = promise.ballot_num;
                    chosen.value = Bytes::from(promise.value);
                }
            }

            promised += 1;
            if promised >= majority {
                break;
            }
        }

        chosen
    }

    pub async fn accept(&self, cfg: &Configuraion, chosen: Promised) -> bool {
        let n = cfg.acceptors.len();
        let (tx, mut rx) = mpsc::channel::<paxospb::Accept>(n);
        for (idx, acceptor) in cfg.acceptors.iter().enumerate() {
            let acceptor = acceptor.clone();
            let tx = tx.clone();
            let choosen = chosen.clone();
            tokio::spawn(async move {
                let mut fails = 0;
                while fails < 3 {
                    let res = acceptor
                        .lock()
                        .await
                        .accept(paxospb::Propose {
                            ballot_num: choosen.ballot_num,
                            value: choosen.value.to_vec(),
                        })
                        .await;
                    match res {
                        Ok(resp) => {
                            let _ = tx.send(resp.into_inner()).await;
                            return;
                        }
                        Err(err) => {
                            log::error!("proposer| acceptor {} accept error: {}", idx, err);
                            fails += 1;
                        }
                    }
                }
                let _ = tx.send(paxospb::Accept { ok: false }).await;
            });
        }

        let majority = n / 2 + 1;

        let mut accepted = 0;
        while let Some(accept) = rx.recv().await {
            if !accept.ok {
                continue;
            }

            accepted += 1;
            if accepted >= majority {
                break;
            }
        }

        accepted >= majority
    }
}

pub async fn paxos(proposer: &mut Proposer, cfg: &Configuraion, value: Bytes) -> io::Result<bool> {
    let ballot_num = proposer.next_ballot_num()?;
    let mut promised = proposer.prepare(cfg, ballot_num).await;
    if promised.value.is_empty() {
        promised.value = value;
    }
    Ok(proposer.accept(cfg, promised).await)
}
