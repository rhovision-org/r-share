//! Native REV and r-cash account funding.
//!
//! `fund_account_rev` moves the caller's own native REV into their r-cash account through
//! ERTP `revFund`. It spends the caller's native vault; it does not mint. r-share is
//! devnet-only, so the CLI and UI label this path accordingly.
//!
//! `withdraw_rev_to` is the reverse bridge: account `withdraw` → ERTP purse → `revRedeem`
//! to a named REV address.

use rcash_chain::rho_expr::RhoValue;
use serde_json::Value;

use rshare_core::render::quote_string;
use rshare_core::ComponentKey;

use crate::error::{Result, SdkError};
use crate::wire;
use crate::writes::Write;
use crate::Client;

const FUND_TERM: &str = r#"new rl(`rho:registry:lookup`), paymentsCh, accountCh, facetCh,
    ertpCh, issuerCh, purseCh, fundCh, wdCh, depCh,
    deployerId(`rho:rchain:deployerId`), deployId(`rho:rchain:deployId`) in {
  rl!(`__PAYMENTS__`, *paymentsCh) |
  for (Payments <- paymentsCh) {
    Payments!("ensureAccount", *deployerId, *accountCh) |
    for (@a <- accountCh) {
      match a {
        (true, account, ownerId) => {
          @account!("depositFacet", "REV", *facetCh) |
          for (@fr <- facetCh) {
            match fr {
              (true, facet) => {
                rl!(`rho:rchain:ertp`, *ertpCh) |
                for (@(_, ERTP) <- ertpCh) {
                  @ERTP!("getRevIssuer", *issuerCh) |
                  for (@(revBrand, revIssuer) <- issuerCh) {
                    @revIssuer!("makeEmptyPurse", *purseCh) |
                    for (@(true, purse) <- purseCh) {
                      @purse!("revFund", *deployerId, __AMOUNT__, *fundCh) |
                      for (@_ <- fundCh) {
                        @purse!("withdraw", (revBrand, __AMOUNT__), *wdCh) |
                        for (@(true, payment) <- wdCh) {
                          @facet!("deposit", payment, *depCh) |
                          for (@d <- depCh) {
                            match d {
                              (true, balance) => { deployId!((true, ownerId, balance)) }
                              _ => { deployId!(d) }
                            }
                          }
                        }
                      }
                    }
                  }
                }
              }
              _ => { deployId!(fr) }
            }
          }
        }
        _ => { deployId!(a) }
      }
    }
  }
}"#;

const WITHDRAW_TERM: &str = r#"new rl(`rho:registry:lookup`), paymentsCh, accountCh, wdCh,
    ertpCh, issuerCh, purseCh, depCh, redCh,
    deployerId(`rho:rchain:deployerId`), deployId(`rho:rchain:deployId`) in {
  rl!(`__PAYMENTS__`, *paymentsCh) |
  for (Payments <- paymentsCh) {
    Payments!("ensureAccount", *deployerId, *accountCh) |
    for (@a <- accountCh) {
      match a {
        (true, account, ownerId) => {
          @account!("withdraw", *deployerId, "REV", __AMOUNT__, *wdCh) |
          for (@(true, payment) <- wdCh) {
            rl!(`rho:rchain:ertp`, *ertpCh) |
            for (@(_, ERTP) <- ertpCh) {
              @ERTP!("getRevIssuer", *issuerCh) |
              for (@(revBrand, revIssuer) <- issuerCh) {
                @revIssuer!("makeEmptyPurse", *purseCh) |
                for (@(true, purse) <- purseCh) {
                  @purse!("deposit", payment, *depCh) |
                  for (@_ <- depCh) {
                    @purse!("revRedeem", __AMOUNT__, __ADDRESS__, *redCh) |
                    for (@r <- redCh) {
                      match r {
                        (true, remaining) => { deployId!((true, ownerId, remaining)) }
                        _ => { deployId!(r) }
                      }
                    }
                  }
                }
              }
            }
          }
        }
        _ => { deployId!(a) }
      }
    }
  }
}"#;

impl Client {
    /// Ask the devnet faucet for native REV. The node's faucet route exists only in dev
    /// mode with a funded deployer key.
    pub async fn faucet(&self, address: &str) -> Result<Value> {
        Ok(self.inner.node().faucet(address).await?)
    }

    /// Move `amount` base units from the caller's native REV vault into their r-cash
    /// account. Returns the resulting account balance.
    pub async fn fund_account_rev(&self, amount: i64) -> Result<Write<i64>> {
        if amount < 1 {
            return Err(SdkError::Decode("amount must be positive".into()));
        }
        let payments = self.resolve(ComponentKey::Payments).await?;
        let term = FUND_TERM
            .replace("__PAYMENTS__", payments.uri.as_str())
            .replace("__AMOUNT__", &amount.to_string());
        let receipt = wire::deploy_receipt(&self.inner, &term).await?;
        let reply = receipt
            .reply
            .clone()
            .ok_or_else(|| SdkError::Decode("fund returned no value".into()))?;
        let values = wire::decode_reply(&reply)?;
        let balance = values
            .get(1)
            .and_then(RhoValue::as_int)
            .ok_or_else(|| SdkError::Decode("fund balance is not an integer".into()))?;
        Ok(Write {
            value: balance,
            receipt,
        })
    }

    /// Withdraw account REV to a named native REV address. Returns the remaining purse
    /// balance reported by `revRedeem`.
    pub async fn withdraw_rev_to(&self, amount: i64, address: &str) -> Result<Write<i64>> {
        if amount < 1 {
            return Err(SdkError::Decode("amount must be positive".into()));
        }
        let address = quote_string(address)?;
        let payments = self.resolve(ComponentKey::Payments).await?;
        let term = WITHDRAW_TERM
            .replace("__PAYMENTS__", payments.uri.as_str())
            .replace("__AMOUNT__", &amount.to_string())
            .replace("__ADDRESS__", &address);
        let receipt = wire::deploy_receipt(&self.inner, &term).await?;
        let reply = receipt
            .reply
            .clone()
            .ok_or_else(|| SdkError::Decode("withdraw returned no value".into()))?;
        let values = wire::decode_reply(&reply)?;
        let remaining = values
            .get(1)
            .and_then(RhoValue::as_int)
            .ok_or_else(|| SdkError::Decode("withdraw result is not an integer".into()))?;
        Ok(Write {
            value: remaining,
            receipt,
        })
    }
}
