#[macro_use]
extern crate serde;

use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};
use std::collections::HashMap;

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Farmer {
    id: u64,
    name: String,
    email: String,
    phone: String,
    product_ids: Vec<u64>,
    created_at: u64,
    updated_at: Option<u64>,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Product {
    id: u64,
    name: String,
    description: String,
    farmer_id: u64,
    created_at: u64,
    updated_at: Option<u64>,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Wholesaler {
    id: u64,
    name: String,
    email: String,
    phone: String,
    order_ids: Vec<u64>,
    created_at: u64,
    updated_at: Option<u64>,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct SupplyOrder {
    id: u64,
    title: String,
    farmer_id: u64,
    wholesaler_id: Option<u64>,
    product_types: Vec<String>,
    products: HashMap<String, u64>,
    is_complete: bool,
    created_at: u64,
    updated_at: Option<u64>,
}

impl Storable for Farmer {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl Storable for Product {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl Storable for Wholesaler {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl Storable for SupplyOrder {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Farmer {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl BoundedStorable for Product {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl BoundedStorable for Wholesaler {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl BoundedStorable for SupplyOrder {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static FARMER_STORAGE: RefCell<StableBTreeMap<u64, Farmer, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));

    static PRODUCT_STORAGE: RefCell<StableBTreeMap<u64, Product, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));

    static WHOLESALER_STORAGE: RefCell<StableBTreeMap<u64, Wholesaler, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
    ));

    static SUPPLY_ORDERS: RefCell<StableBTreeMap<u64, SupplyOrder, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4)))
    ));
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct FarmerPayload {
    name: String,
    email: String,
    phone: String,
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct ProductPayload {
    name: String,
    description: String,
    farmer_id: u64,
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct WholesalerPayload {
    name: String,
    email: String,
    phone: String,
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct SupplyOrderPayload {
    title: String,
    farmer_id: u64,
    wholesaler_id: u64,
    products: HashMap<String, u64>,
    product_types: Vec<String>,
    is_complete: bool,
}

#[derive(candid::CandidType, Deserialize, Serialize, Default)]
struct AddSupplyOrderWholesalerPayload {
    order_id: u64,
    wholesaler_id: u64,
}

#[ic_cdk::query]
fn get_farmer(id: u64) -> Result<Farmer, Error> {
    match _get_farmer(&id) {
        Some(farmer) => Ok(farmer),
        None => Err(Error::NotFound {
            msg: format!("Farmer with id:{} not found", id),
        }),
    }
}

#[ic_cdk::update]
fn add_farmer(payload: FarmerPayload) -> Result<Farmer, Error> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    let farmer = Farmer {
        id,
        name: payload.name,
        email: payload.email,
        phone: payload.phone,
        product_ids: vec![],
        created_at: time(),
        updated_at: None,
    };

    _insert_farmer(&farmer);

    Ok(farmer)
}

fn _get_farmer(id: &u64) -> Option<Farmer> {
    FARMER_STORAGE.with(|farmers| farmers.borrow().get(&id))
}

fn _insert_farmer(farmer: &Farmer) {
    FARMER_STORAGE.with(|farmers| farmers.borrow_mut().insert(farmer.id, farmer.clone()));
}

#[ic_cdk::query]
fn get_product(id: u64) -> Result<Product, Error> {
    match _get_product(&id) {
        Some(product) => Ok(product),
        None => Err(Error::NotFound {
            msg: format!("Product with id:{} not found", id),
        }),
    }
}

#[ic_cdk::update]
fn add_product(payload: ProductPayload) -> Result<Product, Error> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    let product = Product {
        id,
        name: payload.name,
        description: payload.description,
        farmer_id: payload.farmer_id,
        created_at: time(),
        updated_at: None,
    };

    _insert_product(&product);

    Ok(product)
}

fn _get_product(id: &u64) -> Option<Product> {
    PRODUCT_STORAGE.with(|products| products.borrow().get(&id))
}

fn _insert_product(product: &Product) {
    PRODUCT_STORAGE.with(|products| products.borrow_mut().insert(product.id, product.clone()));
}

#[ic_cdk::query]
fn get_wholesaler(id: u64) -> Result<Wholesaler, Error> {
    match _get_wholesaler(&id) {
        Some(wholesaler) => Ok(wholesaler),
        None => Err(Error::NotFound {
            msg: format!("Wholesaler with id:{} not found", id),
        }),
    }
}

#[ic_cdk::query]
fn get_wholesalers() -> Result<Vec<Wholesaler>, Error> {
    let wholesalers_map: Vec<(u64, Wholesaler)> =
        WHOLESALER_STORAGE.with(|service| service.borrow().iter().collect());
    let wholesalers: Vec<Wholesaler> = wholesalers_map
        .into_iter()
        .map(|(_, wholesaler)| wholesaler)
        .collect();

    if !wholesalers.is_empty() {
        Ok(wholesalers)
    } else {
        Err(Error::NotFound {
            msg: "No wholesalers available.".to_string(),
        })
    }
}

#[ic_cdk::update]
fn add_wholesaler(payload: WholesalerPayload) -> Result<Wholesaler, Error> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    let wholesaler = Wholesaler {
        id,
        name: payload.name,
        email: payload.email,
        phone: payload.phone,
        order_ids: vec![],
        created_at: time(),
        updated_at: None,
    };

    _insert_wholesaler(&wholesaler);

    Ok(wholesaler)
}


fn _get_wholesaler(id: &u64) -> Option<Wholesaler> {
    WHOLESALER_STORAGE.with(|wholesalers| wholesalers.borrow().get(&id))
}

fn _insert_wholesaler(wholesaler: &Wholesaler) {
    WHOLESALER_STORAGE
        .with(|wholesalers| wholesalers.borrow_mut().insert(wholesaler.id, wholesaler.clone()));
}

#[ic_cdk::query]
fn get_supply_order(id: u64) -> Result<SupplyOrder, Error> {
    match _get_supply_order(&id) {
        Some(supply_order) => Ok(supply_order),
        None => Err(Error::NotFound {
            msg: format!("Supply order with id:{} not found", id),
        }),
    }
}

#[ic_cdk::query]
fn get_supply_orders() -> Result<Vec<SupplyOrder>, Error> {
    let supply_orders_map: Vec<(u64, SupplyOrder)> =
        SUPPLY_ORDERS.with(|service| service.borrow().iter().collect());
    let supply_orders: Vec<SupplyOrder> = supply_orders_map
        .into_iter()
        .map(|(_, supply_order)| supply_order)
        .collect();

    if !supply_orders.is_empty() {
        Ok(supply_orders)
    } else {
        Err(Error::NotFound {
            msg: "No supply orders available.".to_string(),
        })
    }
}

#[ic_cdk::query]
fn get_incomplete_supply_orders() -> Result<Vec<SupplyOrder>, Error> {
    let supply_orders_map: Vec<(u64, SupplyOrder)> =
        SUPPLY_ORDERS.with(|service| service.borrow().iter().collect());
    let supply_orders: Vec<SupplyOrder> = supply_orders_map
        .into_iter()
        .map(|(_, supply_order)| supply_order)
        .filter(|supply_order| !supply_order.is_complete)
        .collect();

    if !supply_orders.is_empty() {
        Ok(supply_orders)
    } else {
        Err(Error::NotFound {
            msg: "No incomplete supply orders available.".to_string(),
        })
    }
}

#[ic_cdk::query]
fn get_wholesaler_orders(wholesaler_id: u64) -> Result<Vec<SupplyOrder>, Error> {
    let supply_orders_map: Vec<(u64, SupplyOrder)> =
        SUPPLY_ORDERS.with(|service| service.borrow().iter().collect());
    let supply_orders: Vec<SupplyOrder> = supply_orders_map
        .into_iter()
        .map(|(_, supply_order)| supply_order)
        .filter(|supply_order| supply_order.wholesaler_id == Some(wholesaler_id))
        .collect();

    if !supply_orders.is_empty() {
        Ok(supply_orders)
    } else {
        Err(Error::NotFound {
            msg: format!("No supply orders available for wholesaler id:{}", wholesaler_id),
        })
    }
}

#[ic_cdk::update]
fn add_supply_order(payload: SupplyOrderPayload) -> Option<SupplyOrder> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    let supply_order = SupplyOrder {
        id,
        title: payload.title,
        farmer_id: payload.farmer_id,
        wholesaler_id: Some(payload.wholesaler_id),
        products: payload.products,
        product_types: payload.product_types,
        is_complete: false,
        created_at: time(),
        updated_at: None,
    };

    _insert_supply_order(&supply_order);

    Some(supply_order)
}

#[ic_cdk::update]
fn add_supply_order_wholesaler(payload: AddSupplyOrderWholesalerPayload) -> Result<SupplyOrder, Error> {
    match SUPPLY_ORDERS.with(|service| service.borrow().get(&payload.order_id)) {
        Some(mut supply_order) => {
            supply_order.wholesaler_id = Some(payload.wholesaler_id);
            supply_order.updated_at = Some(time());

            _insert_supply_order(&supply_order);

            Ok(supply_order)
        }
        None => Err(Error::NotFound {
            msg: format!("Couldn't update a supply order with id={}. Supply order not found", payload.order_id),
        })
    }
}

#[ic_cdk::update]
fn complete_supply_order(id: u64) -> Result<SupplyOrder, Error> {
    match SUPPLY_ORDERS.with(|service| service.borrow().get(&id)) {
        Some(mut supply_order) => {
            supply_order.is_complete = true;
            supply_order.updated_at = Some(time());

            _insert_supply_order(&supply_order);

            Ok(supply_order)
        }
        None => Err(Error::NotFound {
            msg: format!("Couldn't update a supply order with id={}. Supply order not found", id),
        })
    }
}

#[ic_cdk::update]
fn update_supply_order(id: u64, payload: SupplyOrderPayload) -> Option<SupplyOrder> {
    let supply_order = SUPPLY_ORDERS
        .with(|service| service.borrow().get(&id))
        .expect("Supply order does not exist");

    let updated_supply_order = SupplyOrder {
        id: supply_order.id,
        title: payload.title,
        farmer_id: payload.farmer_id,
        wholesaler_id: Some(payload.wholesaler_id),
        product_types: payload.product_types,
        products: payload.products,
        is_complete: payload.is_complete,
        created_at: supply_order.created_at,
        updated_at: Some(time()),
    };

    _insert_supply_order(&updated_supply_order);

    if payload.is_complete {
        _update_supply_order_ids(supply_order);
    }

    Some(updated_supply_order)
}

#[ic_cdk::update]
fn delete_supply_order(id: u64) -> Result<SupplyOrder, Error> {
    match SUPPLY_ORDERS.with(|supply_orders| supply_orders.borrow_mut().remove(&id)) {
        Some(supply_order) => Ok(supply_order),
        None => Err(Error::NotFound {
            msg: format!("Supply order id:{} deletion unsuccessful. Supply order not found", id),
        }),
    }
}

fn _get_supply_order(id: &u64) -> Option<SupplyOrder> {
    SUPPLY_ORDERS.with(|supply_orders| supply_orders.borrow().get(&id))
}

fn _insert_supply_order(supply_order: &SupplyOrder) {
    SUPPLY_ORDERS.with(|supply_orders| {
        supply_orders.borrow_mut().insert(supply_order.id, supply_order.clone())
    });
}

fn _update_supply_order_ids(supply_order: SupplyOrder) {
    FARMER_STORAGE.with(|farmers| {
        let mut farmer = farmers.borrow_mut().get(&supply_order.farmer_id).unwrap();
        farmer.product_ids.push(supply_order.id);
    });

    if let Some(wholesaler_id) = supply_order.wholesaler_id {
        WHOLESALER_STORAGE.with(|wholesalers| {
            let mut wholesaler = wholesalers.borrow_mut().get(&wholesaler_id).unwrap();
            wholesaler.order_ids.push(supply_order.id);
        });
    }
}

// Add this to the existing Error enum
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    SupplyOrderNotFound { msg: String },
}


// Candid generator for exporting the Candid interface
ic_cdk::export_candid!();
