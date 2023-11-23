use candid::{
  candid_method, CandidType, Decode, Deserialize, Encode, Principal,
};
use ic_cdk::{query, update};
use ic_stable_structures::{
  memory_manager::{MemoryId, MemoryManager, VirtualMemory},
  storable::Bound,
  DefaultMemoryImpl, StableBTreeMap, Storable,
};
use std::{borrow::Cow, cell::RefCell};

// memory for stable structures
type Memory = VirtualMemory<DefaultMemoryImpl>;

// upper limit of todo item size
const MAX_ITEM_SIZE: u32 = 128;

// Todo Item
#[derive(Deserialize, CandidType, Clone)]
struct Item {
  owner: Principal,
  content: String,
}

impl Item {
  pub fn new(owner: Principal, content: String) -> Self {
    Self { owner, content }
  }
}

impl Storable for Item {
  const BOUND: Bound = Bound::Bounded {
    max_size: MAX_ITEM_SIZE,
    is_fixed_size: false,
  };

  fn to_bytes(&self) -> Cow<[u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }
}

// page struct
#[derive(Clone, CandidType, Deserialize)]
pub struct Page {
  items: Vec<Item>,
  has_next_page: bool,
}

// as icp canister is single threaded we can safely use thread_local
thread_local! {
static MEM_MGR : RefCell<MemoryManager<DefaultMemoryImpl>> =
  RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

static DATA_STORE: RefCell<StableBTreeMap<u64, Item, Memory>> =
  RefCell::new(StableBTreeMap::init(MEM_MGR.with(|mem_mgr| mem_mgr.borrow().get(MemoryId::new(0_u8)))));
}

#[query(name = "fetch_all")]
#[candid_method(query)]
fn fetch_all(page_number: usize, page_size: usize) -> Result<Page, String> {
  let caller = ic_cdk::caller();

  let items = DATA_STORE.with(|p| {
    p.borrow()
      .iter()
      .map(|(_, item)| item)
      .filter(|item| item.owner.eq(&caller))
      .skip((page_number - 1) * page_size)
      .take(page_size)
      .collect()
  });

  // for simplyfying the logic always considering has next pages
  Ok(Page {
    items,
    has_next_page: true,
  })
}

#[query(name = "get")]
#[candid_method(query)]
fn get(key: u64) -> Option<Item> {
  DATA_STORE.with(|p| p.borrow().get(&key))
}

#[update(name = "create")]
#[candid_method(update)]

fn create(key: u64, value: String) -> Result<(), String> {
  let caller = ic_cdk::caller();
  DATA_STORE.with(|p| {
    let mut store_ref = p.borrow_mut();
    if store_ref.contains_key(&key) {
      return Err("Duplicate Id".to_owned());
    }
    store_ref.insert(key, Item::new(caller, value));
    Ok(())
  })
}

#[candid_method(update)]
#[update(name = "update")]
fn update(key: u64, value: String) -> Result<(), String> {
  DATA_STORE.with(|map| {
    if map
      .borrow()
      .get(&key)
      .filter(|item| item.owner == ic_cdk::caller())
      .is_some()
    {
      map
        .borrow_mut()
        .insert(key, Item::new(ic_cdk::caller(), value));
    } else {
      return Err("You are not the owner of todo item".to_owned());
    };
    Ok(())
  })
}

#[candid_method(update)]
#[update(name = "delete")]
fn delete(key: u64) -> Result<(), String> {
  DATA_STORE.with(|map| {
    if map
      .borrow()
      .get(&key)
      .filter(|item| item.owner == ic_cdk::caller())
      .is_some()
    {
      map.borrow_mut().remove(&key);
    } else {
      return Err("You are not the owner of todo item".to_owned());
    };
    Ok(())
  })
}

candid::export_service!();

#[query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
  __export_service()
}

#[cfg(test)]
mod tests {
  use crate::export_candid;

  #[test]
  fn save_candid() {
    use std::env;
    use std::fs::write;
    use std::path::PathBuf;

    let dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let dir = dir.parent().unwrap().parent().unwrap();
    println!("{:?}", dir);
    write(dir.join("generated_icp_todo_backend.did"), export_candid())
      .expect("Write failed.");
  }
}
