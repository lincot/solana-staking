use anchor_lang::prelude::*;

#[account]
pub struct Registrar {
    pub authority: Pubkey,
    pub nonce: u8,
    pub withdrawal_timelock: i64,
    pub reward_queue: Pubkey,
    pub mint: Pubkey,
    pub pool_mint: Pubkey,
    pub stake_rate: u64,
}

#[derive(Default, Clone, Copy, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct Reward {
    pub vendor: Pubkey,
    pub ts: i64,
}

#[account]
pub struct RewardQueue {
    pub head: u32,
    pub tail: u32,
    pub events: Vec<Reward>,
}

impl RewardQueue {
    pub fn append(&mut self, reward: Reward) -> Result<u32> {
        let cursor = self.head;

        let h_idx = self.index_of(self.head);
        self.events[h_idx] = reward;

        let is_full = self.index_of(self.head + 1) == self.index_of(self.tail);
        if is_full {
            self.tail += 1;
        }
        self.head += 1;

        Ok(cursor)
    }

    pub fn index_of(&self, counter: u32) -> usize {
        counter as usize % self.capacity()
    }

    pub fn capacity(&self) -> usize {
        self.events.len()
    }

    pub fn get(&self, cursor: u32) -> &Reward {
        &self.events[cursor as usize % self.capacity()]
    }

    pub const fn head(&self) -> u32 {
        self.head
    }

    pub const fn tail(&self) -> u32 {
        self.tail
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Default, Debug, Clone, PartialEq)]
pub struct BalanceSandbox {
    pub spt: Pubkey,
    pub available: Pubkey,
    pub stake: Pubkey,
    pub pending: Pubkey,
}

#[account]
pub struct Member {
    pub registrar: Pubkey,
    pub beneficiary: Pubkey,
    pub metadata: Pubkey,
    pub balances: BalanceSandbox,
    pub rewards_cursor: u32,
    pub last_stake_ts: i64,
    pub nonce: u8,
}

#[account]
pub struct RewardVendor {
    pub registrar: Pubkey,
    pub vault: Pubkey,
    pub mint: Pubkey,
    pub nonce: u8,
    pub pool_token_supply: u64,
    pub reward_event_q_cursor: u32,
    pub start_ts: i64,
    pub expiry_ts: i64,
    pub expiry_receiver: Pubkey,
    pub from: Pubkey,
    pub total: u64,
    pub expired: bool,
}

#[account]
pub struct PendingWithdrawal {
    pub registrar: Pubkey,
    pub member: Pubkey,
    pub burned: bool,
    pub pool: Pubkey,
    pub start_ts: i64,
    pub end_ts: i64,
    pub amount: u64,
}
