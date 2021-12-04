use cosmwasm_std::Addr;
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};
use internnft::nft::{Config, InternTokenInfo};

pub const CONFIG: Item<Config> = Item::new("config");

pub struct TokenIndexes<'a> {
    pub owner: MultiIndex<'a, (Addr, Vec<u8>), InternTokenInfo>,
}

impl<'a> IndexList<InternTokenInfo> for TokenIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<InternTokenInfo>> + '_> {
        let v: Vec<&dyn Index<InternTokenInfo>> = vec![&self.owner];
        Box::new(v.into_iter())
    }
}

pub fn tokens<'a>() -> IndexedMap<'a, &'a str, InternTokenInfo, TokenIndexes<'a>> {
    let indexes = TokenIndexes {
        owner: MultiIndex::new(
            |d: &InternTokenInfo, k: Vec<u8>| (d.owner.clone(), k),
            "tokens",
            "tokens__owner",
        ),
    };
    IndexedMap::new("tokens", indexes)
}

pub const OWNER: Item<String> = Item::new("owner");
