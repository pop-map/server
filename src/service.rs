use std::{collections::HashMap, sync::Mutex, time::SystemTime};

pub use popmap::{
    telegram_auth::Token, Area, GetPep, GetPop, Location, PostPep, PostPop, Time, UserInfo, Uuid,
    LEN_LIMIT_CONTENT,
};

type Pep = GetPep;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct Pop {
    title: String,
    description: String,
    user: UserInfo,
    location: Location,
    created: Time,
    expire: Time,
    peps: Vec<Pep>,
}

#[derive(Debug)]
pub struct Service {
    pops: Mutex<HashMap<Uuid, Pop>>,
    token: Token,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rejection {
    NotFound,
    BadAuth,
    OffLenLimit,
}

impl Default for Service {
    fn default() -> Self {
        Self::new()
    }
}

impl Service {
    pub fn new() -> Self {
        Self {
            pops: Mutex::new(HashMap::new()),
            token: popmap::telegram_auth::token_from_file("telegram-token"),
        }
    }
    pub fn post_a_new_pop(&self, pop: PostPop) -> Result<Uuid, Rejection> {
        let PostPop {
            title,
            description,
            user,
            location,
            expire,
        } = pop;
        let user: UserInfo = (self.token, user)
            .try_into()
            .map_err(|_| Rejection::BadAuth)?;
        let created = SystemTime::UNIX_EPOCH.elapsed().unwrap().as_secs();
        let pop = Pop {
            title,
            description,
            user,
            location,
            created,
            expire,
            peps: Vec::new(),
        };
        let id = Uuid::new_v4();
        let mut pops = self.pops.lock().unwrap();
        // TODO: proper rejection
        if pops.len() > 1000 {
            return Err(Rejection::OffLenLimit);
        }
        pops.insert(id, pop);
        Ok(id)
    }
    pub fn get_pops_in_an_area(&self, area: Area) -> Result<Vec<Uuid>, Rejection> {
        let pops = self.pops.lock().unwrap();
        Ok(pops
            .iter()
            .filter(|(_, pop)| area.contains(pop.location))
            .map(|(id, _)| *id)
            .collect())
    }
    pub fn get_specific_pop(&self, id: Uuid) -> Result<GetPop, Rejection> {
        let pops = self.pops.lock().unwrap();
        pops.get(&id).ok_or(Rejection::NotFound).map(|pop| GetPop {
            title: pop.title.clone(),
            description: pop.description.clone(),
            user: pop.user.clone(),
            location: pop.location,
            expire: pop.expire,
            created: pop.created,
            peps: pop.peps.len(),
        })
    }
    pub fn post_a_pep_in_a_pop(&self, id: Uuid, pep: PostPep) -> Result<usize, Rejection> {
        let PostPep { content, user } = pep;
        if content.len() > LEN_LIMIT_CONTENT {
            return Err(Rejection::OffLenLimit);
        }
        let user: UserInfo = (self.token, user)
            .try_into()
            .map_err(|_| Rejection::BadAuth)?;
        let created = SystemTime::UNIX_EPOCH.elapsed().unwrap().as_secs();
        let mut pops = self.pops.lock().unwrap();
        let peps = &mut pops.get_mut(&id).ok_or(Rejection::NotFound)?.peps;
        let index = peps.len();
        // TODO: proper rejection
        if index > 100 {
            return Err(Rejection::OffLenLimit);
        }
        peps.push(GetPep {
            content,
            user,
            created,
        });
        Ok(index)
    }
    pub fn get_specific_pep(&self, id: Uuid, index: usize) -> Result<GetPep, Rejection> {
        let pops = self.pops.lock().unwrap();
        pops.get(&id)
            .ok_or(Rejection::NotFound)?
            .peps
            .get(index)
            .ok_or(Rejection::NotFound)
            .cloned()
    }
    pub fn dev_action_clear_all(&self) {
        let mut pops = self.pops.lock().unwrap();
        pops.clear();
    }
}
